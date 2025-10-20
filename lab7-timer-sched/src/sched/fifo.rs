use alloc::{collections::VecDeque, sync::Arc};
use ostd::{
    cpu::CpuId,
    sync::SpinLock,
    task::{
        Task, disable_preempt,
        scheduler::{EnqueueFlags, LocalRunQueue, Scheduler, UpdateFlags},
    },
};

pub struct FifoScheduler {
    run_queue: SpinLock<FifoRunQueue>,
}

impl Scheduler for FifoScheduler {
    fn enqueue(&self, runnable: Arc<Task>, _flags: EnqueueFlags) -> Option<CpuId> {
        let mut run_queue = self.run_queue.disable_irq().lock();
        run_queue.queue.push_back(runnable);
        None
    }

    fn local_rq_with(&self, f: &mut dyn FnMut(&dyn LocalRunQueue<Task>)) {
        let _guard = disable_preempt();
        let local_rq: &FifoRunQueue = &self.run_queue.disable_irq().lock();
        f(local_rq);
    }

    fn mut_local_rq_with(&self, f: &mut dyn FnMut(&mut dyn LocalRunQueue<Task>)) {
        let _guard = disable_preempt();
        let local_rq: &mut FifoRunQueue = &mut self.run_queue.disable_irq().lock();
        f(local_rq);
    }
}

impl Default for FifoScheduler {
    fn default() -> Self {
        Self {
            run_queue: SpinLock::new(FifoRunQueue::new()),
        }
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

    fn dequeue_current(&mut self) -> Option<Arc<Task>> {
        self.current.take()
    }

    fn try_pick_next(&mut self) -> Option<&Arc<Task>> {
        if let Some(current_task) = self.current.replace(self.queue.pop_front()?) {
            self.queue.push_back(current_task);
        }

        self.current.as_ref()
    }
}
