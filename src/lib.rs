// SPDX-License-Identifier: MPL-2.0

#![no_std]
// The feature `linkage` is required for `ostd::main` to work.
#![feature(linkage)]

use ostd::early_println;
use process::Process;

pub mod error;
pub mod fs;
pub mod prelude;
pub mod process;
pub mod syscall;
pub mod thread;
pub mod util;
pub mod vm;

extern crate alloc;

/// The kernel's boot and initialization process is managed by OSTD.
/// After the process is done, the kernel's execution environment
/// (e.g., stack, heap, tasks) will be ready for use and the entry function
/// labeled as `#[ostd::main]` will be called.
#[ostd::main]
pub fn main() {
    fs::init();
    early_println!("User_prog:{:?}", fs::USER_PROGS.get().unwrap().keys());
    let program_binary = fs::USER_PROGS.get().unwrap().get("fork").unwrap();
    let process = Process::new_user_process(program_binary);
    process.run();
}
