use core::str;

use alloc::{sync::Arc, vec};
use log::debug;
use ostd::{
    Pod, early_print,
    mm::{FallibleVmRead, Vaddr, VmWriter},
};

use crate::{error::Result, process::Process, syscall::SyscallReturn};

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod)]
pub struct IoVec {
    base: Vaddr,
    len: usize,
}

pub fn sys_writev(
    fd: i32,
    io_vec_ptr: Vaddr,
    io_vec_count: usize,
    current_process: &Arc<Process>,
) -> Result<SyscallReturn> {
    debug!(
        "[SYS_WRITEV] Fd: {:?}, vec ptr: {:x?}, vec count: {:?}",
        fd, io_vec_ptr, io_vec_count
    );

    let mut total_len = 0;

    let mut current_addr = io_vec_ptr;
    let memory_space = current_process.memory_space();

    for _ in 0..io_vec_count {
        let mut reader = memory_space
            .vm_space()
            .reader(current_addr, size_of::<IoVec>())
            .unwrap();
        let io_vec: IoVec = reader.read_val().unwrap();

        let mut buf = vec![0u8; io_vec.len];

        let mut buffer = memory_space
            .vm_space()
            .reader(io_vec.base, io_vec.len)
            .unwrap();
        buffer
            .read_fallible(&mut VmWriter::from(&mut buf as &mut [u8]))
            .unwrap();

        total_len += io_vec.len;
        early_print!("{}", str::from_utf8(&buf).unwrap());
        current_addr += size_of::<IoVec>();
    }

    Ok(SyscallReturn(total_len as _))
}

pub fn sys_write(
    fd: i32,
    buf: Vaddr,
    count: usize,
    current_process: &Arc<Process>,
) -> Result<SyscallReturn> {
    debug!(
        "[SYS_WRITE] Fd: {:?}, buf: {:x?}, count: {:?}",
        fd, buf, count
    );

    let mut buffer = vec![0u8; count];
    current_process
        .memory_space()
        .vm_space()
        .reader(buf, count)
        .unwrap()
        .read_fallible(&mut VmWriter::from(&mut buffer as &mut [u8]))
        .unwrap();

    early_print!("{}", str::from_utf8(&buffer).unwrap());

    Ok(SyscallReturn(count as _))
}
