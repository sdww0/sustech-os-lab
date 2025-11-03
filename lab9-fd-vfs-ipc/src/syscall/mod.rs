mod brk;
mod clone;
mod exec;
mod exit;
mod prlimit;
mod read;
mod time;
mod uname;
mod wait4;
mod write;

use alloc::sync::Arc;
use log::{debug, info};
use ostd::arch::cpu::context::UserContext;
use ostd::arch::qemu::exit_qemu;
use ostd::task::Task;

use crate::error::{Errno, Error, Result};
use crate::process::Process;
use crate::syscall::brk::sys_brk;
use crate::syscall::clone::sys_clone;
use crate::syscall::exec::sys_execve;
use crate::syscall::exit::sys_exit;
use crate::syscall::prlimit::sys_prlimit64;
use crate::syscall::read::sys_read;
use crate::syscall::time::sys_clock_gettime;
use crate::syscall::uname::sys_uname;
use crate::syscall::wait4::sys_wait4;
use crate::syscall::write::{sys_write, sys_writev};

pub struct SyscallReturn(pub isize);

pub fn handle_syscall(user_context: &mut UserContext, current_process: &Arc<Process>) {
    const SYS_READ: usize = 63;
    const SYS_WRITE: usize = 64;
    const SYS_WRITEV: usize = 66;
    const SYS_EXIT: usize = 93;
    const SYS_EXIT_GROUP: usize = 94;

    const SYS_CLOCK_GETTIME: usize = 113;
    const SYS_SCHED_YIELD: usize = 124;
    const SYS_REBOOT: usize = 142;
    const SYS_NEWUNAME: usize = 160;
    const SYS_GETPID: usize = 172;
    const SYS_GETPPID: usize = 173;
    const SYS_BRK: usize = 214;
    const SYS_CLONE: usize = 220;
    const SYS_EXECVE: usize = 221;
    const SYS_MPROTECT: usize = 226;
    const SYS_WAIT4: usize = 260;
    const SYS_PRLIMIT64: usize = 261;

    let args = [
        user_context.a0(),
        user_context.a1(),
        user_context.a2(),
        user_context.a3(),
        user_context.a4(),
        user_context.a5(),
    ];

    info!(
        "[pid: {}] syscall num: {}, args: {:x?}",
        current_process.pid(),
        user_context.a7(),
        &args
    );

    let ret: Result<SyscallReturn> = match user_context.a7() {
        SYS_WRITEV => sys_writev(args[0] as _, args[1] as _, args[2] as _, current_process),
        SYS_NEWUNAME => sys_uname(args[0] as _, current_process),
        SYS_BRK => sys_brk(args[0] as _, current_process),
        SYS_MPROTECT => Ok(SyscallReturn(0)),
        SYS_GETPID => Ok(SyscallReturn(current_process.pid() as _)),
        SYS_GETPPID => {
            let ppid = current_process
                .parent_process()
                .and_then(|p| Some(p.pid()))
                .unwrap_or(0);
            Ok(SyscallReturn(ppid as _))
        }
        SYS_PRLIMIT64 => sys_prlimit64(
            args[0] as _,
            args[1] as _,
            args[2] as _,
            args[3] as _,
            current_process,
        ),
        SYS_CLONE => sys_clone(
            args[0] as _,
            args[1] as _,
            args[2] as _,
            args[3] as _,
            args[4] as _,
            current_process,
            user_context,
        ),

        SYS_EXECVE => sys_execve(
            args[0] as _,
            args[1] as _,
            args[2] as _,
            current_process,
            user_context,
        ),
        SYS_WAIT4 => sys_wait4(
            args[0] as _,
            args[1] as _,
            args[2] as _,
            args[3] as _,
            current_process,
        ),
        SYS_CLOCK_GETTIME => sys_clock_gettime(args[0] as _, args[1] as _, current_process),
        SYS_REBOOT => exit_qemu(ostd::arch::qemu::QemuExitCode::Success),
        SYS_READ => sys_read(args[0] as _, args[1] as _, args[2] as _, current_process),
        SYS_SCHED_YIELD => {
            Task::yield_now();
            Ok(SyscallReturn(0))
        }

        SYS_WRITE => sys_write(args[0] as _, args[1] as _, args[2] as _, current_process),
        SYS_EXIT | SYS_EXIT_GROUP => sys_exit(args[0] as _, current_process),
        _ => Err(Error::new(Errno::ENOSYS)),
    };

    match ret {
        Ok(value) => user_context.set_a0(value.0 as usize),
        Err(e) => {
            debug!(
                "[pid: {}] Syscall num: {}, return error: {:?}",
                current_process.pid(),
                user_context.a7(),
                e
            );
            user_context.set_a0(-(e.code()) as usize);
        }
    }
}
