use alloc::sync::Arc;
use log::debug;

use crate::error::Result;
use crate::process::Process;
use crate::syscall::SyscallReturn;

pub fn sys_exit(exit_code: u32, current_process: &Arc<Process>) -> Result<SyscallReturn> {
    debug!(
        "[pid: {}] exit with code: {}",
        current_process.pid(),
        exit_code
    );
    if current_process.is_zombie() {
        debug!("[pid: {}] has already exited", current_process.pid());
    } else {
        current_process.exit(exit_code);
    }
    Ok(SyscallReturn(0))
}
