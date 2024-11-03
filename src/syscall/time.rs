use int_to_c_enum::TryFromInt;
use ostd::timer::Jiffies;

use super::SyscallReturn;

use crate::{prelude::*, process::current_process, time::timespec_t};

pub fn sys_clock_gettime(clockid: i32, timespec_addr: Vaddr) -> Result<SyscallReturn> {
    debug!("Gettime, clockid: {}", clockid);

    let clock = ClockId::try_from(clockid)?;

    let current_process = current_process().unwrap();
    let vm_space = current_process.memory_space().vm_space();
    let mut writer = vm_space.writer(timespec_addr, size_of::<timespec_t>())?;

    match clock {
        ClockId::CLOCK_REALTIME => todo!(),
        ClockId::CLOCK_MONOTONIC => {
            // Just use the Jiffies
            let duration = Jiffies::elapsed().as_duration();
            writer.write_val(&timespec_t::from(duration))?;
        }
        _ => todo!(),
    }

    Ok(SyscallReturn::Return(0))
}

// The hard-coded clock IDs.
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
