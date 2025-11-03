pub mod mapping;

use alloc::{collections::linked_list::LinkedList, sync::Arc};
pub use mapping::VmMapping;
use ostd::{
    mm::{
        CachePolicy, FrameAllocOptions, MAX_USERSPACE_VADDR, PAGE_SIZE, PageProperty, Segment,
        VmSpace, io_util::HasVmReaderWriter,
    },
    sync::SpinLock,
    task::disable_preempt,
};

pub struct MemorySpace {
    vm_space: Arc<VmSpace>,
    mappings: SpinLock<LinkedList<VmMapping>>,
}

impl MemorySpace {
    pub fn new() -> Self {
        Self {
            vm_space: Arc::new(VmSpace::new()),
            mappings: SpinLock::new(LinkedList::new()),
        }
    }

    pub fn map(&self, mut mapping: VmMapping) -> Segment<()> {
        let guard = disable_preempt();
        let mut cursor_mut = self
            .vm_space
            .cursor_mut(
                &guard,
                &(mapping.base_vaddr()..(mapping.base_vaddr() + mapping.pages() * PAGE_SIZE)),
            )
            .unwrap();
        let frames = FrameAllocOptions::new()
            .alloc_segment(mapping.pages())
            .unwrap();
        for frame in frames.clone() {
            cursor_mut.map(
                frame.into(),
                PageProperty::new_user(mapping.perms(), CachePolicy::Writeback),
            );
        }
        mapping.set_frames(frames.clone());
        self.mappings.lock().push_back(mapping);
        frames
    }

    /// Duplicate self with new phyiscal frames. Also, this will copy the data inside each frame.
    pub fn duplicate(&self) -> Self {
        let new_memory_space = MemorySpace::new();
        let mut new_mappings = new_memory_space.mappings.lock();

        let guard = disable_preempt();
        let mappings = self.mappings.lock();
        for mapping in mappings.iter() {
            let new_frames = FrameAllocOptions::new()
                .alloc_segment(mapping.pages())
                .unwrap();

            let frames = mapping.frames();
            let mut cursor_mut = new_memory_space
                .vm_space
                .cursor_mut(
                    &guard,
                    &(mapping.base_vaddr()..(mapping.base_vaddr() + mapping.pages() * PAGE_SIZE)),
                )
                .unwrap();
            new_frames.writer().write(&mut frames.reader());

            for frame in new_frames.clone() {
                cursor_mut.map(
                    frame.into(),
                    PageProperty::new_user(mapping.perms(), CachePolicy::Writeback),
                );
            }
            let mut mapping =
                VmMapping::new(mapping.base_vaddr(), mapping.pages(), mapping.perms());
            mapping.set_frames(new_frames);

            new_mappings.push_back(mapping);
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
        self.mappings.lock().clear();
    }
}

impl Default for MemorySpace {
    fn default() -> Self {
        Self::new()
    }
}
