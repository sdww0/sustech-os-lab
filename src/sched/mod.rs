use alloc::boxed::Box;
use ostd::task::scheduler::inject_scheduler;
use rr::RrScheduler;

pub mod rr;

pub fn init() {
    let scheduler = Box::new(RrScheduler::default());
    inject_scheduler(Box::leak(scheduler));
}
