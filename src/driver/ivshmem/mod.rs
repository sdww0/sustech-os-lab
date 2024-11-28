pub mod pci;

use core::{any::Any, fmt::Debug};

use alloc::sync::Arc;
use log::info;
use spin::Once;

static IVSHMEM: Once<Arc<dyn IvSharedMemoryDevice>> = Once::new();

pub trait IvSharedMemoryDevice: Send + Sync + Any + Debug {
    fn read_bytes(&self, offset: usize, data: &mut [u8]) -> Result<(), ostd::Error>;
    fn write_bytes(&self, offset: usize, data: &[u8]) -> Result<(), ostd::Error>;
    fn size(&self) -> usize;
}

pub fn init() {
    pci::init();
    test_device();
}

pub fn ivshmem_device() -> Option<&'static Arc<dyn IvSharedMemoryDevice>> {
    IVSHMEM.get()
}

pub fn register_device(device: Arc<dyn IvSharedMemoryDevice>) {
    info!("Registering ivshmem device, size: {:x}", device.size());
    IVSHMEM.call_once(|| device);
}

fn test_device() {
    let Some(device) = ivshmem_device() else {
        return;
    };
    let hello_world = b"Hello World!\n";
    device.write_bytes(0, hello_world).unwrap();
}
