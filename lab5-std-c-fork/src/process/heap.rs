use core::sync::atomic::{AtomicUsize, Ordering};

use align_ext::AlignExt;
use ostd::mm::{PAGE_SIZE, PageFlags, Vaddr};

use crate::mm::VmMapping;

use super::current_process;

#[derive(Debug)]
pub struct UserHeap {
    base: Vaddr,
    limit: usize,
    current_end: AtomicUsize,
}

impl UserHeap {
    pub fn new() -> Self {
        Self {
            base: 0x1000_0000,
            limit: 1024 * PAGE_SIZE,
            current_end: AtomicUsize::new(0x1000_0000),
        }
    }

    pub fn current_end(&self) -> usize {
        self.current_end.load(Ordering::Relaxed)
    }

    pub fn brk(&self, new_end: Option<Vaddr>) -> Option<Vaddr> {
        match new_end {
            Some(new_end) => {
                let process = current_process();
                let current_end = self.current_end.load(Ordering::Acquire);
                if new_end <= current_end {
                    return Some(current_end);
                }
                let old_vaddr = current_end.align_up(PAGE_SIZE);
                let new_vaddr = new_end.align_up(PAGE_SIZE);

                if new_vaddr > old_vaddr {
                    let pages = (new_vaddr - old_vaddr) / PAGE_SIZE;
                    process
                        .memory_space()
                        .map(VmMapping::new(old_vaddr, pages, PageFlags::RW));
                }
                self.current_end.store(new_end, Ordering::Release);
                Some(new_end)
            }
            None => Some(self.current_end.load(Ordering::Relaxed)),
        }
    }
}

impl Default for UserHeap {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for UserHeap {
    fn clone(&self) -> Self {
        Self {
            base: self.base,
            limit: self.limit,
            current_end: AtomicUsize::new(self.current_end.load(Ordering::Relaxed)),
        }
    }
}
