use core::sync::atomic::{AtomicUsize, Ordering};

use align_ext::AlignExt;
use ostd::mm::{FrameAllocOptions, PageFlags, PageProperty, Vaddr, PAGE_SIZE};

use super::current_process;

pub struct UserHeap {
    _base: Vaddr,
    _limit: usize,
    current_end: AtomicUsize,
}

impl UserHeap {
    pub fn new() -> Self {
        Self {
            _base: 0x1000_0000,
            _limit: 1024 * PAGE_SIZE,
            current_end: AtomicUsize::new(0x1000_0000),
        }
    }

    pub fn current_end(&self) -> usize {
        self.current_end.load(Ordering::Relaxed)
    }

    pub fn brk(&self, new_end: Option<Vaddr>) -> Option<Vaddr> {
        match new_end {
            Some(new_end) => {
                let process = current_process()?;
                let user_space = process.user_space.clone()?;
                let current_end = self.current_end.load(Ordering::Acquire);
                if new_end <= current_end {
                    // FIXME: should we allow shrink current user heap?
                    return Some(current_end);
                }
                let old_vaddr = current_end.align_up(PAGE_SIZE);
                let new_vaddr = new_end.align_up(PAGE_SIZE);

                if new_vaddr > old_vaddr {
                    let frames = FrameAllocOptions::new((new_vaddr - old_vaddr) / PAGE_SIZE)
                        .alloc()
                        .unwrap();

                    let mut cursor = user_space
                        .vm_space()
                        .cursor_mut(&(old_vaddr..new_vaddr))
                        .unwrap();
                    for frame in frames {
                        cursor.map(
                            frame,
                            PageProperty::new(PageFlags::RW, ostd::mm::CachePolicy::Writeback),
                        );
                    }
                }
                self.current_end.store(new_end, Ordering::Release);
                Some(new_end)
            }
            None => Some(self.current_end.load(Ordering::Relaxed)),
        }
    }
}
