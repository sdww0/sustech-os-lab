use core::mem::offset_of;

use ostd::{
    Pod,
    io::IoMem,
    mm::{DmaCoherent, HasDaddr, PAGE_SIZE, VmIoOnce},
};

use crate::drivers::virtio::DeviceStatus;

pub struct VirtioMmioTransport {
    layout_io_mem: IoMem,
    is_legacy: bool,
}

impl VirtioMmioTransport {
    pub fn new(layout_io_mem: IoMem) -> Self {
        let version: u32 = layout_io_mem
            .read_once(offset_of!(VirtioMmioLayout, version))
            .unwrap();

        Self {
            layout_io_mem,
            is_legacy: version == 0x1,
        }
    }

    pub fn device_features(&self) -> u64 {
        // Low
        self.layout_io_mem
            .write_once(offset_of!(VirtioMmioLayout, device_features_select), &0u32)
            .unwrap();
        let low = self
            .layout_io_mem
            .read_once::<u32>(offset_of!(VirtioMmioLayout, device_features))
            .unwrap() as u64;

        // High
        self.layout_io_mem
            .write_once(offset_of!(VirtioMmioLayout, device_features_select), &1u32)
            .unwrap();
        let high = self
            .layout_io_mem
            .read_once::<u32>(offset_of!(VirtioMmioLayout, device_features))
            .unwrap() as u64;

        (high << 32) | low
    }

    pub fn set_driver_features(&self, features: u64) {
        let low = (features & 0xFFFF_FFFF) as u32;
        let high = (features >> 32) as u32;

        // Low
        self.layout_io_mem
            .write_once(offset_of!(VirtioMmioLayout, driver_features_select), &0u32)
            .unwrap();
        self.layout_io_mem
            .write_once::<u32>(offset_of!(VirtioMmioLayout, driver_features), &low)
            .unwrap();

        // High
        self.layout_io_mem
            .write_once(offset_of!(VirtioMmioLayout, driver_features_select), &1u32)
            .unwrap();
        self.layout_io_mem
            .write_once::<u32>(offset_of!(VirtioMmioLayout, driver_features), &high)
            .unwrap();
    }

    pub fn set_device_status(&self, status: DeviceStatus) {
        let status: u32 = status.bits as u32;
        self.layout_io_mem
            .write_once(offset_of!(VirtioMmioLayout, status), &status)
            .unwrap();
    }

    pub fn device_status(&self) -> DeviceStatus {
        let status: u32 = self
            .layout_io_mem
            .read_once(offset_of!(VirtioMmioLayout, status))
            .unwrap();
        DeviceStatus::from_bits(status as _).unwrap()
    }

    pub fn device_id(&self) -> u32 {
        self.layout_io_mem
            .read_once::<u32>(offset_of!(VirtioMmioLayout, device_id))
            .unwrap()
    }

    pub fn layout_io_mem(&self) -> &IoMem {
        &self.layout_io_mem
    }

    pub fn is_legacy(&self) -> bool {
        self.is_legacy
    }

    pub fn finish_init(&self) {
        self.set_device_status(
            DeviceStatus::ACKNOWLEDGE
                | DeviceStatus::DRIVER
                | DeviceStatus::FEATURES_OK
                | DeviceStatus::DRIVER_OK,
        );
    }

    pub fn enable_queue(
        &self,
        queue_index: u32,
        queue_size: u16,
        desc: &DmaCoherent,
        avail: &DmaCoherent,
        used: &DmaCoherent,
    ) {
        self.layout_io_mem
            .write_once(offset_of!(VirtioMmioLayout, queue_select), &queue_index)
            .unwrap();

        let queue_num_max: u32 = self
            .layout_io_mem
            .read_once(offset_of!(VirtioMmioLayout, queue_num_max))
            .unwrap();

        assert!(queue_size <= queue_num_max as u16);

        let queue_size = queue_size as u32;

        self.layout_io_mem
            .write_once(offset_of!(VirtioMmioLayout, queue_num), &queue_size)
            .unwrap();

        // We only suppport legacy virtio mmio device for now.
        assert!(self.is_legacy);

        let daddr = desc.daddr() as u32;

        self.layout_io_mem
            .write_once(
                offset_of!(VirtioMmioLayout, legacy_queue_align),
                &(PAGE_SIZE as u32),
            )
            .unwrap();
        self.layout_io_mem
            .write_once(offset_of!(VirtioMmioLayout, legacy_queue_pfn), &daddr)
            .unwrap();
    }
}

/// The memory layout of a Virtio MMIO transport device.
#[derive(Debug, Clone, Copy, Pod)]
#[repr(C)]
pub struct VirtioMmioLayout {
    /// Magic value: "virt" (0x74726976)
    pub magic_value: u32,
    /// Virtio MMIO version
    pub version: u32,
    /// Device ID
    pub device_id: u32,
    /// Vendor ID
    pub vendor_id: u32,
    /// Device features
    pub device_features: u32,
    /// Device features selector
    pub device_features_select: u32,

    _reserved0: [u8; 8],

    /// Driver features
    pub driver_features: u32,
    /// Driver features selector
    pub driver_features_select: u32,

    pub legacy_guest_page_size: u32,

    _reserved1: [u8; 4],

    /// Queue selector
    pub queue_select: u32,

    pub queue_num_max: u32,

    pub queue_num: u32,

    pub legacy_queue_align: u32,
    pub legacy_queue_pfn: u32,

    pub queue_ready: u32,

    _reserved2: [u8; 8],

    pub queue_notify: u32,

    _reserved3: [u8; 12],

    pub interrupt_status: u32,
    pub interrupt_ack: u32,

    _reserved4: [u8; 8],

    pub status: u32,

    _reserved5: [u8; 12],

    pub queue_desc_low: u32,
    pub queue_desc_high: u32,

    _reserved6: [u8; 8],

    pub queue_driver_low: u32,
    pub queue_driver_high: u32,

    _reserved7: [u8; 8],

    pub queue_device_low: u32,
    pub queue_device_high: u32,

    _reserved8: [u8; 84],

    pub config_space: u32,
}
