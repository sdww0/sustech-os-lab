const EXEC: &[u8] =
    include_bytes_aligned::include_bytes_aligned!(32, "../../target/user_prog/exec");
const FORK: &[u8] =
    include_bytes_aligned::include_bytes_aligned!(32, "../../target/user_prog/fork");
const HELLO_WORLD: &[u8] =
    include_bytes_aligned::include_bytes_aligned!(32, "../../target/user_prog/hello_world");
const INIT_PROC: &[u8] =
    include_bytes_aligned::include_bytes_aligned!(32, "../../target/user_prog/init_proc");
const READ_STDIN: &[u8] =
    include_bytes_aligned::include_bytes_aligned!(32, "../../target/user_prog/read_stdin");
const REPARENT: &[u8] =
    include_bytes_aligned::include_bytes_aligned!(32, "../../target/user_prog/reparent");
const SHELL: &[u8] =
    include_bytes_aligned::include_bytes_aligned!(32, "../../target/user_prog/shell");
const WAIT: &[u8] =
    include_bytes_aligned::include_bytes_aligned!(32, "../../target/user_prog/wait");

pub fn init() {
    super::USER_PROGS.call_once(|| {
        let mut user_progs = alloc::collections::btree_map::BTreeMap::new();
        user_progs.insert("exec", EXEC);
        user_progs.insert("fork", FORK);
        user_progs.insert("hello_world", HELLO_WORLD);
        user_progs.insert("init_proc", INIT_PROC);
        user_progs.insert("read_stdin", READ_STDIN);
        user_progs.insert("reparent", REPARENT);
        user_progs.insert("shell", SHELL);
        user_progs.insert("wait", WAIT);
        user_progs
    });
}
