const FORK: &[u8] =
    include_bytes_aligned::include_bytes_aligned!(32, "../../target/user_prog/fork");
const HELLO_WORLD: &[u8] =
    include_bytes_aligned::include_bytes_aligned!(32, "../../target/user_prog/hello_world");

pub fn init() {
    super::USER_PROGS.call_once(|| {
        let mut user_progs = alloc::collections::btree_map::BTreeMap::new();
        user_progs.insert("fork", FORK);
        user_progs.insert("hello_world", HELLO_WORLD);
        user_progs
    });
}
