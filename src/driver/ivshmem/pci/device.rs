use alloc::sync::Arc;
use core::fmt::Debug;
use ostd::{
    bus::{
        pci::{bus::PciDevice, cfg_space::MemoryBar, common_device::PciCommonDevice, PciDeviceId},
        BusProbeError,
    },
    early_println,
    mm::VmIo,
};
pub struct IvSharedMemoryDevice {
    common_device: PciCommonDevice,
    shared_memory_bar: Arc<MemoryBar>,
}
#[allow(clippy::result_large_err)]
impl IvSharedMemoryDevice {
    pub(crate) fn new(
        common_device: PciCommonDevice,
    ) -> Result<Self, (BusProbeError, PciCommonDevice)> {
        let shared_memory_bar = {
            let bar_manager = common_device.bar_manager();
            let Some(bar) = bar_manager.bar(2) else {
                return Err((BusProbeError::ConfigurationSpaceError, common_device));
            };
            match bar {
                ostd::bus::pci::cfg_space::Bar::Memory(bar) => bar.clone(),
                ostd::bus::pci::cfg_space::Bar::Io(_) => {
                    return Err((BusProbeError::ConfigurationSpaceError, common_device))
                }
            }
        };

        early_println!(
            "Shared memory base:{:x?}, size:{:x?}",
            shared_memory_bar.base(),
            shared_memory_bar.size()
        );
        Ok(Self {
            common_device,
            shared_memory_bar,
        })
    }
}
impl PciDevice for IvSharedMemoryDevice {
    fn device_id(&self) -> PciDeviceId {
        *self.common_device.device_id()
    }
}
impl Debug for IvSharedMemoryDevice {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("IvSharedMemoryDevice")
            .field("common_device", &self.common_device)
            .finish()
    }
}
impl super::super::IvSharedMemoryDevice for IvSharedMemoryDevice {
    fn read_bytes(&self, offset: usize, data: &mut [u8]) -> Result<(), ostd::Error> {
        self.shared_memory_bar.io_mem().read_bytes(offset, data)
    }
    fn write_bytes(&self, offset: usize, data: &[u8]) -> Result<(), ostd::Error> {
        self.shared_memory_bar.io_mem().write_bytes(offset, data)
    }
    fn size(&self) -> usize {
        self.shared_memory_bar.size() as usize
    }
}
