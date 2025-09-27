use core::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

use align_ext::AlignExt;
use alloc::sync::Arc;
use ostd::arch::cpu::context::UserContext;
use ostd::arch::qemu::{QemuExitCode, exit_qemu};
use ostd::early_println;
use ostd::mm::{
    CachePolicy, FrameAllocOptions, PAGE_SIZE, PageFlags, PageProperty, Vaddr, VmIo, VmSpace,
};
use ostd::task::{Task, TaskOptions, disable_preempt};
use ostd::user::{ReturnReason, UserContextApi, UserMode};
use riscv::register::scause::Exception;
use spin::Once;

use crate::process;

pub struct Process {
    // ======================== Basic info of process ===============================
    /// The id of this process.
    pid: Pid,
    /// Process state
    status: AtomicU64,
    /// The threads of this process
    task: Once<Arc<Task>>,
    // TODO-1.1: Add the name of the process, use the name passed in from `Process::new()`.

    // ======================== Memory management ===============================
    vm_space: Arc<VmSpace>,
}

impl Process {
    pub fn new(user_prog_bin: &[u8]) -> Arc<Self> {
        let vm_space = Arc::new(create_vm_space(user_prog_bin));
        vm_space.activate();

        let process = Arc::new(Process {
            pid: alloc_pid(),
            status: AtomicU64::new(Status::Uninit as u64),
            task: Once::new(),
            vm_space,
        });

        let task = create_user_task(&process);

        process.task.call_once(|| task);
        process
            .status
            .store(Status::Runnable as u64, Ordering::SeqCst);

        process
    }

    pub fn run(&self) {
        self.task.get().unwrap().run();
    }

    pub fn vm_space(&self) -> &Arc<VmSpace> {
        &self.vm_space
    }
}

#[repr(u64)]
enum Status {
    Uninit = 0,
    Runnable = 1,
    Zombie = 2,
}

fn create_user_task(process: &Arc<Process>) -> Arc<Task> {
    fn user_task() {
        let process = current_process();

        let mut user_mode = {
            let user_ctx = create_user_context();
            UserMode::new(user_ctx)
        };

        loop {
            let return_reason = user_mode.execute(|| false);
            let user_context = user_mode.context_mut();
            match return_reason {
                ReturnReason::UserException => {
                    // Exit on any user exception
                    let exception = user_context.take_exception().unwrap();
                    early_println!(
                        "Process {} killed by exception: {:#x?}",
                        process.pid,
                        exception
                    );
                    exit_qemu(QemuExitCode::Success);
                }
                ReturnReason::UserSyscall => {
                    crate::syscall::handle_syscall(user_context, &process);
                    // TODO-2.2: Check if the process is Zombie, if so, call the QEMU exit function here.
                }
                ReturnReason::KernelEvent => unreachable!(),
            }
        }
    }

    Arc::new(
        TaskOptions::new(user_task)
            .data(process.clone())
            .build()
            .unwrap(),
    )
}

fn create_user_context() -> UserContext {
    let mut user_ctx = UserContext::default();
    const ENTRY_POINT: Vaddr = 0x10078;
    user_ctx.set_instruction_pointer(ENTRY_POINT);
    user_ctx
}

fn create_vm_space(program: &[u8]) -> VmSpace {
    let nbytes = program.len().align_up(PAGE_SIZE);

    // Allocate some physical pages for the user program
    let user_pages = FrameAllocOptions::new()
        .zeroed(true)
        .alloc_segment(nbytes / PAGE_SIZE)
        .unwrap();
    user_pages.write_bytes(0, program).unwrap();

    // Map the user pages to a fixed virtual address
    let vm_space = VmSpace::new();
    const MAP_ADDR: Vaddr = 0x0001_0000;

    let preempt_guard = disable_preempt();
    let mut cursor = vm_space
        .cursor_mut(&preempt_guard, &(MAP_ADDR..MAP_ADDR + nbytes))
        .unwrap();

    let map_prop = PageProperty::new_user(PageFlags::RWX, CachePolicy::Writeback);
    for frame in user_pages {
        cursor.map(frame.into(), map_prop);
    }

    drop(cursor);
    vm_space
}

type Pid = usize;

fn alloc_pid() -> Pid {
    static NEXT_PID: AtomicUsize = AtomicUsize::new(1);
    NEXT_PID.fetch_add(1, Ordering::Relaxed)
}

fn current_process() -> Arc<Process> {
    let current = Task::current().unwrap();
    current
        .data()
        .downcast_ref::<Arc<Process>>()
        .unwrap()
        .clone()
}
