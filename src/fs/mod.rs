use alloc::{collections::btree_map::BTreeMap, string::String};
use spin::Once;

use crate::prelude::*;

mod progs;
pub mod stdin;

pub static USER_PROGS: Once<BTreeMap<&str, &'static [u8]>> = Once::new();

pub const STDIN: usize = 0;
pub const STDOUT: usize = 1;
pub const STDERR: usize = 2;

pub fn init() {
    progs::init();
}

pub fn lookup_file(file_name: String) -> Result<&'static [u8]> {
    USER_PROGS
        .get()
        .unwrap()
        .get(file_name.as_str())
        .ok_or(Error::new(Errno::ENOENT))
        .copied()
}
