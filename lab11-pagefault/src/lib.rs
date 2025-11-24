#![no_std]
#![deny(unsafe_code)]
#![feature(fn_traits)]
#![feature(ascii_char)]

pub mod console;
mod error;
mod fs;
mod logger;
mod mm;
pub mod process;
pub mod progs;
mod sched;
pub mod syscall;

extern crate alloc;

#[ostd::main]
pub fn main() {
    logger::init();
    progs::init();
    sched::init();
    fs::init();
    mm::init();

    let process = process::Process::new(progs::lookup_progs("init_proc").unwrap());
    process.run();
}
