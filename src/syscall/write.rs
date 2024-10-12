use core::str;

use alloc::vec;
use log::debug;
use ostd::{
    early_print,
    mm::{FallibleVmRead, Vaddr, VmWriter},
    Pod,
};

use super::SyscallReturn;
use crate::{prelude::*, process::current_process};

pub const STDIN: i32 = 0;
pub const STDOUT: i32 = 1;
pub const STDERR: i32 = 2;

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod)]
pub struct IoVec {
    base: Vaddr,
    len: usize,
}

pub fn sys_writev(fd: i32, io_vec_ptr: Vaddr, io_vec_count: usize) -> Result<SyscallReturn> {
    debug!(
        "[SYS_WRITEV] Fd: {:?}, vec ptr: {:x?}, vec count: {:?}",
        fd, io_vec_ptr, io_vec_count
    );

    let mut total_len = 0;

    let mut current_addr = io_vec_ptr;
    let process = current_process().unwrap();
    let user_space = process.user_space().unwrap();

    for _ in 0..io_vec_count {
        let mut reader = user_space
            .vm_space()
            .reader(current_addr, size_of::<IoVec>())?;
        let io_vec: IoVec = reader.read_val()?;

        let mut buf = vec![0u8; io_vec.len];

        let mut buffer = user_space.vm_space().reader(io_vec.base, io_vec.len)?;
        buffer
            .read_fallible(&mut VmWriter::from(&mut buf as &mut [u8]))
            .unwrap();

        total_len += io_vec.len;
        early_print!("{}", str::from_utf8(&buf).unwrap());
        current_addr += size_of::<IoVec>();
    }

    Ok(SyscallReturn::Return(total_len as isize))
}
