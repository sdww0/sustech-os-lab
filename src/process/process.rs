use align_ext::AlignExt;
use alloc::collections::btree_map::BTreeMap;
use ostd::sync::{Mutex, MutexGuard};
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

use super::{Pid, PROCESS_TABLE, USER_STACK_SIZE};

pub struct Process {
    pid: Pid,

    // ------ Memory related ------
    pub(super) memory_space: MemorySpace,
    user_space: Option<Arc<UserSpace>>,
    pub(crate) heap: UserHeap,

    parent_process: Mutex<Weak<Process>>,
    children: Mutex<BTreeMap<Pid, Arc<Process>>>,

    threads: Mutex<Vec<Arc<Thread>>>,
}

impl Process {
    pub fn new_kernel_process<F>(func: F) -> Arc<Self>
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
        });
        process.threads.lock().push(Thread::new_kernel_thread(
            func,
            Arc::downgrade(&process),
            pid,
        ));
        PROCESS_TABLE.lock().insert(pid, process.clone());
        process
    }

    pub fn new_user_process(program: &[u8]) -> Arc<Self> {
        // FIXME: We use tid to indicate pid.
        let pid = alloc_tid();
        let (memory_space, user_space) = create_user_space(program);
        let process = Arc::new(Self {
            threads: Mutex::new(Vec::new()),
            user_space: Some(user_space.clone()),
            heap: UserHeap::new(),
            pid,
            memory_space,
            parent_process: Mutex::new(Weak::new()),
            children: Mutex::new(BTreeMap::new()),
        });
        process.threads.lock().push(Thread::new_user_thread(
            user_space,
            Arc::downgrade(&process),
            pid,
        ));
        PROCESS_TABLE.lock().insert(pid, process.clone());
        process
    }

    /// Create user process based on constructed user context and memory space
    pub fn raw_new_user_process(
        user_context: UserContext,
        memory_space: MemorySpace,
        heap: &UserHeap,
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
        });
        process.threads.lock().push(Thread::new_user_thread(
            user_space,
            Arc::downgrade(&process),
            pid,
        ));
        PROCESS_TABLE.lock().insert(pid, process.clone());
        process
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
        let thread = self.threads.lock().get(0).unwrap().clone();
        thread.run();
    }

    pub fn user_space(&self) -> Option<&Arc<UserSpace>> {
        self.user_space.as_ref()
    }

    pub fn pid(&self) -> Pid {
        self.pid
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

pub fn create_user_space(program: &[u8]) -> (MemorySpace, Arc<UserSpace>) {
    parse_elf(program)
}

fn parse_elf(input: &[u8]) -> (MemorySpace, Arc<UserSpace>) {
    let header = xmas_elf::header::parse_header(input).unwrap();

    let pt2 = header.pt2;
    let ph_count = pt2.ph_count();
    let entry = header.pt2.entry_point();

    let memory_space = MemorySpace::new();
    let mut user_cpu_state = {
        let mut user_cpu_state = UserContext::default();
        user_cpu_state.set_instruction_pointer(entry as usize);
        user_cpu_state
    };

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
            match typ {
                xmas_elf::program::Type::Load => {
                    let raw_start_addr = ph64.virtual_addr;
                    let raw_end_addr = ph64.virtual_addr + ph64.mem_size;

                    let start_addr = (raw_start_addr as usize).align_down(PAGE_SIZE);
                    let end_addr = (raw_end_addr as usize).align_up(PAGE_SIZE);

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
                    frames
                        .write_bytes(
                            raw_start_addr as usize - start_addr,
                            &input[ph64.offset as usize..(ph64.offset + ph64.file_size) as usize],
                        )
                        .unwrap();
                }
                _ => {}
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

    // Third, map the 0 address
    memory_space.map(VmMapping::new(0, 1, PageFlags::RW));

    let user_address_space = memory_space.vm_space().clone();
    (
        memory_space,
        Arc::new(UserSpace::new(user_address_space, user_cpu_state)),
    )
}
