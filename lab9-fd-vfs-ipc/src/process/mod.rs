mod elf;
mod heap;
mod status;

use core::sync::atomic::{AtomicUsize, Ordering};

use alloc::boxed::Box;
use alloc::collections::btree_map::BTreeMap;
use alloc::sync::{Arc, Weak};
use log::{debug, info};
use ostd::arch::cpu::context::UserContext;
use ostd::arch::qemu::{exit_qemu, QemuExitCode};
use ostd::early_println;
use ostd::sync::{Mutex, WaitQueue};
use ostd::task::{Task, TaskOptions};
use ostd::user::{ReturnReason, UserContextApi, UserMode};
use riscv::register::scause::Exception;
use spin::Once;

use crate::error::{Errno, Error, Result};
use crate::mm::MemorySpace;
use crate::process::heap::UserHeap;
use crate::process::status::ProcessStatus;
pub const USER_STACK_SIZE: usize = 8192 * 1024; // 8MB

static PROCESS_TABLE: Mutex<BTreeMap<Pid, Arc<Process>>> = Mutex::new(BTreeMap::new());

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
    /// The WaitQueue for a child process to become a zombie.
    wait_children_queue: WaitQueue,
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
            wait_children_queue: WaitQueue::new(),
        });

        let task = create_user_task(&process, Box::new(user_context));
        process.task.call_once(|| task);
        process.status.set_runnable();
        PROCESS_TABLE.lock().insert(process.pid(), process.clone());

        process
    }

    pub fn fork(self: &Arc<Self>, user_context: &UserContext) -> Arc<Process> {
        let memory_space = self.memory_space.duplicate();

        let user_context = {
            let mut ctx = user_context.clone();
            ctx.set_a0(0);
            ctx
        };

        let child_process = Arc::new(Process {
            pid: alloc_pid(),
            status: ProcessStatus::new(),
            task: Once::new(),
            memory_space,
            heap: UserHeap::new(),
            parent_process: Mutex::new(Arc::downgrade(self)),
            children: Mutex::new(BTreeMap::new()),
            wait_children_queue: WaitQueue::new(),
        });

        let task = create_user_task(&child_process, Box::new(user_context));
        child_process.task.call_once(|| task);
        child_process.status.set_runnable();

        self.children
            .lock()
            .insert(child_process.pid(), child_process.clone());
        PROCESS_TABLE
            .lock()
            .insert(child_process.pid(), child_process.clone());

        child_process
    }

    pub fn exec(&self, binary: &[u8]) -> UserContext {
        self.memory_space.clear();
        elf::load_user_space(binary, &self.memory_space)
    }

    pub fn wait(&self, wait_pid: i32) -> Result<(Pid, u32)> {
        let wait_pid = if wait_pid == -1 {
            None
        } else {
            Some(wait_pid.abs() as Pid)
        };

        let res = self.try_wait(wait_pid);

        match res {
            Ok((pid, exit_code)) => return Ok((pid as Pid, exit_code)),
            Err(err) if err.code == Errno::EAGAIN => {}
            Err(err) => return Err(err),
        }

        // No child exit, waiting...
        let wait_queue = &self.wait_children_queue;
        Ok(wait_queue.wait_until(|| self.try_wait(wait_pid).ok()))
    }

    pub fn reparent_children_to_init(&self) {
        const INIT_PROCESS_ID: Pid = 1;
        if self.pid == INIT_PROCESS_ID || self.children.lock().is_empty() {
            return;
        }

        // Do re-parenting
        let init_process = {
            let process_table = PROCESS_TABLE.lock();
            process_table.get(&INIT_PROCESS_ID).unwrap().clone()
        };

        let mut init_children = init_process.children.lock();
        let mut self_children = self.children.lock();
        while let Some((pid, child)) = self_children.pop_first() {
            *child.parent_process.lock() = Arc::downgrade(&init_process);
            init_children.insert(pid, child);
        }
    }

    pub fn parent_process(&self) -> Option<Arc<Process>> {
        self.parent_process.lock().upgrade()
    }

    pub fn exit(&self, exit_code: u32) {
        self.status.exit(exit_code);
        self.reparent_children_to_init();
        // Wakeup the parent process if it is waiting.
        if let Some(parent) = self.parent_process() {
            parent.wait_children_queue.wake_all();
        }
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

    fn try_wait(&self, pid: Option<Pid>) -> Result<(Pid, u32)> {
        let mut children = self.children.lock();
        if children.is_empty() {
            return Err(Error::new(Errno::ECHILD));
        }

        let mut wait_pid = None;
        if let Some(pid) = pid {
            if let Some(child) = children.get(&pid) {
                if child.status.is_zombie() {
                    wait_pid = Some(pid);
                }
            } else {
                return Err(Error::new(Errno::ECHILD));
            }
        } else {
            for (child_pid, child) in children.iter() {
                debug!(
                    "try_wait: check child pid = {}, is zombie = {:?}",
                    child_pid,
                    child.status.is_zombie()
                );
                if child.status.is_zombie() {
                    wait_pid = Some(*child_pid);
                    break;
                }
            }
        }

        debug!("try_wait: wait_pid = {:?}", wait_pid);

        if let Some(pid) = wait_pid {
            let child = children.remove(&pid).unwrap();
            PROCESS_TABLE.lock().remove(&pid);
            return Ok((pid, child.status.exit_code().unwrap()));
        }

        Err(Error::new(crate::error::Errno::EAGAIN))
    }
}

fn create_user_task(process: &Arc<Process>, user_context: Box<UserContext>) -> Arc<Task> {
    let entry = move |user_ctx| {
        let process = current_process();

        let mut user_mode = UserMode::new(user_ctx);
        let vm_space = process.memory_space().vm_space();

        loop {
            vm_space.activate();
            let return_reason = user_mode.execute(|| true);
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
                ReturnReason::KernelEvent => {
                    ostd::task::halt_cpu();
                }
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
