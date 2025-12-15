#![expect(dead_code)]
#![expect(unused_variables)]

use alloc::{sync::Arc, vec::Vec};
use core::ffi::CStr;
use ostd::{
    early_println,
    mm::{DmaDirection, DmaStream, FrameAllocOptions},
};
use spin::{Mutex, Once};

use crate::drivers::{blk::BlockDevice, utils::DmaSliceAlloc};

pub mod blk;
pub mod utils;
pub mod virtio;

pub static BLOCK_DEVICES: Once<Mutex<Vec<Arc<dyn BlockDevice>>>> = Once::new();

pub fn init() {
    BLOCK_DEVICES.call_once(|| Mutex::new(Vec::new()));
    virtio::init();
    blk::init();
    test_blk_device_read();
}

fn test_blk_device_read() {
    let block_devices = BLOCK_DEVICES.get().unwrap().lock();

    let test_dma = DmaStream::map(
        FrameAllocOptions::new().alloc_segment(1).unwrap().into(),
        DmaDirection::Bidirectional,
        true,
    )
    .unwrap();

    let mut test_dma_slice_alloc = DmaSliceAlloc::new(test_dma);

    early_println!("Testing block device read...");
    for blk_device in block_devices.iter() {
        let mut dma_slice = test_dma_slice_alloc.alloc().unwrap();
        blk_device.read_block(0, &mut dma_slice);

        let data = dma_slice.read();
        let cstr = CStr::from_bytes_until_nul(&data).unwrap();
        early_println!("Read string: {}", cstr.to_str().unwrap());
    }

    early_println!("Testing block device write...");
    let bytes = b"Hello, Virtio Block Device!";
    for blk_device in block_devices.iter() {
        let mut dma_slice = test_dma_slice_alloc.alloc().unwrap();
        let mut buffer = [0; 512];
        buffer[..bytes.len()].copy_from_slice(bytes);
        dma_slice.write(&buffer);

        blk_device.write_block(0, &mut dma_slice);
    }
}
