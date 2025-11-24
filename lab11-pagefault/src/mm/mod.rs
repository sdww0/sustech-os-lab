pub mod area;
pub mod mapping;

use align_ext::AlignExt;
use alloc::{collections::linked_list::LinkedList, sync::Arc};
pub use mapping::VmMapping;
use ostd::{
    arch::cpu::context::CpuExceptionInfo,
    mm::{
        CachePolicy, FrameAllocOptions, MAX_USERSPACE_VADDR, PAGE_SIZE, PageFlags, PageProperty,
        Segment, VmSpace, io_util::HasVmReaderWriter,
    },
    sync::SpinLock,
    task::disable_preempt,
};

use crate::{
    mm::area::VmArea,
    process::{Process, USER_STACK_SIZE},
};

pub fn page_fault_handler(
    process: &Arc<Process>,
    cpu_exception: &CpuExceptionInfo,
) -> core::result::Result<(), ()> {
    let memory_space = process.memory_space();
    let page_fault_addr = cpu_exception.page_fault_addr;

    // Stack
    let stack_low = 0x40_0000_0000 - 10 * PAGE_SIZE - USER_STACK_SIZE;
    let stack_high = 0x40_0000_0000 - 10 * PAGE_SIZE;
    if (stack_low..stack_high).contains(&page_fault_addr) {
        memory_space.map(VmArea::new(
            page_fault_addr.align_down(PAGE_SIZE),
            1,
            PageFlags::RW,
        ));
        return Ok(());
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
            let mut mapping = VmMapping::new(area.base_vaddr() + i * PAGE_SIZE, area.perms());
            mapping.set_frame(frame);
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
            let mut new_area = VmArea::new(area.base_vaddr(), area.pages(), area.perms());

            let mut cursor_mut = new_memory_space
                .vm_space
                .cursor_mut(
                    &guard,
                    &(new_area.base_vaddr()
                        ..(new_area.base_vaddr() + new_area.pages() * PAGE_SIZE)),
                )
                .unwrap();

            let new_frames = FrameAllocOptions::new()
                .alloc_segment(new_area.pages())
                .unwrap();
            let mut old_frames_iter = area.mappings().iter().map(|mapping| mapping.frame());
            for (i, new_frame) in new_frames.enumerate() {
                // Copy data from old frame to new frame
                new_frame
                    .writer()
                    .write(&mut old_frames_iter.next().unwrap().reader());

                // Map new frame
                cursor_mut.map(
                    new_frame.clone().into(),
                    PageProperty::new_user(area.perms(), CachePolicy::Writeback),
                );

                let mut mapping = VmMapping::new(area.base_vaddr() + i * PAGE_SIZE, area.perms());
                mapping.set_frame(new_frame.clone());
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
