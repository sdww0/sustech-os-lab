use core::sync::atomic::{AtomicU32, Ordering};

use ostd::sync::{Mutex, MutexGuard};
use ostd::task::{Task, TaskOptions};

use alloc::sync::{Arc, Weak};

use ostd::cpu::UserContext;

use ostd::prelude::*;
use ostd::user::{ReturnReason, UserContextApi, UserMode, UserSpace};
use spin::Once;

use crate::process::Process;

pub type Tid = u32;

pub struct Thread {
    tid: Tid,
    task: Once<Arc<Task>>,
    process: Weak<Process>,

    set_child_tid: Mutex<Vaddr>,
    clear_child_tid: Mutex<Vaddr>,
}

impl Thread {
    pub fn new_kernel_thread<F>(func: F, process: Weak<Process>, tid: Tid) -> Arc<Self>
    where
        F: Fn() + Send + Sync + 'static,
    {
        let thread = Arc::new(Self {
            task: Once::new(),
            process,
            tid,
            set_child_tid: Mutex::new(0),
            clear_child_tid: Mutex::new(0),
        });
        thread
            .task
            .call_once(|| Arc::new(TaskOptions::new(func).data(thread.clone()).build().unwrap()));

        thread
    }

    pub fn new_user_thread(
        user_space: Arc<UserSpace>,
        process: Weak<Process>,
        tid: Tid,
    ) -> Arc<Self> {
        fn user_task() {
            let current = Task::current().unwrap();
            let mut user_mode = {
                let user_space = current.user_space().unwrap();
                UserMode::new(user_space)
            };

            loop {
                let return_reason = user_mode.execute(|| false);
                let user_context = user_mode.context_mut();
                match return_reason {
                    ReturnReason::UserSyscall => {
                        crate::syscall::handle_syscall(user_context, current.user_space().unwrap())
                    }
                    ReturnReason::UserException => {
                        handle_exception(user_context, current.user_space().unwrap())
                    }
                    ReturnReason::KernelEvent => {}
                }
            }
        }

        let thread = Arc::new(Self {
            task: Once::new(),
            process,
            tid,
            set_child_tid: Mutex::new(0),
            clear_child_tid: Mutex::new(0),
        });
        thread.task.call_once(|| {
            Arc::new(
                TaskOptions::new(user_task)
                    .user_space(Some(user_space))
                    .data(thread.clone())
                    .build()
                    .unwrap(),
            )
        });
        thread
    }

    pub fn run(&self) {
        self.task.get().unwrap().run();
    }

    pub fn process(&self) -> Option<Arc<Process>> {
        self.process.upgrade()
    }

    // ================= Getter =======================

    pub fn tid(&self) -> Tid {
        self.tid
    }

    pub fn clear_child_tid(&self) -> MutexGuard<Vaddr> {
        self.clear_child_tid.lock()
    }

    pub fn set_child_tid(&self) -> MutexGuard<Vaddr> {
        self.set_child_tid.lock()
    }
}

pub fn current_thread() -> Arc<Thread> {
    Task::current()
        .unwrap()
        .data()
        .downcast_ref::<Arc<Thread>>()
        .unwrap()
        .clone()
}

static TID_ALLOC: AtomicU32 = AtomicU32::new(0);

pub fn alloc_tid() -> u32 {
    TID_ALLOC.fetch_add(1, Ordering::SeqCst)
}

fn handle_exception(user_context: &mut UserContext, _user_space: &UserSpace) {
    println!(
        "Catch CPU exception, skip this instruction. CPU exception: {:?} instruction addr: {:x?}, fault addr:{:x?}",
        user_context.trap_information().cpu_exception(),
        user_context.instruction_pointer(),
        user_context.trap_information().page_fault_addr,
    );
    user_context.set_instruction_pointer(user_context.instruction_pointer() + 2);
}