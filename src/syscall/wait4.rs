use crate::{
    prelude::*,
    process::{current_process, wait::wait_child},
};

pub fn sys_wait4(
    wait_pid: i32,
    exit_status_ptr: Vaddr,
    wait_options: u32,
    rusage_addr: Vaddr,
) -> Result<SyscallReturn> {
    debug!("[SYS_WAIT4] Wait pid: {:?}, exit_status_ptr: {:x?}, wait_options: {:x?}, rusage_addr: {:x?}",wait_pid, exit_status_ptr,wait_options,rusage_addr);
    let current = current_process().unwrap();
    debug!("[SYS_WAIT4] current pid: {:?}", current.pid());
    let (pid, _exit_code) = wait_child(wait_pid, current)?;

    // TODO: Write exit code to exit_status_ptr

    Ok(SyscallReturn::Return(pid as isize))
}
