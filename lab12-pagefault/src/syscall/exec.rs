use core::ffi::CStr;

use alloc::sync::Arc;
use alloc::vec;
use log::{debug, info};
use ostd::arch::cpu::context::UserContext;
use ostd::mm::{FallibleVmRead, Vaddr, VmWriter};

use crate::error::Result;
use crate::process::Process;
use crate::syscall::SyscallReturn;

pub fn sys_execve(
    path: Vaddr, /* &[u8] */
    argv: Vaddr, /* &[&str] */
    envp: Vaddr, /* &[&str] */
    current_process: &Arc<Process>,
    user_context: &mut UserContext,
) -> Result<SyscallReturn> {
    debug!(
        "[SYS_EXECVE] path vaddr: {:#x?}, argv vaddr: {:#x?}, envp vaddr: {:#x?}",
        path, argv, envp
    );

    // We ignore the argv and envp for now.
    // The max file name: 255 bytes + 1(\0)
    const MAX_FILENAME_LENGTH: usize = 256;
    let mut buffer = vec![0u8; MAX_FILENAME_LENGTH];
    current_process
        .memory_space()
        .vm_space()
        .reader(path, MAX_FILENAME_LENGTH)
        .unwrap()
        .read_fallible(&mut VmWriter::from(&mut buffer as &mut [u8]))
        .unwrap();

    let exec_name = CStr::from_bytes_until_nul(&buffer)
        .unwrap()
        .to_str()
        .unwrap();

    info!("[SYS_EXECVE] Execute program path: {}", exec_name);

    let binary = crate::progs::lookup_progs(exec_name)?;

    // Do exec:
    // 1. Cleanup all the memory space, including heap
    // 2. Change the user context to zero
    // 3. Parse ELF and load program

    *user_context = current_process.exec(binary);

    Ok(SyscallReturn(0 as _))
}
