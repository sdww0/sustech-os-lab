use alloc::vec::Vec;
use ostd::{
    mm::{DmaStream, FrameAllocOptions, VmIo, VmWriter},
    sync::Mutex,
};
use spin::Once;

use crate::drivers::utils::{DmaSlice, DmaSliceAlloc};

pub const SECTOR_SIZE: usize = 512;

static DMA_ALLOCATOR: Once<Mutex<DmaSliceAlloc<[u8; SECTOR_SIZE], DmaStream>>> = Once::new();

pub trait BlockDevice: Send + Sync {
    fn read_block(&self, req: &mut BioRequest);

    fn write_block(&self, req: &BioRequest);
}

impl dyn BlockDevice {
    pub fn read_to_vm_writer(&self, index: usize, num_sectors: usize, writer: &mut VmWriter) {
        let mut request = BioRequest::new(index, num_sectors);
        self.read_block(&mut request);

        for data in request.data.iter() {
            data.read(0, &mut *writer).unwrap();
        }
    }

    pub fn read_val_offset<T: ostd::Pod>(&self, index: usize, offset: usize) -> T {
        assert!(core::mem::size_of::<T>() + offset <= SECTOR_SIZE);
        let mut request = BioRequest::new(index, 1);
        self.read_block(&mut request);
        request.data.pop().unwrap().read_val(offset).unwrap()
    }

    pub fn write_val_offset<T: ostd::Pod>(&self, index: usize, offset: usize, val: &T) {
        assert!(core::mem::size_of::<T>() + offset <= SECTOR_SIZE);
        let request = BioRequest::new(index, 1);
        request.data[0].write_val(offset, val).unwrap();
        self.write_block(&request);
    }

    pub fn read_one(&self, index: usize) -> DmaSlice<DmaStream> {
        let mut request = BioRequest::new(index, 1);
        self.read_block(&mut request);
        request.data.pop().unwrap()
    }

    pub fn write_one(&self, index: usize, data: &[u8; SECTOR_SIZE]) {
        let request = BioRequest::new(index, 1);
        request.data[0].write_bytes(0, &data.as_ref()).unwrap();
        self.write_block(&request);
    }

    pub fn read_val<T: ostd::Pod>(&self, index: usize) -> T {
        assert!(core::mem::size_of::<T>() <= SECTOR_SIZE);
        let mut request = BioRequest::new(index, 1);
        self.read_block(&mut request);
        request.data.pop().unwrap().read_val(0).unwrap()
    }

    pub fn write_val<T: ostd::Pod>(&self, index: usize, val: &T) {
        assert!(core::mem::size_of::<T>() <= SECTOR_SIZE);
        let request = BioRequest::new(index, 1);
        request.data[0].write_val(0, val).unwrap();
        self.write_block(&request);
    }
}

pub struct BioRequest {
    index: usize,
    pub data: Vec<DmaSlice<DmaStream>>,
}

impl BioRequest {
    pub fn new(index: usize, num_sectors: usize) -> Self {
        let mut data = Vec::with_capacity(num_sectors);
        let mut dma_allocator = DMA_ALLOCATOR.get().unwrap().lock();
        for _ in 0..num_sectors {
            data.push(dma_allocator.alloc().unwrap());
        }

        Self { index, data }
    }

    pub fn data_slices_mut(&mut self) -> &mut [DmaSlice<DmaStream>] {
        &mut self.data
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn num_sectors(&self) -> usize {
        self.data.len()
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
