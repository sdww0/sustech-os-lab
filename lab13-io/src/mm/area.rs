use alloc::{collections::linked_list::LinkedList, sync::Arc};
use ostd::mm::{PAGE_SIZE, PageFlags, Vaddr};
use riscv::register::scause::Exception;

use crate::{
    mm::{
        VmMapping,
        fault::{DefaultPageFaultHandler, PageFaultContext, PageFaultHandler},
    },
    process::Process,
};

/// Represents a continous virtual memory area, which consists of multiple mappings.
#[derive(Debug)]
pub struct VmArea {
    base_vaddr: Vaddr,
    /// Mapping page count with PAGE_SIZE as unit.
    pages: usize,
    perms: PageFlags,
    mappings: LinkedList<VmMapping>,
    fault_handler: Arc<dyn PageFaultHandler>,
}

impl VmArea {
    pub fn new(base_vaddr: Vaddr, pages: usize, perms: PageFlags) -> Self {
        Self {
            base_vaddr,
            pages,
            perms,
            mappings: LinkedList::new(),
            fault_handler: Arc::new(DefaultPageFaultHandler),
        }
    }

    pub fn new_with_handler(
        base_vaddr: Vaddr,
        pages: usize,
        perms: PageFlags,
        fault_handler: Arc<dyn PageFaultHandler>,
    ) -> Self {
        Self {
            base_vaddr,
            pages,
            perms,
            mappings: LinkedList::new(),
            fault_handler,
        }
    }

    pub fn handle_page_fault(
        &mut self,
        process: &Arc<Process>,
        vaddr: Vaddr,
        fault: Exception,
    ) -> crate::error::Result<()> {
        debug_assert!(
            self.contains_vaddr(vaddr),
            "VmArea does not contain vaddr {:x?}",
            vaddr
        );
        self.fault_handler.handle_page_fault(PageFaultContext::new(
            self.perms,
            &mut self.mappings,
            process,
            vaddr,
            fault,
        ))
    }

    pub fn page_fault_handler(&self) -> &Arc<dyn PageFaultHandler> {
        &self.fault_handler
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
