use alloc::{collections::btree_map::BTreeMap, string::String};
use spin::Once;

use crate::prelude::*;

mod progs;

pub static USER_PROGS: Once<BTreeMap<&str, &'static [u8]>> = Once::new();

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
