use alloc::sync::Arc;
use log::debug;
use ostd::mm::Vaddr;

use crate::error::Result;
use crate::process::Process;
use crate::syscall::SyscallReturn;

pub fn sys_wait4(
    wait_pid: i32,
    exit_status_ptr: Vaddr,
    wait_options: u32,
    rusage_addr: Vaddr,
    current_process: &Arc<Process>,
) -> Result<SyscallReturn> {
    debug!(
        "[SYS_WAIT4] Wait pid: {:?}, exit_status_ptr: {:x?}, wait_options: {:x?}, rusage_addr: {:x?}",
        wait_pid, exit_status_ptr, wait_options, rusage_addr
    );

    let (pid, exit_code) = current_process.wait(wait_pid)?;

    // Write the exit code to the user space
    if exit_status_ptr != 0 {
        current_process.memory_space().vm_space().activate();
        current_process
            .memory_space()
            .vm_space()
            .writer(exit_status_ptr, 4)
            .unwrap()
            .write_val(&exit_code)
            .unwrap();
    }

    Ok(SyscallReturn(pid as _))
}
