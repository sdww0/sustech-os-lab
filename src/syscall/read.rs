use core::ascii;

use ostd::{
    early_print,
    mm::{Fallible, FallibleVmWrite, Vaddr, VmReader},
};

use super::SyscallReturn;
use crate::{console::receive_str, fs::STDIN, prelude::*, process::current_process, return_errno};

pub fn sys_read(fd: i32, user_buf_addr: Vaddr, buf_len: usize) -> Result<SyscallReturn> {
    debug!(
        "fd: {:?}, user_buf_addr: 0x{:x?}, buf_len: {:?}",
        fd, user_buf_addr, buf_len
    );

    if fd != STDIN as i32 || buf_len == 0 {
        return_errno!(Errno::ENOSYS)
    }

    let mut read_len = 0;

    let current_process = current_process().unwrap();
    let mut writer = current_process
        .memory_space()
        .vm_space()
        .writer(user_buf_addr, buf_len)
        .unwrap();

    let mut callback = |mut reader: VmReader<Fallible>| {
        if reader.remain() == 0 {
            return;
        }
        writer.write_fallible(&mut reader).unwrap();
    };

    while read_len == 0 {
        read_len = receive_str(&mut callback);
    }

    {
        let mut reader = current_process
            .memory_space()
            .vm_space()
            .reader(user_buf_addr, read_len)
            .unwrap();

        // Output the character, although we cannot use backspace and other special char :)
        while reader.has_remain() {
            if let Some(ascii_char) = ascii::Char::from_u8(reader.read_val::<u8>().unwrap()) {
                early_print!("{}", ascii_char);
            }
        }
    }

    Ok(SyscallReturn::Return(read_len as _))
}
