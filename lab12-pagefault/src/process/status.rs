use core::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, PartialEq, Eq)]
#[repr(u64)]
enum Status {
    Uninit = 0,
    Runnable = 1,
    Zombie = 2,
}

/// The status of a process.
///
/// ```
/// 0-31: Status (0: Uninit, 1: Runnable, 2: Zombie)
/// 32-63: Exit code (if status is Zombie)
/// ```
pub struct ProcessStatus(AtomicU64);

impl ProcessStatus {
    pub fn new() -> Self {
        ProcessStatus(AtomicU64::new(Status::Uninit as u64))
    }

    pub fn exit(&self, exit_code: u32) {
        let status = self.get_status();
        assert!(status == Status::Runnable);
        let value = (Status::Zombie as u64) | ((exit_code as u64) << 32);
        self.0.store(value, Ordering::SeqCst);
    }

    pub fn exit_code(&self) -> Option<u32> {
        let status = self.get_status();
        if status == Status::Zombie {
            let value = self.0.load(Ordering::SeqCst);
            Some((value >> 32) as u32)
        } else {
            None
        }
    }

    pub fn set_runnable(&self) {
        self.0.store(Status::Runnable as u64, Ordering::SeqCst);
    }

    pub fn is_zombie(&self) -> bool {
        self.get_status() == Status::Zombie
    }

    fn get_status(&self) -> Status {
        match self.0.load(Ordering::SeqCst) & 0xFFFF_FFFF {
            0 => Status::Uninit,
            1 => Status::Runnable,
            2 => Status::Zombie,
            _ => panic!("Invalid process status"),
        }
    }
}
