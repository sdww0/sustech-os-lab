// SPDX-License-Identifier: MPL-2.0
// Ref: asterinas: ostd/src/task/scheduler/fifo_scheduler.rs

use alloc::{boxed::Box, collections::VecDeque, sync::Arc};
use ostd::{
    cpu::CpuId,
    sync::SpinLock,
    task::{
        disable_preempt,
        scheduler::{inject_scheduler, EnqueueFlags, LocalRunQueue, Scheduler, UpdateFlags},
        Task,
    },
};

pub fn init() {
    let fifo_scheduler = Box::new(FifoScheduler {
        rq: SpinLock::new(FifoRunQueue::new()),
    });
    inject_scheduler(Box::leak(fifo_scheduler));
}

/// A simple FIFO (First-In-First-Out) task scheduler.
struct FifoScheduler {
    rq: SpinLock<FifoRunQueue>,
}

impl Scheduler for FifoScheduler {
    fn enqueue(&self, runnable: Arc<Task>, _flags: EnqueueFlags) -> Option<CpuId> {
        let mut rq = self.rq.disable_irq().lock();
        rq.queue.push_back(runnable);
        Some(CpuId::bsp())
    }

    fn local_rq_with(&self, f: &mut dyn FnMut(&dyn LocalRunQueue<Task>)) {
        let _preempt_guard = disable_preempt();
        let local_rq: &FifoRunQueue = &self.rq.disable_irq().lock();
        f(local_rq);
    }

    fn local_mut_rq_with(&self, f: &mut dyn FnMut(&mut dyn LocalRunQueue<Task>)) {
        let _preempt_guard = disable_preempt();
        let local_rq: &mut FifoRunQueue = &mut self.rq.disable_irq().lock();
        f(local_rq);
    }
}

struct FifoRunQueue {
    current: Option<Arc<Task>>,
    queue: VecDeque<Arc<Task>>,
}

impl FifoRunQueue {
    pub const fn new() -> Self {
        Self {
            current: None,
            queue: VecDeque::new(),
        }
    }
}

impl LocalRunQueue for FifoRunQueue {
    fn current(&self) -> Option<&Arc<Task>> {
        self.current.as_ref()
    }

    fn update_current(&mut self, flags: UpdateFlags) -> bool {
        !matches!(flags, UpdateFlags::Tick)
    }

    fn pick_next_current(&mut self) -> Option<&Arc<Task>> {
        let next_task = self.queue.pop_front()?;
        if let Some(prev_task) = self.current.replace(next_task) {
            self.queue.push_back(prev_task);
        }

        self.current.as_ref()
    }

    fn dequeue_current(&mut self) -> Option<Arc<Task>> {
        self.current.take()
    }
}
