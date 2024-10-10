pub mod heap;

use align_ext::AlignExt;
use heap::UserHeap;
use ostd::task::Task;
use spin::Mutex;

use alloc::sync::Arc;
use alloc::vec::Vec;

use ostd::cpu::UserContext;
use ostd::mm::{CachePolicy, FrameAllocOptions, PageFlags, PageProperty, VmIo, VmSpace, PAGE_SIZE};
use ostd::user::{UserContextApi, UserSpace};

use crate::thread::{alloc_tid, Thread};

pub type Pid = u32;

pub struct Process {
    pid: Pid,

    thread: Mutex<Vec<Arc<Thread>>>,
    pub(self) user_space: Option<Arc<UserSpace>>,

    pub(crate) heap: UserHeap,
}

impl Process {
    pub fn new_kernel_process<F>(func: F) -> Arc<Self>
    where
        F: Fn() + Send + Sync + 'static,
    {
        // FIXME: We use tid to indicate pid.
        let pid = alloc_tid();
        let process = Arc::new(Self {
            thread: Mutex::new(Vec::new()),
            user_space: None,
            heap: UserHeap::new(),
            pid,
        });
        process.thread.lock().push(Thread::new_kernel_thread(
            func,
            Arc::downgrade(&process),
            pid,
        ));
        process
    }

    pub fn new_user_process(program: &[u8]) -> Arc<Self> {
        // FIXME: We use tid to indicate pid.
        let pid = alloc_tid();
        let user_space = Arc::new(create_user_space(program));
        let process = Arc::new(Self {
            thread: Mutex::new(Vec::new()),
            user_space: Some(user_space.clone()),
            heap: UserHeap::new(),
            pid,
        });
        process.thread.lock().push(Thread::new_user_thread(
            user_space,
            Arc::downgrade(&process),
            pid,
        ));
        process
    }

    pub fn run(&self) {
        // FIXME: We only run first thread.
        let thread = self.thread.lock().get(0).unwrap().clone();
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

pub fn create_user_space(program: &[u8]) -> UserSpace {
    parse_elf(program)
}

fn parse_elf(input: &[u8]) -> UserSpace {
    let header = xmas_elf::header::parse_header(input).unwrap();

    let pt2 = header.pt2;
    let ph_count = pt2.ph_count();
    let entry = header.pt2.entry_point();

    let user_address_space = VmSpace::new();
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

                    let nframes = (end_addr - start_addr) / PAGE_SIZE;
                    let frames = FrameAllocOptions::new(nframes).alloc().unwrap();
                    frames
                        .write_bytes(
                            raw_start_addr as usize - start_addr,
                            &input[ph64.offset as usize..(ph64.offset + ph64.file_size) as usize],
                        )
                        .unwrap();

                    let mut cursor = user_address_space
                        .cursor_mut(&(start_addr..end_addr))
                        .unwrap();

                    let mut flags = PageFlags::empty();
                    if ph64.flags.is_execute() {
                        flags |= PageFlags::X;
                    }
                    if ph64.flags.is_read() {
                        flags |= PageFlags::R;
                    }
                    if ph64.flags.is_write() {
                        flags |= PageFlags::W;
                    }
                    let map_prop = PageProperty::new(flags, CachePolicy::Writeback);
                    for frame in frames {
                        cursor.map(frame, map_prop);
                    }
                }
                _ => {}
            }
        }
    }

    // Second, init the user stack with addr: 0x8000_0000_0000 - 10 * PAGE_SIZE.
    // Stack size: 32 PAGE_SIZE
    let frames = FrameAllocOptions::new(32).alloc().unwrap();
    let map_prop = PageProperty::new(PageFlags::RW, CachePolicy::Writeback);
    let mut cursor = user_address_space
        .cursor_mut(
            &((0x8000_0000_0000 - 10 * PAGE_SIZE - 32 * PAGE_SIZE)
                ..(0x8000_0000_0000 - 10 * PAGE_SIZE)),
        )
        .unwrap();
    for frame in frames {
        cursor.map(frame, map_prop);
    }
    drop(cursor);

    // Third, map the 0 address
    let frames = FrameAllocOptions::new(1).alloc().unwrap();
    let map_prop = PageProperty::new(PageFlags::RW, CachePolicy::Writeback);
    let mut cursor = user_address_space.cursor_mut(&(0..PAGE_SIZE)).unwrap();
    for frame in frames {
        cursor.map(frame, map_prop);
    }

    drop(cursor);
    user_cpu_state.set_stack_pointer(0x8000_0000_0000 - 10 * PAGE_SIZE - 32);

    UserSpace::new(Arc::new(user_address_space), user_cpu_state)
}
