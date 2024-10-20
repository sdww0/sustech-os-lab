// SPDX-License-Identifier: MPL-2.0
// Copy from asterinas: kernel/src/context.rs

use core::mem;

use alloc::{ffi::CString, vec::Vec};
use ostd::mm::{Fallible, VmReader};

use crate::{prelude::*, return_errno_with_message};

/// A trait providing the ability to read a C string from the user space.
///
/// The user space should be of the current process. The implemented method
/// should read the bytes iteratively in the reader ([`VmReader`]) until
/// encountering the end of the reader or reading a `\0` (which is also
/// included in the final C String).
pub trait ReadCString {
    fn read_cstring(&mut self) -> Result<CString>;
}

impl ReadCString for VmReader<'_, Fallible> {
    /// Reads a C string from the user space.
    ///
    /// This implementation is inspired by
    /// the `do_strncpy_from_user` function in Linux kernel.
    /// The original Linux implementation can be found at:
    /// <https://elixir.bootlin.com/linux/v6.0.9/source/lib/strncpy_from_user.c#L28>
    fn read_cstring(&mut self) -> Result<CString> {
        let max_len = self.remain();
        let mut buffer: Vec<u8> = Vec::with_capacity(max_len);

        macro_rules! read_one_byte_at_a_time_while {
            ($cond:expr) => {
                while $cond {
                    let byte = self.read_val::<u8>()?;
                    buffer.push(byte);
                    if byte == 0 {
                        return Ok(CString::from_vec_with_nul(buffer)
                            .expect("We provided 0 but no 0 is found"));
                    }
                }
            };
        }

        // Handle the first few bytes to make `cur_addr` aligned with `size_of::<usize>`
        read_one_byte_at_a_time_while!(
            !is_addr_aligned(self.cursor() as usize) && buffer.len() < max_len
        );

        // Handle the rest of the bytes in bulk
        while (buffer.len() + mem::size_of::<usize>()) <= max_len {
            let Ok(word) = self.read_val::<usize>() else {
                break;
            };

            if has_zero(word) {
                for byte in word.to_ne_bytes() {
                    buffer.push(byte);
                    if byte == 0 {
                        return Ok(CString::from_vec_with_nul(buffer)
                            .expect("We provided 0 but no 0 is found"));
                    }
                }
                unreachable!("The branch should never be reached unless `has_zero` has bugs.")
            }

            buffer.extend_from_slice(&word.to_ne_bytes());
        }

        // Handle the last few bytes that are not enough for a word
        read_one_byte_at_a_time_while!(buffer.len() < max_len);

        // Maximum length exceeded before finding the null terminator
        return_errno_with_message!(Errno::EFAULT, "Fails to read CString from user");
    }
}

/// Determines whether the value contains a zero byte.
///
/// This magic algorithm is from the Linux `has_zero` function:
/// <https://elixir.bootlin.com/linux/v6.0.9/source/include/asm-generic/word-at-a-time.h#L93>
const fn has_zero(value: usize) -> bool {
    const ONE_BITS: usize = usize::from_le_bytes([0x01; mem::size_of::<usize>()]);
    const HIGH_BITS: usize = usize::from_le_bytes([0x80; mem::size_of::<usize>()]);

    value.wrapping_sub(ONE_BITS) & !value & HIGH_BITS != 0
}

/// Checks if the given address is aligned.
const fn is_addr_aligned(addr: usize) -> bool {
    (addr & (mem::size_of::<usize>() - 1)) == 0
}
