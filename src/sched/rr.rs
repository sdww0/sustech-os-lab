use alloc::{collections::vec_deque::VecDeque, sync::Arc};
use ostd::{
    cpu::CpuId,
    sync::SpinLock,
    task::{
        disable_preempt,
        scheduler::{EnqueueFlags, LocalRunQueue, Scheduler},
        Task,
    },
    trap::disable_local,
};

pub struct RrScheduler {
    run_queue: SpinLock<RrRunQueue>,
}

impl Scheduler for RrScheduler {
    fn enqueue(&self, runnable: Arc<Task>, _flags: EnqueueFlags) -> Option<CpuId> {
        let _irq = disable_local();
        let mut rq = self.run_queue.lock();
        rq.entities.push_back(Entity {
            task: runnable,
            time_slice: TimeSlice::default(),
        });
        Some(CpuId::bsp())
    }

    fn local_rq_with(&self, f: &mut dyn FnMut(&dyn LocalRunQueue<Task>)) {
        let _guard = disable_preempt();
        let rq = self.run_queue.disable_irq().lock();
        f(&*rq)
    }

    fn local_mut_rq_with(&self, f: &mut dyn FnMut(&mut dyn LocalRunQueue<Task>)) {
        let _guard = disable_preempt();
        let mut rq = self.run_queue.disable_irq().lock();
        f(&mut *rq)
    }
}

impl Default for RrScheduler {
    fn default() -> Self {
        Self {
            run_queue: SpinLock::new(RrRunQueue::default()),
        }
    }
}

#[derive(Default)]
struct RrRunQueue {
    current: Option<Entity>,
    entities: VecDeque<Entity>,
}

impl LocalRunQueue for RrRunQueue {
    fn current(&self) -> Option<&Arc<Task>> {
        self.current.as_ref().map(|entity| &entity.task)
    }

    fn update_current(&mut self, flags: ostd::task::scheduler::UpdateFlags) -> bool {
        match flags {
            ostd::task::scheduler::UpdateFlags::Tick => {
                let Some(entity) = self.current.as_mut() else {
                    return false;
                };
                entity.time_slice.elapse()
            }
            _ => true,
        }
    }

    fn pick_next_current(&mut self) -> Option<&Arc<Task>> {
        if let Some(current_task) = self.current.replace(self.entities.pop_front()?) {
            self.entities.push_back(current_task);
        }
        self.current.as_ref().map(|entity| &entity.task)
    }

    fn dequeue_current(&mut self) -> Option<Arc<Task>> {
        self.current.take().map(|entity| entity.task)
    }
}

struct Entity {
    task: Arc<Task>,
    time_slice: TimeSlice,
}

#[derive(Default)]
struct TimeSlice {
    tick: usize,
}

impl TimeSlice {
    const PROCESS_TIME_SLICE: usize = 100;

    fn elapse(&mut self) -> bool {
        self.tick = (self.tick + 1) % Self::PROCESS_TIME_SLICE;

        self.tick == 0
    }
}
