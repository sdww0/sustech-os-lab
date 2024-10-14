use alloc::collections::btree_map::BTreeMap;
use spin::Once;

mod progs;

pub static USER_PROGS: Once<BTreeMap<&str, &[u8]>> = Once::new();

pub fn init() {
    progs::init();
}
