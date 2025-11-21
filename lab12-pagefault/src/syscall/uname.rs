use alloc::sync::Arc;
use ostd::Pod;

use crate::error::Result;
use crate::process::Process;
use crate::syscall::SyscallReturn;

const UTS_FIELD_LEN: usize = 65;

#[derive(Debug, Clone, Copy, Pod)]
#[repr(C)]
pub struct UtsName {
    sysname: [u8; UTS_FIELD_LEN],
    nodename: [u8; UTS_FIELD_LEN],
    release: [u8; UTS_FIELD_LEN],
    version: [u8; UTS_FIELD_LEN],
    machine: [u8; UTS_FIELD_LEN],
    domainname: [u8; UTS_FIELD_LEN],
}

pub fn sys_uname(utsname_addr: usize, current_process: &Arc<Process>) -> Result<SyscallReturn> {
    let mut uts = UtsName::new_zeroed();
    uts.sysname[..5].copy_from_slice(b"Linux");
    uts.nodename[..7].copy_from_slice(b"WHITLEY");
    uts.release[..5].copy_from_slice(b"6.8.0");
    uts.version[..6].copy_from_slice(b"#1 SMP");
    uts.machine[..7].copy_from_slice(b"riscv64");

    current_process
        .memory_space()
        .vm_space()
        .writer(utsname_addr, core::mem::size_of::<UtsName>())
        .unwrap()
        .write_val(&uts)
        .unwrap();

    Ok(SyscallReturn(0))
}
