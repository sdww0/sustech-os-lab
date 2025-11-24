use alloc::vec;
use ostd::{
    early_print,
    mm::{Fallible, FallibleVmRead, VmReader, VmWriter},
};

use crate::{
    console::receive_str,
    error::{Errno, Error, Result},
};
use core::str;

pub trait FileLike: Sync + Send {
    fn read(&self, writer: VmWriter) -> Result<usize>;
    fn write(&self, reader: VmReader) -> Result<usize>;
}

pub struct Stdin;

impl FileLike for Stdin {
    fn read(&self, mut buf: VmWriter) -> Result<usize> {
        let mut read_len = 0;
        let mut need_return = false;

        while !need_return {
            let mut callback = |mut reader: VmReader<Fallible>| {
                while reader.has_remain() {
                    if let Some(ascii_char) =
                        core::ascii::Char::from_u8(reader.read_val::<u8>().unwrap())
                    {
                        read_len += 1;
                        // Return.
                        if ascii_char.to_u8() == 13 {
                            need_return = true;
                            // We convert "Return" to "New Line" (Ascii 10)
                            buf.write_val::<u8>(&10).unwrap();
                        }
                        // Output the character, although we cannot use backspace and other special char :)
                        early_print!("{}", ascii_char);
                        buf.write_val(&ascii_char.to_u8()).unwrap();
                    }
                }
            };

            receive_str(&mut callback);
        }
        Ok(read_len)
    }

    fn write(&self, _buf: VmReader) -> Result<usize> {
        Err(Error::new(Errno::ENOSYS))
    }
}

pub struct Stdout;

impl FileLike for Stdout {
    fn read(&self, _buf: VmWriter) -> Result<usize> {
        Err(Error::new(Errno::ENOSYS))
    }

    fn write(&self, mut buf: VmReader) -> Result<usize> {
        let mut buffer = vec![0u8; buf.remain()];
        buf.read_fallible(&mut VmWriter::from(&mut buffer as &mut [u8]))
            .unwrap();

        early_print!("{}", str::from_utf8(&buffer).unwrap());

        Ok(buffer.len())
    }
}

pub struct Stderr;

impl FileLike for Stderr {
    fn read(&self, _buf: VmWriter) -> Result<usize> {
        Err(Error::new(Errno::ENOSYS))
    }

    fn write(&self, mut buf: VmReader) -> Result<usize> {
        let mut buffer = vec![0u8; buf.remain()];
        buf.read_fallible(&mut VmWriter::from(&mut buffer as &mut [u8]))
            .unwrap();

        early_print!("{}", str::from_utf8(&buffer).unwrap());

        Ok(buffer.len())
    }
}
