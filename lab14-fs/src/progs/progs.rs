const EXEC: &[u8] =
    include_bytes_aligned::include_bytes_aligned!(32, "../../target/user_prog/exec");
const FORK: &[u8] =
    include_bytes_aligned::include_bytes_aligned!(32, "../../target/user_prog/fork");
const FORK_EXEC_TIME: &[u8] =
    include_bytes_aligned::include_bytes_aligned!(32, "../../target/user_prog/fork_exec_time");
const FORK_TIME: &[u8] =
    include_bytes_aligned::include_bytes_aligned!(32, "../../target/user_prog/fork_time");
const FORK_TIME_LOOP: &[u8] =
    include_bytes_aligned::include_bytes_aligned!(32, "../../target/user_prog/fork_time_loop");
const HELLO_WORLD: &[u8] =
    include_bytes_aligned::include_bytes_aligned!(32, "../../target/user_prog/hello_world");
const INIT_PROC: &[u8] =
    include_bytes_aligned::include_bytes_aligned!(32, "../../target/user_prog/init_proc");
const MMAP_ANON_TEST: &[u8] =
    include_bytes_aligned::include_bytes_aligned!(32, "../../target/user_prog/mmap_anon_test");
const MMAP_TEST: &[u8] =
    include_bytes_aligned::include_bytes_aligned!(32, "../../target/user_prog/mmap_test");
const PIPE: &[u8] =
    include_bytes_aligned::include_bytes_aligned!(32, "../../target/user_prog/pipe");
const RAMFS: &[u8] =
    include_bytes_aligned::include_bytes_aligned!(32, "../../target/user_prog/ramfs");
const READ_STDIN: &[u8] =
    include_bytes_aligned::include_bytes_aligned!(32, "../../target/user_prog/read_stdin");
const REPARENT: &[u8] =
    include_bytes_aligned::include_bytes_aligned!(32, "../../target/user_prog/reparent");
const RR_TEST: &[u8] =
    include_bytes_aligned::include_bytes_aligned!(32, "../../target/user_prog/rr_test");
const SHELL: &[u8] =
    include_bytes_aligned::include_bytes_aligned!(32, "../../target/user_prog/shell");
const WAIT: &[u8] =
    include_bytes_aligned::include_bytes_aligned!(32, "../../target/user_prog/wait");

pub fn init() {
    super::USER_PROGS.call_once(|| {
        let mut user_progs = alloc::collections::btree_map::BTreeMap::new();
        user_progs.insert("exec", EXEC);
        user_progs.insert("fork", FORK);
        user_progs.insert("fork_exec_time", FORK_EXEC_TIME);
        user_progs.insert("fork_time", FORK_TIME);
        user_progs.insert("fork_time_loop", FORK_TIME_LOOP);
        user_progs.insert("hello_world", HELLO_WORLD);
        user_progs.insert("init_proc", INIT_PROC);
        user_progs.insert("mmap_anon_test", MMAP_ANON_TEST);
        user_progs.insert("mmap_test", MMAP_TEST);
        user_progs.insert("pipe", PIPE);
        user_progs.insert("ramfs", RAMFS);
        user_progs.insert("read_stdin", READ_STDIN);
        user_progs.insert("reparent", REPARENT);
        user_progs.insert("rr_test", RR_TEST);
        user_progs.insert("shell", SHELL);
        user_progs.insert("wait", WAIT);
        user_progs
    });
}
