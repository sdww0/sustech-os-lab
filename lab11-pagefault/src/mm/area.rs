use alloc::collections::linked_list::LinkedList;
use ostd::mm::{PAGE_SIZE, PageFlags, Vaddr};

use crate::mm::VmMapping;

/// Represents a continous virtual memory area, which consists of multiple mappings.
#[derive(Debug, Clone)]
pub struct VmArea {
    base_vaddr: Vaddr,
    /// Mapping page count with PAGE_SIZE as unit.
    pages: usize,
    perms: PageFlags,
    mappings: LinkedList<VmMapping>,
}

impl VmArea {
    pub fn new(base_vaddr: Vaddr, pages: usize, perms: PageFlags) -> Self {
        Self {
            base_vaddr,
            pages,
            perms,
            mappings: LinkedList::new(),
        }
    }

    pub fn perms(&self) -> PageFlags {
        self.perms
    }

    pub fn add_mapping(&mut self, mapping: VmMapping) {
        self.mappings.push_back(mapping);
    }

    pub fn mappings_mut(&mut self) -> &mut LinkedList<VmMapping> {
        &mut self.mappings
    }

    pub fn mappings(&self) -> &LinkedList<VmMapping> {
        &self.mappings
    }

    pub fn base_vaddr(&self) -> Vaddr {
        self.base_vaddr
    }

    pub fn pages(&self) -> usize {
        self.pages
    }

    pub fn contains_vaddr(&self, vaddr: Vaddr) -> bool {
        vaddr >= self.base_vaddr && vaddr < self.base_vaddr + self.pages * PAGE_SIZE
    }
}
