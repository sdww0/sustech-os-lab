use align_ext::AlignExt;
use alloc::{
    collections::{btree_map::BTreeMap, linked_list::LinkedList},
    sync::Arc,
    vec::Vec,
};
use log::{info, warn};
use ostd::{
    cpu::UserContext,
    mm::{
        CachePolicy, Frame, FrameAllocOptions, PageProperty, Vaddr, VmSpace, MAX_USERSPACE_VADDR,
        PAGE_SIZE,
    },
    sync::SpinLock,
    user::UserContextApi,
};

use crate::process::current_process;

use super::PageFlags;

pub fn handle_page_fault(user_context: &mut UserContext, page_fault_addr: Vaddr) {
    if do_handle_page_fault(user_context, page_fault_addr).is_err() {
        warn!(
            "Handle page fault error, base vaddr: {:x?}, killing process...",
            page_fault_addr
        );
        current_process().unwrap().exit(u32::MAX);
    }
}

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

    /// Just adding the VmMapping, allocate the physical frame when user accessing.
    pub fn add_vm_mapping(&self, mapping: VmMapping) {
        self.mappings.lock().push_back(mapping);
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

        let mut current = mapping.base_vaddr;
        for frame in frames.iter() {
            mapping.frames.insert(current, frame.clone());
            current += PAGE_SIZE;
        }

        self.mappings.lock().push_back(mapping);
        frames
    }

    /// Duplicate self with new phyiscal frames. Also, this will copy the data inside each frame.
    pub fn duplicate_with_new_frames(&self) -> Self {
        let new_memory_space = MemorySpace::new();
        let mut new_mappings = new_memory_space.mappings.lock();

        let mappings = self.mappings.lock();
        for mapping in mappings.iter() {
            // Prepare frames
            let frames = &mapping.frames;
            let mut cursor_mut = new_memory_space
                .vm_space
                .cursor_mut(&(mapping.base_vaddr..(mapping.base_vaddr + mapping.pages * PAGE_SIZE)))
                .unwrap();

            let mut new_frames = BTreeMap::new();
            let mut new_frames_pool = FrameAllocOptions::new(mapping.frames.len())
                .alloc()
                .unwrap();
            for (vaddr, frame) in frames.iter() {
                let new_frame = new_frames_pool.pop().unwrap();
                new_frame.copy_from(frame);
                new_frames.insert(*vaddr, new_frame.clone());
                cursor_mut.map(
                    new_frame,
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
    frames: BTreeMap<Vaddr, Frame>,
    perms: PageFlags,
}

impl VmMapping {
    pub fn new(base_vaddr: Vaddr, pages: usize, perms: PageFlags) -> Self {
        Self {
            base_vaddr,
            pages,
            frames: BTreeMap::new(),
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

fn do_handle_page_fault(
    user_context: &mut UserContext,
    page_fault_addr: Vaddr,
) -> core::result::Result<(), ()> {
    let current_process = current_process().unwrap();
    let current_space = current_process.memory_space();

    // Find mapping
    let mut mapping_some = None;
    let mut mappings = current_space.mappings.lock();
    for mapping in mappings.iter_mut() {
        if page_fault_addr >= mapping.base_vaddr
            && page_fault_addr < mapping.base_vaddr + PAGE_SIZE * mapping.pages
        {
            mapping_some = Some(mapping);
            break;
        }
    }

    if let Some(mapping) = mapping_some {
        let page_fault_base = page_fault_addr.align_down(PAGE_SIZE);

        let mut cursor_mut = current_space
            .vm_space()
            .cursor_mut(&(page_fault_base..page_fault_base + PAGE_SIZE))
            .unwrap();

        // Now, we can handle page fault
        let item = cursor_mut.query().unwrap();
        match item {
            ostd::mm::vm_space::VmItem::NotMapped { va: _, len: _ } => {
                let frame = FrameAllocOptions::new(1).alloc_single().unwrap();
                cursor_mut.map(
                    frame.clone(),
                    PageProperty::new(mapping.perms, CachePolicy::Writeback),
                );
                mapping.frames.insert(page_fault_base, frame);
            }
            ostd::mm::vm_space::VmItem::Mapped {
                va: _,
                frame: _,
                prop: _,
            } => {
                warn!("We cannot handle page fault with mapped pages, skip it");
                user_context.set_instruction_pointer(user_context.instruction_pointer() + 2);
            }
        }
        info!(
            "Successfully handle page fault, base vaddr: {:x?}",
            page_fault_addr
        );
        Ok(())
    } else {
        Err(())
    }
}
