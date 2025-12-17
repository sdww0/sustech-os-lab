use core::ffi::CStr;

use alloc::string::{String, ToString};
use ostd::Pod;

const MAX_NAME_LEN: usize = 256;

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod)]
pub struct Ext2DirEntry {
    ino: u32,
    record_len: u16,
    name_len: u8,
    type_: u8,
    name: [u8; MAX_NAME_LEN],
}

impl Ext2DirEntry {
    pub fn inode(&self) -> u32 {
        self.ino
    }

    pub fn length(&self) -> u16 {
        self.record_len
    }

    pub fn name_length(&self) -> u8 {
        self.name_len
    }

    pub fn name(&self) -> String {
        CStr::from_bytes_until_nul(&self.name)
            .unwrap()
            .to_string_lossy()
            .to_string()
    }
}

impl Default for Ext2DirEntry {
    fn default() -> Self {
        Self {
            ino: Default::default(),
            record_len: Default::default(),
            name_len: Default::default(),
            name: [0; MAX_NAME_LEN],
            type_: 0,
        }
    }
}
