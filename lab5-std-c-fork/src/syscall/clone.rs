use alloc::sync::Arc;
use log::debug;
use ostd::arch::cpu::context::UserContext;
use ostd::mm::Vaddr;

use crate::error::Result;
use crate::process::Process;
use crate::syscall::SyscallReturn;

pub fn sys_clone(
    clone_flags: u64,
    child_stack: u64,
    parent_tidptr: Vaddr,
    tls: u64,
    child_tidptr: Vaddr,
    current_process: &Arc<Process>,
    user_context: &mut UserContext,
) -> Result<SyscallReturn> {
    debug!(
        "[SYS_CLONE] clone_flags: {:#x}, child_stack: {:#x}, parent_tidptr: {:#x}, tls: {:#x}, child_tidptr: {:#x}",
        clone_flags, child_stack, parent_tidptr, tls, child_tidptr
    );

    // TODO-2: Implement clone syscall
    // First, we need to duplicate the current process's memory space and registers.
    // Then, we need to create the child process structure with the duplicated memory space and registers.
    // Next, we need to set the return value of the child process to 0, and setup the process tree.
    // Finally, we run the child process and return its PID to the parent process.

    Ok(SyscallReturn(0 as _))
}
