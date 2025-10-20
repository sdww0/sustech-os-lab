mod fifo;
mod rr;

use alloc::boxed::Box;
use fifo::FifoScheduler;
use ostd::task::scheduler::inject_scheduler;
use rr::RrScheduler;

const USE_RR_SCHEDULER: bool = false;

pub fn init() {
    if USE_RR_SCHEDULER {
        let rr_scheduler = Box::new(RrScheduler::default());
        inject_scheduler(Box::leak(rr_scheduler));
    } else {
        let fifo_scheduler = Box::new(FifoScheduler::default());
        inject_scheduler(Box::leak(fifo_scheduler));
    }
    ostd::task::scheduler::enable_preemption_on_cpu();
}

