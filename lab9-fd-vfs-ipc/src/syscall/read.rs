use alloc::sync::Arc;
use log::debug;
use ostd::mm::Vaddr;

use super::SyscallReturn;
use crate::error::Result;
use crate::{
    error::{Errno, Error},
    process::Process,
};

pub fn sys_read(
    fd: i32,
    user_buf_addr: Vaddr,
    buf_len: usize,
    current_process: &Arc<Process>,
) -> Result<SyscallReturn> {
    debug!(
        "fd: {:?}, user_buf_addr: 0x{:x?}, buf_len: {:?}",
        fd, user_buf_addr, buf_len
    );

    if fd != 0 as i32 || buf_len == 0 {
        return Err(Error::new(Errno::ENOSYS));
    }

    let writer = current_process
        .memory_space()
        .vm_space()
        .writer(user_buf_addr, buf_len)
        .unwrap();

    let file_table = current_process.file_table();
    let file = file_table.get(fd).ok_or(Error::new(Errno::EBADF))?;
    let read_len = file.file().read(writer)?;

    Ok(SyscallReturn(read_len as _))
}
