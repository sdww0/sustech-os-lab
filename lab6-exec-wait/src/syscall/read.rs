use core::ascii;

use alloc::sync::Arc;
use log::debug;
use ostd::{
    early_print,
    mm::{Fallible, Vaddr, VmReader},
};

use super::SyscallReturn;
use crate::error::Result;
use crate::{
    error::{Errno, Error},
    process::Process,
};

use crate::console::receive_str;

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

    let mut writer = current_process
        .memory_space()
        .vm_space()
        .writer(user_buf_addr, buf_len)
        .unwrap();

    let mut read_len = 0;
    let mut need_return = false;

    while !need_return {
        let mut callback = |mut reader: VmReader<Fallible>| {
            while reader.has_remain() {
                if let Some(ascii_char) = ascii::Char::from_u8(reader.read_val::<u8>().unwrap()) {
                    read_len += 1;
                    // Return.
                    if ascii_char.to_u8() == 13 {
                        need_return = true;
                        // We convert "Return" to "New Line" (Ascii 10)
                        writer.write_val::<u8>(&10).unwrap();
                    }
                    // Output the character, although we cannot use backspace and other special char :)
                    early_print!("{}", ascii_char);
                    writer.write_val(&ascii_char.to_u8()).unwrap();
                }
            }
        };

        receive_str(&mut callback);
    }
    Ok(SyscallReturn(read_len as _))
}
