use crate::error::{Errno, Error, Result};
use alloc::collections::btree_map::BTreeMap;
use spin::Once;

mod progs;

pub static USER_PROGS: Once<BTreeMap<&str, &'static [u8]>> = Once::new();

pub fn init() {
    progs::init();
}

pub fn lookup_progs(prog_name: &str) -> Result<&'static [u8]> {
    USER_PROGS
        .get()
        .unwrap()
        .get(prog_name)
        .ok_or(Error::new(Errno::ENOENT))
        .copied()
}
