pub mod heap;
#[allow(clippy::module_inception)]
pub mod process;
pub mod signal;
pub mod status;

pub type Pid = u32;
use alloc::{collections::btree_map::BTreeMap, sync::Arc};
use ostd::mm::PAGE_SIZE;
pub use process::{current_process, Process};
use spin::Mutex;

pub const USER_STACK_SIZE: usize = 32 * PAGE_SIZE;

pub static PROCESS_TABLE: Mutex<BTreeMap<Pid, Arc<Process>>> = Mutex::new(BTreeMap::new());
