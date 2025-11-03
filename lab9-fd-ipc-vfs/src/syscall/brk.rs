use alloc::sync::Arc;
use log::debug;

use crate::error::Result;
use crate::process::Process;
use crate::syscall::SyscallReturn;

pub fn sys_brk(new_brk: usize, current_process: &Arc<Process>) -> Result<SyscallReturn> {
    let val = if new_brk == 0 { None } else { Some(new_brk) };
    let ret = current_process.heap().brk(val).unwrap();
    debug!("[SYS_BRK] new_brk: {:#x?}, return: {:#x}", new_brk, ret);
    Ok(SyscallReturn(ret as _))
}
