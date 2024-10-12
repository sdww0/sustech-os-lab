// SPDX-License-Identifier: MPL-2.0

#![no_std]
// The feature `linkage` is required for `ostd::main` to work.
#![feature(linkage)]

use process::Process;

pub mod error;
pub mod mm;
pub mod prelude;
pub mod process;
pub mod syscall;
pub mod thread;
pub mod util;

extern crate alloc;

/// The kernel's boot and initialization process is managed by OSTD.
/// After the process is done, the kernel's execution environment
/// (e.g., stack, heap, tasks) will be ready for use and the entry function
/// labeled as `#[ostd::main]` will be called.
#[ostd::main]
pub fn main() {
    let program_binary = include_bytes!("../user_prog");
    let process = Process::new_user_process(program_binary);
    process.run();
}
