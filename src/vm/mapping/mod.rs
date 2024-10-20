use alloc::{collections::linked_list::LinkedList, sync::Arc, vec::Vec};
use ostd::{
    mm::{
        CachePolicy, Frame, FrameAllocOptions, PageProperty, Vaddr, VmSpace, MAX_USERSPACE_VADDR,
        PAGE_SIZE,
    },
    sync::SpinLock,
};

use super::PageFlags;

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

    pub fn map(&self, mut mapping: VmMapping) -> Vec<Frame> {
        let mut cursor_mut = self
            .vm_space
            .cursor_mut(&(mapping.base_vaddr..(mapping.base_vaddr + mapping.pages * PAGE_SIZE)))
            .unwrap();
        let frames = FrameAllocOptions::new(mapping.pages).alloc().unwrap();
        for frame in frames.iter() {
            cursor_mut.map(
                frame.clone(),
                PageProperty::new(mapping.perms, CachePolicy::Writeback),
            );
        }
        mapping.frames = frames.clone();
        self.mappings.lock().push_back(mapping);
        frames
    }

    /// Duplicate self with new phyiscal frames. Also, this will copy the data inside each frame.
    pub fn duplicate_with_new_frames(&self) -> Self {
        let new_memory_space = MemorySpace::new();
        let mut new_mappings = new_memory_space.mappings.lock();

        let mappings = self.mappings.lock();
        for mapping in mappings.iter() {
            let new_frames = FrameAllocOptions::new(mapping.pages).alloc().unwrap();
            let frames = &mapping.frames;
            let mut cursor_mut = new_memory_space
                .vm_space
                .cursor_mut(&(mapping.base_vaddr..(mapping.base_vaddr + mapping.pages * PAGE_SIZE)))
                .unwrap();
            for (index, frame) in new_frames.iter().enumerate() {
                frame.copy_from(frames.get(index).unwrap());
                cursor_mut.map(
                    frame.clone(),
                    PageProperty::new(mapping.perms, CachePolicy::Writeback),
                );
            }
            let mapping = VmMapping {
                base_vaddr: mapping.base_vaddr,
                pages: mapping.pages,
                frames: new_frames,
                perms: mapping.perms,
            };

            new_mappings.push_back(mapping);
        }
        drop(new_mappings);
        new_memory_space
    }

    pub fn vm_space(&self) -> &Arc<VmSpace> {
        &self.vm_space
    }

    pub fn clear(&self) {
        let mut cursor = self.vm_space.cursor_mut(&(0..MAX_USERSPACE_VADDR)).unwrap();
        cursor.unmap(MAX_USERSPACE_VADDR);
        self.mappings.lock().clear();
    }
}

impl Default for MemorySpace {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct VmMapping {
    base_vaddr: Vaddr,
    /// Mapping page count with PAGE_SIZE as unit.
    pages: usize,
    frames: Vec<Frame>,
    perms: PageFlags,
}

impl VmMapping {
    pub fn new(base_vaddr: Vaddr, pages: usize, perms: PageFlags) -> Self {
        Self {
            base_vaddr,
            pages,
            frames: Vec::new(),
            perms,
        }
    }

    pub fn base_vaddr(&self) -> Vaddr {
        self.base_vaddr
    }

    pub fn size(&self) -> usize {
        self.pages
    }

    pub fn vm_perms(&self) -> PageFlags {
        self.perms
    }
}
