// SPDX-License-Identifier: MPL-2.0
// Copy from asterinas:kernel/src/process/status.rs

//! The process status

use core::sync::atomic::{AtomicU64, Ordering};

/// The status of process.
///
/// The `ProcessStatus` can be viewed as two parts,
/// the highest 32 bits is the value of `TermStatus`, if any,
/// the lowest 32 bits is the value of status.
#[derive(Debug)]
pub struct ProcessStatus(AtomicU64);

#[repr(u8)]
enum Status {
    Uninit = 0,
    Runnable = 1,
    Zombie = 2,
}

impl ProcessStatus {
    const LOW_MASK: u64 = 0xffff_ffff;

    pub fn new_uninit() -> Self {
        Self(AtomicU64::new(Status::Uninit as u64))
    }

    pub fn set_zombie(&self, exit_code: u32) {
        let new_val = (exit_code as u64) << 32 | Status::Zombie as u64;
        self.0.store(new_val, Ordering::Relaxed);
    }

    pub fn is_zombie(&self) -> bool {
        self.0.load(Ordering::Relaxed) & Self::LOW_MASK == Status::Zombie as u64
    }

    pub fn set_runnable(&self) {
        let new_val = Status::Runnable as u64;
        self.0.store(new_val, Ordering::Relaxed);
    }

    pub fn is_runnable(&self) -> bool {
        self.0.load(Ordering::Relaxed) & Self::LOW_MASK == Status::Runnable as u64
    }

    /// Returns the exit code.
    ///
    /// If the process is not exited, the exit code is zero.
    /// But if exit code is zero, the process may or may not exit.
    pub fn exit_code(&self) -> u32 {
        let val = self.0.load(Ordering::Relaxed);
        (val >> 32 & Self::LOW_MASK) as u32
    }
}
