#![no_std]
#![deny(unsafe_code)]
#![feature(fn_traits)]
#![feature(ascii_char)]

pub mod console;
mod error;
mod logger;
mod mm;
pub mod process;
pub mod progs;
pub mod syscall;

extern crate alloc;

#[ostd::main]
pub fn main() {
    logger::init();
    progs::init();
    let process = process::Process::new(progs::lookup_progs("exec").unwrap());
    process.run();
}
