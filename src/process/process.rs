use align_ext::AlignExt;
use alloc::collections::btree_map::BTreeMap;
use alloc::string::String;
use log::debug;
use ostd::sync::{Mutex, MutexGuard, RwLock, WaitQueue};
use ostd::task::Task;

use alloc::sync::{Arc, Weak};
use alloc::vec::Vec;

use ostd::cpu::UserContext;
use ostd::mm::{VmIo, MAX_USERSPACE_VADDR, PAGE_SIZE};
use ostd::user::{UserContextApi, UserSpace};

use crate::process::heap::UserHeap;
use crate::thread::{alloc_tid, Thread};
use crate::vm::mapping::{MemorySpace, VmMapping};
use crate::vm::PageFlags;
use crate::{prelude::*, return_errno};

use super::status::ProcessStatus;
use super::{Pid, PROCESS_TABLE, USER_STACK_SIZE};

pub struct Process {
    // ======================== Basic info of process ===============================
    /// The id of this process.
    pid: Pid,
    /// Process state
    status: ProcessStatus,
    /// The name of this process, we use executable path for the user process.
    name: RwLock<String>,
    /// The threads of this process
    threads: Mutex<Vec<Arc<Thread>>>,

    // ======================== Memory-related fields ===============================
    /// The memory space of this process
    pub(super) memory_space: MemorySpace,
    /// The user space of this process contains CPU registers information.
    user_space: Option<Arc<UserSpace>>,
    /// The heap of the user process
    pub(crate) heap: UserHeap,

    // ======================== Process-tree fields =================================
    /// Parent process.
    parent_process: Mutex<Weak<Process>>,
    /// Children process.
    children: Mutex<BTreeMap<Pid, Arc<Process>>>,
    /// The WaitQueue for a child process to become a zombie.
    wait_children_queue: WaitQueue,
    // TODO: more field of process, including fd table...
}

impl Process {
    pub fn new_kernel_process<F>(func: F, name: String) -> Arc<Self>
    where
        F: Fn() + Send + Sync + 'static,
    {
        // FIXME: We use tid to indicate pid.
        let pid = alloc_tid();
        let process = Arc::new(Self {
            threads: Mutex::new(Vec::new()),
            user_space: None,
            heap: UserHeap::new(),
            pid,
            memory_space: MemorySpace::new(),
            parent_process: Mutex::new(Weak::new()),
            children: Mutex::new(BTreeMap::new()),
            status: ProcessStatus::new_uninit(),
            wait_children_queue: WaitQueue::new(),
            name: RwLock::new(name),
        });
        process.threads.lock().push(Thread::new_kernel_thread(
            func,
            Arc::downgrade(&process),
            pid,
        ));
        PROCESS_TABLE.lock().insert(pid, process.clone());
        process.status.set_runnable();
        process
    }

    pub fn new_user_process(program: &[u8], path: String) -> Arc<Self> {
        // FIXME: We use tid to indicate pid.
        let pid = alloc_tid();
        let (memory_space, user_context) = create_user_space(program);
        let user_space = Arc::new(UserSpace::new(
            memory_space.vm_space().clone(),
            user_context,
        ));
        let process = Arc::new(Self {
            threads: Mutex::new(Vec::new()),
            user_space: Some(user_space.clone()),
            heap: UserHeap::new(),
            pid,
            memory_space,
            parent_process: Mutex::new(Weak::new()),
            children: Mutex::new(BTreeMap::new()),
            status: ProcessStatus::new_uninit(),
            wait_children_queue: WaitQueue::new(),
            name: RwLock::new(path),
        });
        process.threads.lock().push(Thread::new_user_thread(
            user_space,
            Arc::downgrade(&process),
            pid,
        ));
        PROCESS_TABLE.lock().insert(pid, process.clone());
        process.status.set_runnable();
        process
    }

