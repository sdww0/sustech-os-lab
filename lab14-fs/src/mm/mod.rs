pub mod area;
pub mod fault;
pub mod mapping;

use alloc::{collections::linked_list::LinkedList, sync::Arc};
pub use mapping::VmMapping;
use ostd::{
    arch::cpu::context::CpuExceptionInfo,
    mm::{
        CachePolicy, FrameAllocOptions, MAX_USERSPACE_VADDR, PAGE_SIZE, PageProperty, Segment,
        VmSpace, io_util::HasVmReaderWriter,
    },
    sync::SpinLock,
    task::disable_preempt,
};

use crate::{mm::area::VmArea, process::Process};

pub fn page_fault_handler(
    process: &Arc<Process>,
    cpu_exception: &CpuExceptionInfo,
) -> core::result::Result<(), ()> {
    let memory_space = process.memory_space();
    let page_fault_addr = cpu_exception.page_fault_addr;

    let mut areas = memory_space.areas.lock();
    for area in areas.iter_mut() {
        if !area.contains_vaddr(page_fault_addr) {
            continue;
        }

        return area
            .handle_page_fault(process, page_fault_addr, cpu_exception.code)
            .map_err(|_| ());
    }

    Err(())
}

pub struct MemorySpace {
    vm_space: Arc<VmSpace>,
    areas: SpinLock<LinkedList<VmArea>>,
}

impl MemorySpace {
    pub fn new() -> Self {
        Self {
            vm_space: Arc::new(VmSpace::new()),
            areas: SpinLock::new(LinkedList::new()),
        }
    }

    /// Add a virtual memory area without initializing the frames.
    pub fn add_area(&self, area: VmArea) {
        self.areas.lock().push_back(area);
    }

    pub fn map(&self, mut area: VmArea) -> Segment<()> {
        let guard = disable_preempt();

        let mut cursor_mut = self
            .vm_space
            .cursor_mut(
                &guard,
                &(area.base_vaddr()..(area.base_vaddr() + area.pages() * PAGE_SIZE)),
            )
            .unwrap();

        let frames = FrameAllocOptions::new()
            .alloc_segment(area.pages())
            .unwrap();
        for (i, frame) in frames.clone().enumerate() {
            cursor_mut.map(
                frame.clone().into(),
                PageProperty::new_user(area.perms(), CachePolicy::Writeback),
            );

            // Add mapping
            let mapping = VmMapping::new(area.base_vaddr() + i * PAGE_SIZE, area.perms(), frame);
            area.add_mapping(mapping);
        }

        self.areas.lock().push_back(area);

        frames
    }

    /// Duplicate self with new phyiscal frames. Also, this will copy the data inside each frame.
    pub fn duplicate(&self) -> Self {
        let new_memory_space = MemorySpace::new();
        let mut new_mappings = new_memory_space.areas.lock();

        let guard = disable_preempt();
        let areas = self.areas.lock();
        for area in areas.iter() {
            let mut new_area = VmArea::new_with_handler(
                area.base_vaddr(),
                area.pages(),
                area.perms(),
                area.page_fault_handler().clone(),
            );

            let old_mappings = area.mappings().iter().map(|mapping| mapping);
            for old_mapping in old_mappings {
                let new_frame = FrameAllocOptions::new().alloc_frame().unwrap();

                // Copy data from old frame to new frame
                new_frame.writer().write(&mut old_mapping.frame().reader());

                let mut cursor_mut = new_memory_space
                    .vm_space
                    .cursor_mut(
                        &guard,
                        &(old_mapping.base_vaddr()..(old_mapping.base_vaddr() + PAGE_SIZE)),
                    )
                    .unwrap();
                // Map new frame
                cursor_mut.map(
                    new_frame.clone().into(),
                    PageProperty::new_user(new_area.perms(), CachePolicy::Writeback),
                );

                let mapping = VmMapping::new(old_mapping.base_vaddr(), new_area.perms(), new_frame);
                new_area.add_mapping(mapping);
            }

            new_mappings.push_back(new_area);
        }
        drop(new_mappings);
        new_memory_space
    }

    pub fn vm_space(&self) -> &Arc<VmSpace> {
        &self.vm_space
    }

    pub fn clear(&self) {
        let guard = disable_preempt();
        let mut cursor = self
            .vm_space
            .cursor_mut(&guard, &(0..MAX_USERSPACE_VADDR))
            .unwrap();
        cursor.unmap(MAX_USERSPACE_VADDR);
        self.areas.lock().clear();
    }
}

impl Default for MemorySpace {
    fn default() -> Self {
        Self::new()
    }
}
