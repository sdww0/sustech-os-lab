mod elf;
mod heap;
mod status;

use core::sync::atomic::{AtomicUsize, Ordering};

use alloc::boxed::Box;
use alloc::collections::btree_map::BTreeMap;
use alloc::sync::{Arc, Weak};
use log::info;
use ostd::arch::cpu::context::UserContext;
use ostd::arch::qemu::{QemuExitCode, exit_qemu};
use ostd::early_println;
use ostd::sync::Mutex;
use ostd::task::{Task, TaskOptions};
use ostd::user::{ReturnReason, UserContextApi, UserMode};
use riscv::register::scause::Exception;
use spin::Once;

use crate::mm::MemorySpace;
use crate::process::heap::UserHeap;
use crate::process::status::ProcessStatus;
pub const USER_STACK_SIZE: usize = 8192 * 1024; // 8MB

#[inline]
pub fn current_process() -> Arc<Process> {
    let current = Task::current().unwrap();
    current
        .data()
        .downcast_ref::<Arc<Process>>()
        .unwrap()
        .clone()
}

pub struct Process {
    // ======================== Basic info of process ===========================
    /// The id of this process.
    pid: Pid,
    /// Process state
    status: ProcessStatus,
    /// The thread of this process
    task: Once<Arc<Task>>,

    // ======================== Memory management ===============================
    memory_space: MemorySpace,
    // Heap
    heap: UserHeap,

    // ======================== Process-tree fields =================================
    /// Parent process.
    parent_process: Mutex<Weak<Process>>,
    /// Children process.
    children: Mutex<BTreeMap<Pid, Arc<Process>>>,
}

impl Process {
    pub fn new(user_prog_bin: &[u8]) -> Arc<Self> {
        let (memory_space, user_context) = elf::create_user_space(user_prog_bin);

        let process = Arc::new(Process {
            pid: alloc_pid(),
            status: ProcessStatus::new(),
            task: Once::new(),
            memory_space,
            heap: UserHeap::new(),
            parent_process: Mutex::new(Weak::new()),
            children: Mutex::new(BTreeMap::new()),
        });

        let task = create_user_task(&process, Box::new(user_context));
        process.task.call_once(|| task);
        process.status.set_runnable();

        process
    }

    pub fn parent_process(&self) -> Option<Arc<Process>> {
        self.parent_process.lock().upgrade()
    }

    pub fn exit(&self, exit_code: u32) {
        self.status.exit(exit_code);
    }

    pub fn is_zombie(&self) -> bool {
        self.status.is_zombie()
    }

    pub fn exit_code(&self) -> Option<u32> {
        self.status.exit_code()
    }

    pub fn pid(&self) -> Pid {
        self.pid
    }

    pub fn run(&self) {
        self.task.get().unwrap().run();
    }

    pub fn memory_space(&self) -> &MemorySpace {
        &self.memory_space
    }

    pub fn heap(&self) -> &UserHeap {
        &self.heap
    }
}

fn create_user_task(process: &Arc<Process>, user_context: Box<UserContext>) -> Arc<Task> {
    let entry = move |user_ctx| {
        let process = current_process();

        let mut user_mode = UserMode::new(user_ctx);
        let vm_space = process.memory_space().vm_space();

        loop {
            vm_space.activate();
            let return_reason = user_mode.execute(|| false);
            let user_context = user_mode.context_mut();
            match return_reason {
                ReturnReason::UserException => {
                    let exception = user_context.take_exception().unwrap();
                    if exception.cpu_exception() == Exception::IllegalInstruction {
                        // The illegal instructions can include the floating point instructions
                        // if the FPU is not enabled. Here we just skip it.
                        user_context
                            .set_instruction_pointer(user_context.instruction_pointer() + 2);
                    } else {
                        early_println!(
                            "Process {} killed by exception: {:#x?}   at instruction {:#x}",
                            process.pid,
                            exception,
                            user_context.instruction_pointer()
                        );
                        exit_qemu(QemuExitCode::Success);
                    }
                }
                ReturnReason::UserSyscall => {
                    crate::syscall::handle_syscall(user_context, &process);
                }
                ReturnReason::KernelEvent => unreachable!(),
            }
            if let Some(exit_code) = process.exit_code() {
                info!("Process {} exited with code {}", process.pid(), exit_code);
                break;
            }
        }
    };

    let user_task_func = move || entry(*user_context);

    Arc::new(
        TaskOptions::new(user_task_func)
            .data(process.clone())
            .build()
            .unwrap(),
    )
}

type Pid = usize;

fn alloc_pid() -> Pid {
    static NEXT_PID: AtomicUsize = AtomicUsize::new(1);
    NEXT_PID.fetch_add(1, Ordering::Relaxed)
}
