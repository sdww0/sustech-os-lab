use super::device::IvSharedMemoryDevice;
use alloc::sync::Arc;
use ostd::bus::{
    pci::{
        bus::{PciDevice, PciDriver},
        common_device::PciCommonDevice,
    },
    BusProbeError,
};
#[derive(Debug)]
pub struct IvSharedMemoryDriver {}
impl IvSharedMemoryDriver {
    pub(super) fn new() -> Self {
        IvSharedMemoryDriver {}
    }
}
impl PciDriver for IvSharedMemoryDriver {
    fn probe(
        &self,
        device: PciCommonDevice,
    ) -> Result<Arc<dyn PciDevice>, (BusProbeError, PciCommonDevice)> {
        const IVSHMEM_VENDOR_ID: u16 = 0x1af4;
        const IVSHMEM_DEVICE_ID: u16 = 0x1110;
        if device.device_id().vendor_id != IVSHMEM_VENDOR_ID
            || device.device_id().device_id != IVSHMEM_DEVICE_ID
        {
            return Err((BusProbeError::DeviceNotMatch, device));
        }

        let dev = Arc::new(IvSharedMemoryDevice::new(device)?);
        crate::driver::ivshmem::register_device(dev.clone());
        Ok(dev)
    }
}
