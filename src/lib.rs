// SPDX-License-Identifier: MPL-2.0

#![no_std]
// The feature `linkage` is required for `ostd::main` to work.
#![feature(linkage)]
#![feature(fn_traits)]
#![feature(ascii_char)]

use ostd::{arch::qemu::exit_qemu, early_println, task::Task};
use process::Process;

pub mod console;
pub mod context;
pub mod error;
pub mod fs;
pub mod prelude;
pub mod process;
pub mod syscall;
pub mod thread;
pub mod time;
pub mod util;
pub mod vm;

extern crate alloc;

#[ostd::main]
pub fn main() {
    fs::init();
    let init_thread = Process::new_kernel_process(init_thread, "idle".into());
    init_thread.run();
}

fn init_thread() {
    const INIT_PROCESS_NAME: &str = "hello_world";

    early_println!("User programs: {:?}", fs::USER_PROGS.get().unwrap().keys());
    let program_binary = fs::USER_PROGS
        .get()
        .unwrap()
        .get(INIT_PROCESS_NAME)
        .unwrap();
    let init_process = Process::new_user_process(program_binary, INIT_PROCESS_NAME.into());
    init_process.run();

    while !init_process.status().is_zombie() {
        Task::yield_now();
    }

    early_println!(
        "Init process exit with exit code: {:?}",
        init_process.status().exit_code()
    );

    if init_process.status().exit_code() != 0 {
        exit_qemu(ostd::arch::qemu::QemuExitCode::Failed)
    } else {
        exit_qemu(ostd::arch::qemu::QemuExitCode::Success)
    }
}
