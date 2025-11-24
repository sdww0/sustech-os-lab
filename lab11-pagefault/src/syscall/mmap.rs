use core::fmt::Debug;

use align_ext::AlignExt;
use alloc::sync::Arc;
use ostd::irq::disable_local;
use ostd::mm::io_util::HasVmReaderWriter;
use ostd::mm::{CachePolicy, FrameAllocOptions, PAGE_SIZE, PageFlags, PageProperty, Vaddr};

use crate::error::{Errno, Error, Result};
use crate::fs::Inode;
use crate::mm::VmMapping;
use crate::mm::area::VmArea;
use crate::mm::fault::{PageFaultContext, PageFaultHandler};
use crate::process::Process;
use crate::syscall::SyscallReturn;

bitflags::bitflags! {
    pub struct MMapFlags : u32 {
        const MAP_FIXED           = 0x10;
        const MAP_ANONYMOUS       = 0x20;
        const MAP_32BIT           = 0x40;
        const MAP_GROWSDOWN       = 0x100;
        const MAP_DENYWRITE       = 0x800;
        const MAP_EXECUTABLE      = 0x1000;
        const MAP_LOCKED          = 0x2000;
        const MAP_NORESERVE       = 0x4000;
        const MAP_POPULATE        = 0x8000;
        const MAP_NONBLOCK        = 0x10000;
        const MAP_STACK           = 0x20000;
        const MAP_HUGETLB         = 0x40000;
        const MAP_SYNC            = 0x80000;
        const MAP_FIXED_NOREPLACE = 0x100000;
    }
}

pub fn sys_mmap(
    vaddr: u64,
    length: u64,
    perms: u64,
    flags: u32,
    fd: u64,
    offset: u64,
    current_process: &Arc<Process>,
) -> Result<SyscallReturn> {
    // Current, we only support mmap with file
    // MAP_PRIVATE, MAP_FIXED, no MAP_ANONYMOUS
    assert!(vaddr != 0);
    assert!(vaddr.align_down(PAGE_SIZE as _) == vaddr);
    assert!(offset == 0);
    assert!(flags & 0xf == 0x2);
    let mmap_flags = MMapFlags::from_bits_truncate(flags as u32);
    assert!(mmap_flags == MMapFlags::MAP_FIXED);

    // Now, we can map the file
    let page_flags = PageFlags::from_bits_truncate(perms as _);
    let inode = current_process
        .file_table()
        .get(fd as _)
        .unwrap()
        .file()
        .as_inode()
        .ok_or(Error::new(Errno::EBADF))?;

    let handler = Arc::new(MMapInodeFaultHandler {
        base_vaddr: vaddr as _,
        inode,
    });

    let memory_space = current_process.memory_space();
    memory_space.add_area(VmArea::new_with_handler(
        vaddr as _,
        length.align_up(PAGE_SIZE as _) as usize / PAGE_SIZE,
        page_flags,
        handler,
    ));

    Ok(SyscallReturn(vaddr as _))
}

pub struct MMapInodeFaultHandler {
    base_vaddr: Vaddr,
    inode: Arc<dyn Inode>,
}

impl Debug for MMapInodeFaultHandler {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("MMapInodeFaultHandler")
            .field("base_vaddr", &self.base_vaddr)
            .finish()
    }
}

impl PageFaultHandler for MMapInodeFaultHandler {
    fn handle_page_fault<'a>(&self, context: PageFaultContext<'a>) -> Result<()> {
        let memory_space = context.process.memory_space();
        let vm_space = memory_space.vm_space();
        let frame = FrameAllocOptions::new().alloc_frame().unwrap();
        let align_down_vaddr = context.vaddr.align_down(PAGE_SIZE);

        // Read data from Inode
        self.inode
            .read_at(
                align_down_vaddr - self.base_vaddr,
                frame.writer().to_fallible(),
            )
            .unwrap();

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