    /// Create user process based on constructed user context and memory space
    pub fn raw_new_user_process(
        user_context: UserContext,
        memory_space: MemorySpace,
        heap: &UserHeap,
        path: String,
    ) -> Arc<Self> {
        let pid = alloc_tid();
        let user_space = Arc::new(UserSpace::new(
            memory_space.vm_space().clone(),
            user_context,
        ));
        let process = Arc::new(Self {
            pid,
            memory_space,
            user_space: Some(user_space.clone()),
            heap: heap.clone(),
            threads: Mutex::new(Vec::new()),
            parent_process: Mutex::new(Weak::new()),
            children: Mutex::new(BTreeMap::new()),
            status: ProcessStatus::new_uninit(),
            wait_children_queue: WaitQueue::new(),
            name: RwLock::new(path),
        });
        process.threads.lock().push(Thread::new_user_thread(
            user_space,
            Arc::downgrade(&process),
            pid,
        ));
        PROCESS_TABLE.lock().insert(pid, process.clone());
        process.status.set_runnable();
        process
    }

    pub fn memory_space(&self) -> &MemorySpace {
        &self.memory_space
    }

    pub fn wait_with_pid_nonblock(&self, pid: Pid) -> Result<u32> {
        let mut children = self.children.lock();
        let Some(child) = children.get(&pid) else {
            return Err(Error::new(crate::error::Errno::ECHILD));
        };
        if child.status.is_zombie() {
            let child = children.remove(&pid).unwrap();
            return Ok(child.status.exit_code());
        }
        Err(Error::new(crate::error::Errno::EAGAIN))
    }

    pub fn wait_remove_one_nonblock(&self) -> Result<(Pid, u32)> {
        let mut children = self.children.lock();
        if children.is_empty() {
            return_errno!(Errno::ECHILD);
        }
        let mut wait_pid = None;
        for (pid, child) in children.iter() {
            if child.status.is_zombie() {
                wait_pid = Some(*pid);
                break;
            }
        }
        if let Some(pid) = wait_pid {
            let child = children.remove(&pid).unwrap();
            return Ok((pid, child.status.exit_code()));
        }
        Err(Error::new(crate::error::Errno::EAGAIN))
    }

    pub fn reparent_children_to_init(&self) {
        const INIT_PROCESS_ID: Pid = 1;
        if self.pid == INIT_PROCESS_ID || self.children.lock().is_empty() {
            return;
        }
        // Do re-parenting
        let mut self_children = self.children.lock();
        let process_table = PROCESS_TABLE.lock();
        let init_process = process_table.get(&INIT_PROCESS_ID).unwrap();
        let mut init_process_children = init_process.children.lock();
        while let Some((pid, child)) = self_children.pop_first() {
            *child.parent_process.lock() = Arc::downgrade(init_process);
            init_process_children.insert(pid, child);
        }
    }

    pub fn wait_children_queue(&self) -> &WaitQueue {
        &self.wait_children_queue
    }

    pub fn threads(&self) -> MutexGuard<Vec<Arc<Thread>>> {
        self.threads.lock()
    }

    pub fn set_parent_process(self: &Arc<Process>, parent_process: &Arc<Process>) {
        *self.parent_process.lock() = Arc::downgrade(parent_process);
        parent_process
            .children
            .lock()
            .insert(self.pid, self.clone());
    }

    pub fn run(&self) {
        // FIXME: We only run first thread.
        let thread = self.threads.lock().first().unwrap().clone();
        thread.run();
    }

    pub fn user_space(&self) -> Option<&Arc<UserSpace>> {
        self.user_space.as_ref()
    }

    pub fn exit(&self, exit_code: u32) {
        self.status.set_zombie(exit_code);
        if let Some(parent) = self.parent_process.lock().upgrade() {
            parent.wait_children_queue.wake_all();
        }
    }

    pub fn status(&self) -> &ProcessStatus {
        &self.status
    }

    pub fn pid(&self) -> Pid {
        self.pid
    }

    pub fn name(&self) -> String {
        self.name.read().clone()
    }

    pub fn set_name(&self, name: String) {
        *self.name.write() = name;
    }

    pub fn parent_process(&self) -> Weak<Process> {
        self.parent_process.lock().clone()
    }
}

