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
    debug!("[SYS_WAIT4] Wait pid: {:?}, exit_status_ptr: {:x?}, wait_options: {:x?}, rusage_addr: {:x?}", wait_pid, exit_status_ptr,wait_options,rusage_addr);
    let current = current_process().unwrap();
    debug!("[SYS_WAIT4] current pid: {:?}", current.pid());
    let (pid, exit_code) = wait_child(wait_pid, current.clone())?;

    if exit_status_ptr != 0 {
        let vm_space = current.memory_space().vm_space();
        let mut writer = vm_space.writer(exit_status_ptr, size_of::<u32>())?;
        writer.write_val(&exit_code)?;
    }

    Ok(SyscallReturn::Return(pid as isize))
}
