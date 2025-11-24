use core::fmt::Debug;

use crate::{
    error::{Errno, Error, Result},
    mm::VmMapping,
    process::Process,
};
use align_ext::AlignExt;
use alloc::{collections::linked_list::LinkedList, sync::Arc};
use log::error;
use ostd::{
    irq::disable_local,
    mm::{CachePolicy, FrameAllocOptions, PAGE_SIZE, PageFlags, PageProperty, Vaddr},
};
use riscv::register::scause::Exception;

pub struct PageFaultContext<'a> {
    perms: PageFlags,
    mappings: &'a mut LinkedList<VmMapping>,
    process: &'a Arc<Process>,
    vaddr: Vaddr,
    fault: Exception,
}

impl PageFaultContext<'_> {
    pub fn new<'a>(
        perms: PageFlags,
        mappings: &'a mut LinkedList<VmMapping>,
        process: &'a Arc<Process>,
        vaddr: Vaddr,
        fault: Exception,
    ) -> PageFaultContext<'a> {
        PageFaultContext {
            perms,
            mappings,
            process,
            vaddr,
            fault,
        }
    }
}

pub trait PageFaultHandler: Send + Sync + Debug {
    fn handle_page_fault<'a>(&self, context: PageFaultContext<'a>) -> Result<()>;
}

#[derive(Debug)]
pub struct DefaultPageFaultHandler;

impl PageFaultHandler for DefaultPageFaultHandler {
    fn handle_page_fault<'a>(&self, context: PageFaultContext<'a>) -> Result<()> {
        error!(
            "Unhandled page fault at address {:x?}, exception code: {:?}",
            context.vaddr, context.fault
        );
        Err(Error::new(Errno::EACCES))
    }
}

#[derive(Debug)]
pub struct AllocationPageFaultHandler;

impl PageFaultHandler for AllocationPageFaultHandler {
    fn handle_page_fault<'a>(&self, context: PageFaultContext<'a>) -> Result<()> {
        let memory_space = context.process.memory_space();
        let vm_space = memory_space.vm_space();
        let frame = FrameAllocOptions::new().alloc_frame().unwrap();
        let align_down_vaddr = context.vaddr.align_down(PAGE_SIZE);

        let guard = disable_local();
        let mut cursor_mut = vm_space
            .cursor_mut(&guard, &(align_down_vaddr..align_down_vaddr + PAGE_SIZE))
            .unwrap();
        cursor_mut.map(
            frame.clone().into(),
            PageProperty::new_user(context.perms, CachePolicy::Writeback),
        );

        // Add mapping
        let mapping = VmMapping::new(align_down_vaddr, context.perms, frame);
        context.mappings.push_back(mapping);

        Ok(())
    }
}
