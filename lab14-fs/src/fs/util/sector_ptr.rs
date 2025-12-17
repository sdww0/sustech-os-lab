use alloc::sync::{Arc, Weak};
use ostd::Pod;

use crate::drivers::blk::BlockDevice;

pub struct SectorPtr<T: Pod> {
    sector: usize,
    offset: usize,
    _marker: core::marker::PhantomData<T>,
    blk_device: Weak<dyn BlockDevice>,
}

impl<T: Pod> SectorPtr<T> {
    pub fn new(sector: usize, offset: usize, blk_device: &Arc<dyn BlockDevice>) -> Self {
        SectorPtr {
            sector,
            offset,
            _marker: core::marker::PhantomData,
            blk_device: Arc::downgrade(blk_device),
        }
    }

    pub fn read(&self) -> T {
        let blk_device = self
            .blk_device
            .upgrade()
            .expect("Block device has been dropped");
        blk_device.read_val_offset::<T>(self.sector, self.offset)
    }
}
