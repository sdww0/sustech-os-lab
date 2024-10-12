// SPDX-License-Identifier: MPL-2.0

pub mod mprotect;
pub mod write;

use core::str;

use alloc::vec;
use log::debug;
use mprotect::sys_mprotect;
use ostd::{
    arch::qemu::{exit_qemu, QemuExitCode},
    cpu::UserContext,
    early_print as print, early_println as println,
    mm::{FallibleVmRead, VmWriter},
    user::UserSpace,
};
use write::sys_writev;

use crate::{
    error::{Errno, Error},
    prelude::*,
    process::current_process,
    thread::current_thread,
};

#[allow(dead_code)]
pub fn handle_syscall(user_context: &mut UserContext, user_space: &UserSpace) {
    const SYS_WRITE: usize = 64;
    const SYS_READV: usize = 65;
    const SYS_WRITEV: usize = 66;
    const SYS_READLINKAT: usize = 78;
    const SYS_NEWFSTAT: usize = 80;
    const SYS_EXIT: usize = 93;
    const SYS_EXIT_GROUP: usize = 94;
    const SYS_SET_TID_ADDRESS: usize = 96;
    const SYS_SET_ROUBST_LIST: usize = 99;
    const SYS_BRK: usize = 214;
    const SYS_CLONE: usize = 220;
    const SYS_EXECVE: usize = 221;
    const SYS_MPROTECT: usize = 226;
    const SYS_HWPROBE: usize = 258;
    const SYS_WAIT4: usize = 260;
    const SYS_PRLIMIT64: usize = 261;
    const SYS_GETRANDOM: usize = 278;

    let args = [
        user_context.a0(),
        user_context.a1(),
        user_context.a2(),
        user_context.a3(),
        user_context.a4(),
        user_context.a5(),
    ];

    debug!("Syscall:{:?}, args:{:#x?}", user_context.a7(), args);

    let ret: Result<SyscallReturn> = match user_context.a7() {
        SYS_WRITE => {
            let (_, addr, len) = (args[0], args[1], args[2]);
            let mut buf = vec![0u8; len];

            let current_vm_space = user_space.vm_space();
            let mut reader = current_vm_space.reader(addr, len).unwrap();
            reader
                .read_fallible(&mut VmWriter::from(&mut buf as &mut [u8]))
                .unwrap();

            print!("{}", str::from_utf8(&buf).unwrap());
            Ok(SyscallReturn::Return(len as isize))
        }
        SYS_SET_TID_ADDRESS => {
            let current_thread = current_thread();
            // FIXME: We should use clone flags to determine which one to use.
            let mut clear_child_tid = current_thread.clear_child_tid();
            if *clear_child_tid == 0 {
                *clear_child_tid = args[0];
            }
            Ok(SyscallReturn::Return(current_thread.tid() as isize))
        }
        SYS_BRK => {
            let process = current_process().unwrap();
            let val = if args[0] == 0 { None } else { Some(args[0]) };
            Ok(SyscallReturn::Return(
                process.heap.brk(val).unwrap() as isize
            ))
        }
        SYS_EXIT | SYS_EXIT_GROUP => {
            println!("Exit from userland program, code: 0x{:x}", args[0]);
            exit_qemu(QemuExitCode::Success)
        }
        SYS_HWPROBE => Err(Error::new(Errno::ENOSYS)),
        SYS_PRLIMIT64 => Err(Error::new(Errno::ENOSYS)),
        SYS_READLINKAT => Err(Error::new(Errno::ENOSYS)),
        SYS_NEWFSTAT => Err(Error::new(Errno::ENOSYS)),
        SYS_SET_ROUBST_LIST => Err(Error::new(Errno::ENOSYS)),
        SYS_MPROTECT => sys_mprotect(args[0], args[1], args[2] as _),
        SYS_WRITEV => sys_writev(args[0] as _, args[1], args[2]),
        SYS_GETRANDOM => {
            // TODO
            Ok(SyscallReturn::Return(args[1] as isize))
        }
        val => {
            todo!("Unimplement syscall: {:?}", val);
            // Err(Error::new(Errno::ENOSYS))
        }
    };

    let val: isize = match ret {
        Ok(val) => match val {
            SyscallReturn::Return(val) => val,
            SyscallReturn::NoReturn => todo!(),
        },
        Err(err) => -(err.error() as i32 as isize),
    };

    user_context.set_a0(val as usize);
}

/// Syscall return
#[derive(Debug, Clone, Copy)]
pub enum SyscallReturn {
    /// return isize, this value will be used to set rax
    Return(isize),
    /// does not need to set rax
    NoReturn,
}