pub fn current_process() -> Option<Arc<Process>> {
    Task::current()
        .unwrap()
        .data()
        .downcast_ref::<Arc<Thread>>()
        .unwrap()
        .process()
        .clone()
}

pub fn load_elf_to_vm_and_context(
    program: &[u8],
    memory_space: &MemorySpace,
    context: &mut UserContext,
) {
    memory_space.clear();

    // Reset context
    let default_context = UserContext::default();
    *context.general_regs_mut() = *default_context.general_regs();
    context.set_tls_pointer(default_context.tls_pointer());
    *context.fp_regs_mut() = *default_context.fp_regs();

    parse_elf(program, memory_space, context);
}

pub fn create_user_space(program: &[u8]) -> (MemorySpace, UserContext) {
    let memory_space = MemorySpace::new();
    let mut user_context = UserContext::default();

    parse_elf(program, &memory_space, &mut user_context);
    (memory_space, user_context)
}

fn parse_elf(input: &[u8], memory_space: &MemorySpace, user_cpu_state: &mut UserContext) {
    let header = xmas_elf::header::parse_header(input).unwrap();

    let pt2 = header.pt2;
    let ph_count = pt2.ph_count();

    // First, map each ph
    for index in 0..ph_count {
        let program_header = xmas_elf::program::parse_program_header(input, header, index).unwrap();
        let ph64 = match program_header {
            xmas_elf::program::ProgramHeader::Ph64(ph64) => *ph64,
            xmas_elf::program::ProgramHeader::Ph32(_) => {
                todo!("Not 64 byte executable")
            }
        };
        if let Ok(typ) = ph64.get_type() {
            if typ == xmas_elf::program::Type::Load {
                let raw_start_addr = ph64.virtual_addr;
                let raw_end_addr = ph64.virtual_addr + ph64.mem_size;

                let start_addr = (raw_start_addr as usize).align_down(PAGE_SIZE);
                let end_addr = (raw_end_addr as usize).align_up(PAGE_SIZE);

                debug!(
                    "Mapping elf, raw_start_addr: {:x?}, raw_end_addr: {:x?}, mem_size: {:x?}, file_size: {:x?}",
                    raw_start_addr, raw_end_addr,ph64.mem_size,ph64.file_size
                );

                let mut perms = PageFlags::empty();
                if ph64.flags.is_execute() {
                    perms |= PageFlags::X;
                }
                if ph64.flags.is_read() {
                    perms |= PageFlags::R;
                }
                if ph64.flags.is_write() {
                    perms |= PageFlags::W;
                }

                let nframes = (end_addr - start_addr) / PAGE_SIZE;
                let frames = memory_space.map(VmMapping::new(start_addr, nframes, perms));

                let copy_bytes =
                    &input[ph64.offset as usize..(ph64.offset + ph64.file_size) as usize];
                let mut remain_bytes = copy_bytes.len();
                let mut index = 0;

                while remain_bytes > 0 {
                    let offset = if index == 0 {
                        raw_start_addr as usize - start_addr
                    } else {
                        0
                    };
                    let length = remain_bytes.min(PAGE_SIZE) - offset;
                    let base = copy_bytes.len() - remain_bytes;
                    frames
                        .get(index)
                        .unwrap()
                        .write_bytes(offset, &copy_bytes[base..(base + length)])
                        .unwrap();
                    index += 1;
                    remain_bytes -= length;
                }
            }
        }
    }

    // Second, init the user stack with addr: 0x8000_0000_0000 - 10 * PAGE_SIZE.
    // Stack size: 32 PAGE_SIZE
    let stack_low = 0x8000_0000_0000 - 10 * PAGE_SIZE - USER_STACK_SIZE;
    memory_space.map(VmMapping::new(
        stack_low,
        USER_STACK_SIZE / PAGE_SIZE,
        PageFlags::RW,
    ));
    user_cpu_state.set_stack_pointer(MAX_USERSPACE_VADDR - 10 * PAGE_SIZE - 32);
    user_cpu_state.set_instruction_pointer(header.pt2.entry_point() as usize);

    // Third, map the 0 address
    memory_space.map(VmMapping::new(0, 1, PageFlags::RW));
}
