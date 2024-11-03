use ostd::mm::Vaddr;

use super::SyscallReturn;
use crate::{
    fs::{stdin, STDIN},
    prelude::*,
    return_errno,
};

pub fn sys_read(fd: i32, user_buf_addr: Vaddr, buf_len: usize) -> Result<SyscallReturn> {
    debug!(
        "fd: {:?}, user_buf_addr: 0x{:x?}, buf_len: {:?}",
        fd, user_buf_addr, buf_len
    );

    if fd != STDIN as i32 || buf_len == 0 {
        return_errno!(Errno::ENOSYS)
    }

    let read_len = stdin::read(user_buf_addr, buf_len)?;

    Ok(SyscallReturn::Return(read_len as _))
}
