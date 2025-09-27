use alloc::sync::Arc;
use alloc::vec;
use ostd::arch::cpu::context::UserContext;
use ostd::arch::qemu::{QemuExitCode, exit_qemu};
use ostd::mm::{FallibleVmRead, VmSpace, VmWriter};
use ostd::prelude::*;

use core::str;

use crate::process::Process;

pub fn handle_syscall(user_context: &mut UserContext, process: &Arc<Process>) {
    // TODO-1.2: Define a new syscall number for printing the name and the pid of the process.
    const SYS_WRITE: usize = 64;
    const SYS_EXIT: usize = 93;

    match user_context.a7() {
        SYS_WRITE => {
            // Read buffer from user space
            let (_, buf_addr, buf_len) = (user_context.a0(), user_context.a1(), user_context.a2());
            let buf = {
                let mut buf = vec![0u8; buf_len];
                let mut reader = process.vm_space().reader(buf_addr, buf_len).unwrap();
                reader
                    .read_fallible(&mut VmWriter::from(&mut buf as &mut [u8]))
                    .unwrap();
                buf
            };

            // Write to stdout
            println!("{}", str::from_utf8(&buf).unwrap());

            user_context.set_a0(buf_len);
        }
        SYS_EXIT => {
            // TODO-2.1: If the process call SYS_EXIT, the handle_syscall will only set the process state to Zombie. DO NOT call the QEMU exit function here.
            exit_qemu(QemuExitCode::Success);
        }
        _ => unimplemented!(),
    }
}
