use ostd::{
    mm::{DmaStream, FrameAllocOptions, VmIo},
    sync::Mutex,
};
use spin::Once;

use crate::drivers::utils::{DmaSlice, DmaSliceAlloc};

pub const SECTOR_SIZE: usize = 512;

static DMA_ALLOCATOR: Once<Mutex<DmaSliceAlloc<[u8; SECTOR_SIZE], DmaStream>>> = Once::new();

pub trait BlockDevice: Send + Sync {
    fn read_block(&self, index: usize, data: &mut DmaSlice<[u8; SECTOR_SIZE], DmaStream>);

    fn write_block(&self, index: usize, data: &DmaSlice<[u8; SECTOR_SIZE], DmaStream>);
}

impl dyn BlockDevice {
    pub fn read_val<T: ostd::Pod>(&self, index: usize) -> T {
        assert!(core::mem::size_of::<T>() <= SECTOR_SIZE);

        let mut buf = DMA_ALLOCATOR.get().unwrap().lock().alloc().unwrap();
        self.read_block(index, &mut buf);
        buf.read_val(0).unwrap()
    }
}

pub(super) fn init() {
    const POOL_SIZE: usize = 128;
    let segment = FrameAllocOptions::new().alloc_segment(POOL_SIZE).unwrap();
    let dma_allocator = DmaSliceAlloc::<[u8; SECTOR_SIZE], DmaStream>::new(
        DmaStream::map(segment.into(), ostd::mm::DmaDirection::Bidirectional, false).unwrap(),
    );
    DMA_ALLOCATOR.call_once(|| Mutex::new(dma_allocator));
}
