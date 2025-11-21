use core::time::Duration;

use alloc::sync::Arc;
use int_to_c_enum::TryFromInt;
use log::debug;
use ostd::{Pod, mm::Vaddr, timer::Jiffies};

use super::SyscallReturn;

use crate::{error::Result, process::Process};

pub fn sys_clock_gettime(
    clockid: i32,
    timespec_addr: Vaddr,
    current_process: &Arc<Process>,
) -> Result<SyscallReturn> {
    debug!("Gettime, clockid: {}", clockid);

    let clock = ClockId::try_from(clockid).unwrap();

    let vm_space = current_process.memory_space().vm_space();
    let mut writer = vm_space
        .writer(timespec_addr, size_of::<timespec_t>())
        .unwrap();

    match clock {
        ClockId::CLOCK_REALTIME => todo!(),
        ClockId::CLOCK_MONOTONIC => {
            // Just use the Jiffies
            let duration = Jiffies::elapsed().as_duration();
            writer.write_val(&timespec_t::from(duration)).unwrap();
        }
        _ => todo!(),
    }

    Ok(SyscallReturn(0))
}

#[derive(Debug, Copy, Clone, TryFromInt, PartialEq)]
#[repr(i32)]
#[allow(non_camel_case_types)]
pub enum ClockId {
    CLOCK_REALTIME = 0,
    CLOCK_MONOTONIC = 1,
    CLOCK_PROCESS_CPUTIME_ID = 2,
    CLOCK_THREAD_CPUTIME_ID = 3,
    CLOCK_MONOTONIC_RAW = 4,
    CLOCK_REALTIME_COARSE = 5,
    CLOCK_MONOTONIC_COARSE = 6,
    CLOCK_BOOTTIME = 7,
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, Pod)]
pub struct timespec_t {
    pub sec: i64,
    pub nsec: i64,
}

impl From<Duration> for timespec_t {
    fn from(duration: Duration) -> timespec_t {
        let sec = duration.as_secs() as i64;
        let nsec = duration.subsec_nanos() as i64;
        timespec_t { sec, nsec }
    }
}
