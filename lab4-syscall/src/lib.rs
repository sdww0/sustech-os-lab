#![no_std]
#![deny(unsafe_code)]

pub mod process;
pub mod syscall;

extern crate alloc;

#[ostd::main]
pub fn main() {
    // let program_binary = include_bytes!("../target/ebreak");
    let program_binary = include_bytes!("../target/hello");
    let process = process::Process::new(program_binary);
    process.run();
}
