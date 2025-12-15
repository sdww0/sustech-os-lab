pub mod blk;
pub mod mmio;
pub mod queue;

use core::{hint::spin_loop, mem::offset_of};

use alloc::{sync::Arc, vec::Vec};
use ostd::{
    Pod,
    arch::boot::DEVICE_TREE,
    early_println,
    io::IoMem,
    mm::{PodOnce, VmIoOnce},
};

use crate::drivers::virtio::{
    blk::VirtioBlkDevice,
    mmio::{VirtioMmioLayout, VirtioMmioTransport},
};

pub fn init() {
    // We use device tree to initialize virtio devices.
    let device_tree = DEVICE_TREE.get().unwrap();
    let mmio_virtio_nodes = device_tree.all_nodes().filter(|node| {
        node.compatible().is_some_and(|compatible| {
            compatible
                .all()
                .any(|compatible| compatible == "virtio,mmio")
        })
    });

    let mut transports = Vec::new();
    for node in mmio_virtio_nodes {
        let mmio_region = node.reg().unwrap().next().unwrap();
        let start = mmio_region.starting_address as usize;
        let size = mmio_region.size.unwrap();

        let Ok(layout_io_mem) = IoMem::acquire(start..(start + size)) else {
            early_println!("Failed to map Virtio MMIO device at {:#x}", start);
            continue;
        };

        // Read Magic Value
        let magic_value = layout_io_mem.read_once::<u32>(0).unwrap();
        if magic_value != 0x7472_6976 {
            early_println!(
                "Invalid Virtio MMIO magic value {:#x} at {:#x}",
                magic_value,
                start
            );
            continue;
        }

        // Check device id
        let device_id = layout_io_mem.read_once::<u32>(8).unwrap();
        if device_id == 0 {
            continue;
        }

        let version: u32 = layout_io_mem
            .read_once(offset_of!(VirtioMmioLayout, version))
            .unwrap();

        early_println!(
            "Virtio MMIO device found at {:#x} with size {:#x}, device id {}, version: {}",
            start,
            size,
            device_id,
            version
        );

        transports.push(VirtioMmioTransport::new(layout_io_mem));
    }

    // Next, Check if we support the device.
    for transport in transports {
        // Start initialization procedure
        // First, reset device
        transport.set_device_status(DeviceStatus::empty());
        while transport.device_status() != DeviceStatus::empty() {
            spin_loop();
        }

        // Next, set to acknowledge
        transport.set_device_status(DeviceStatus::ACKNOWLEDGE | DeviceStatus::DRIVER);

        // Then, negotiate features
        let device_id = transport.device_id();
        let mut features = transport.device_features();
        // Remove the indirect descriptor feature
        features &= !(1u64 << 28);
        features &= !(1u64 << 29);
        match device_id {
            2 => {
                // Remove the MQ features
                features &= !(1u64 << 12);
            }
            _ => unimplemented!(),
        }
        transport.set_driver_features(features);

        if !transport.is_legacy() {
            transport.set_device_status(
                DeviceStatus::ACKNOWLEDGE | DeviceStatus::DRIVER | DeviceStatus::FEATURES_OK,
            );
        }

        match device_id {
            2 => {
                let blk_device = VirtioBlkDevice::new(transport);

                super::BLOCK_DEVICES
                    .get()
                    .unwrap()
                    .lock()
                    .push(Arc::new(blk_device));
            }
            _ => unimplemented!(),
        }
    }
}

bitflags::bitflags! {
    #[derive(Pod)]
    #[repr(C)]
    pub struct DeviceStatus: u8 {
        const ACKNOWLEDGE           = 1 << 0;
        const DRIVER                = 1 << 1;
        const DRIVER_OK             = 1 << 2;
        const FEATURES_OK           = 1 << 3;
        const DEVICE_NEEDS_RESET    = 1 << 6;
        const FAILED                = 1 << 7;
    }
}

impl PodOnce for DeviceStatus {}
