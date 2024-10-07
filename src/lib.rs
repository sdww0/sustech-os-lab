// SPDX-License-Identifier: MPL-2.0

#![no_std]
// The feature `linkage` is required for `ostd::main` to work.
#![feature(linkage)]

extern crate alloc;

use align_ext::AlignExt;
use core::str;

use alloc::sync::Arc;
use alloc::vec;

use ostd::arch::qemu::{exit_qemu, QemuExitCode};
use ostd::cpu::UserContext;
use ostd::mm::{
    CachePolicy, FallibleVmRead, FrameAllocOptions, PageFlags, PageProperty, Vaddr, VmIo, VmSpace,
    VmWriter, PAGE_SIZE,
};
use ostd::prelude::*;
use ostd::task::{Task, TaskOptions};
use ostd::user::{ReturnReason, UserContextApi, UserMode, UserSpace};

/// The kernel's boot and initialization process is managed by OSTD.
/// After the process is done, the kernel's execution environment
/// (e.g., stack, heap, tasks) will be ready for use and the entry function
/// labeled as `#[ostd::main]` will be called.
#[ostd::main]
pub fn main() {
    let program_binary = include_bytes!("../user_prog");
    let user_space = create_user_space(program_binary);
    let user_task = create_user_task(Arc::new(user_space));
    user_task.run();
}

fn create_user_space(program: &[u8]) -> UserSpace {
    let nframes = program.len().align_up(PAGE_SIZE) / PAGE_SIZE;
    let user_pages = {
        let vm_frames = FrameAllocOptions::new(nframes).alloc().unwrap();
        // Physical memory pages can be only accessed
        // via the Frame abstraction.
        vm_frames.write_bytes(0, program).unwrap();
        vm_frames
    };
    let user_address_space = {
        const MAP_ADDR: Vaddr = 0x0001_0000; // The map addr for statically-linked executable

        // The page table of the user space can be
        // created and manipulated safely through
        // the `VmSpace` abstraction.
        let vm_space = VmSpace::new();
        let mut cursor = vm_space
            .cursor_mut(&(MAP_ADDR..MAP_ADDR + nframes * PAGE_SIZE))
            .unwrap();
        let map_prop = PageProperty::new(PageFlags::RWX, CachePolicy::Writeback);
        for frame in user_pages {
            cursor.map(frame, map_prop);
        }
        drop(cursor);
        Arc::new(vm_space)
    };
    let user_cpu_state = {
        const ENTRY_POINT: Vaddr = 0x0001_00b0; // The entry point for statically-linked executable

        // The user-space CPU states can be initialized
        // to arbitrary values via the UserContext
        // abstraction.
        let mut user_cpu_state = UserContext::default();
        user_cpu_state.set_instruction_pointer(ENTRY_POINT);
        user_cpu_state
    };
    UserSpace::new(user_address_space, user_cpu_state)
}

fn create_user_task(user_space: Arc<UserSpace>) -> Arc<Task> {
    fn user_task() {
        let current = Task::current().unwrap();
        // Switching between user-kernel space is
        // performed via the UserMode abstraction.
        let mut user_mode = {
            let user_space = current.user_space().unwrap();
            UserMode::new(user_space)
        };

        loop {
            // The execute method returns when system
            // calls or CPU exceptions occur or some
            // events specified by the kernel occur.
            let return_reason = user_mode.execute(|| false);

            // The CPU registers of the user space
            // can be accessed and manipulated via
            // the `UserContext` abstraction.
            let user_context = user_mode.context_mut();
            match return_reason {
                ReturnReason::UserSyscall => {
                    handle_syscall(user_context, current.user_space().unwrap())
                }
                ReturnReason::UserException => {
                    handle_exception(user_context, current.user_space().unwrap())
                }
                ReturnReason::KernelEvent => {}
            }
        }
    }

    // Kernel tasks are managed by the Framework,
    // while scheduling algorithms for them can be
    // determined by the users of the Framework.
    Arc::new(
        TaskOptions::new(user_task)
            .user_space(Some(user_space))
            .data(0)
            .build()
            .unwrap(),
    )
}

fn handle_exception(user_context: &mut UserContext, _user_space: &UserSpace) {
    println!(
        "Catch CPU exception, skip this instruction. CPU exception: {:?}",
        user_context.trap_information().cpu_exception()
    );
    user_context.set_instruction_pointer(user_context.instruction_pointer() + 2);
}

fn handle_syscall(user_context: &mut UserContext, user_space: &UserSpace) {
    const SYS_DUMMY_CALL: usize = 1;
    const SYS_WRITE: usize = 64;
    const SYS_EXIT: usize = 93;

    match user_context.a7() {
        SYS_WRITE => {
            // Access the user-space CPU registers safely.
            let (_, buf_addr, buf_len) = (user_context.a0(), user_context.a1(), user_context.a2());
            let buf = {
                let mut buf = vec![0u8; buf_len];
                // Copy data from the user space without
                // unsafe pointer dereferencing.
                let current_vm_space = user_space.vm_space();
                let mut reader = current_vm_space.reader(buf_addr, buf_len).unwrap();
                reader
                    .read_fallible(&mut VmWriter::from(&mut buf as &mut [u8]))
                    .unwrap();
                buf
            };
            // Use the console for output safely.
            println!("{}", str::from_utf8(&buf).unwrap());
            // Manipulate the user-space CPU registers safely.
            user_context.set_a0(buf_len);
        }
        SYS_DUMMY_CALL => {
            let value = user_context.a0();
            println!("Value from userland program: 0x{:x}", value);
            user_context.set_a0(user_context.a1());
        }
        SYS_EXIT => {
            println!(
                "Exit from userland program, code: 0x{:x}",
                user_context.a0()
            );
            exit_qemu(QemuExitCode::Success)
        }
        val => {
            todo!("Unimplement syscall: {:?}", val);
        }
    }
}
