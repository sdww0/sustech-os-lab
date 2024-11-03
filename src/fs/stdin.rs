use core::ascii;

use ostd::{
    early_print,
    mm::{Fallible, Vaddr, VmReader},
};

use crate::prelude::*;

use crate::{console::receive_str, process::current_process};

pub fn read(user_buf_addr: Vaddr, buf_len: usize) -> Result<usize> {
    let current_process = current_process().unwrap();
    let mut writer = current_process
        .memory_space()
        .vm_space()
        .writer(user_buf_addr, buf_len)?;

    let mut read_len = 0;
    let mut need_return = false;

    while !need_return {
        let mut callback = |mut reader: VmReader<Fallible>| -> Result<()> {
            if reader.remain() == 0 {
                return Ok(());
            }

            while reader.has_remain() {
                if let Some(ascii_char) = ascii::Char::from_u8(reader.read_val::<u8>().unwrap()) {
                    read_len += 1;
                    // Return.
                    if ascii_char.to_u8() == 13 {
                        need_return = true;
                        // We convert "Return" to "New Line" (Ascii 10)
                        writer.write_val::<u8>(&10)?;
                        return Ok(());
                    }
                    // Output the character, although we cannot use backspace and other special char :)
                    early_print!("{}", ascii_char);
                    writer.write_val(&ascii_char.to_u8())?;
                }
            }
            Ok(())
        };

        receive_str(&mut callback)?;
    }

    Ok(read_len)
}
