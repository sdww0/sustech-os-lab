use alloc::boxed::Box;
use alloc::vec::Vec;
use log::{debug, error};
use ostd::{
    Pod,
    mm::{DmaCoherent, FrameAllocOptions, VmIo},
    sync::{LocalIrqDisabled, SpinLock},
};

use crate::drivers::{
    blk::BioRequest,
    virtio::queue::{VirtqueueCoherentRequest, VirtqueueRequest, VirtqueueStreamRequest},
};
use crate::drivers::{
    blk::BlockDevice,
    utils::DmaSliceAlloc,
    virtio::{mmio::VirtioMmioTransport, queue::Virtqueue},
};

pub struct VirtioBlkDevice {
    transport: VirtioMmioTransport,
    config: VirtioBlkConfig,
    request_queue: SpinLock<Virtqueue, LocalIrqDisabled>,

    request_alloc: SpinLock<DmaSliceAlloc<BlockReq, DmaCoherent>, LocalIrqDisabled>,
    resp_alloc: SpinLock<DmaSliceAlloc<BlockResp, DmaCoherent>, LocalIrqDisabled>,
}

impl VirtioBlkDevice {
    pub fn new(transport: VirtioMmioTransport) -> Self {
        let queue = Virtqueue::new(0, &transport).unwrap();
        let request_dma = DmaCoherent::map(
            FrameAllocOptions::new().alloc_segment(1).unwrap().into(),
            false,
        )
        .unwrap();
        let resp_dma = DmaCoherent::map(
            FrameAllocOptions::new().alloc_segment(1).unwrap().into(),
            false,
        )
        .unwrap();

        let config_io_mem = transport.config_space();
        let blk_config: VirtioBlkConfig = config_io_mem.read_val(0).unwrap();

        debug!("Virtio Block Device config: {:#?}", blk_config);

        transport.finish_init();

        Self {
            transport,
            request_queue: SpinLock::new(queue),
            request_alloc: SpinLock::new(DmaSliceAlloc::new(request_dma)),
            resp_alloc: SpinLock::new(DmaSliceAlloc::new(resp_dma)),
            config: blk_config,
        }
    }
}

impl BlockDevice for VirtioBlkDevice {
    fn read_block(&self, bio_request: &mut BioRequest) {
        let req_dma = self.request_alloc.lock().alloc().unwrap();
        let resp_dma = self.resp_alloc.lock().alloc().unwrap();

        let req = BlockReq {
            type_: ReqType::In as _,
            reserved: 0,
            sector: bio_request.index() as u64,
        };
        req_dma.write_no_offset_val(&req).unwrap();

        let resp = BlockResp::default();
        resp_dma.write_no_offset_val(&resp).unwrap();

        // Construct Requests
        let mut requests: Vec<Box<dyn VirtqueueRequest>> =
            Vec::with_capacity(bio_request.num_sectors() + 2);
        requests.push(Box::new(VirtqueueCoherentRequest::from_dma_slice(
            &req_dma, false,
        )));
        for data in bio_request.data_slices_mut().iter_mut() {
            let stream_req = VirtqueueStreamRequest::from_dma_slice(data, true);
            requests.push(Box::new(stream_req));
        }
        requests.push(Box::new(VirtqueueCoherentRequest::from_dma_slice(
            &resp_dma, true,
        )));
        let queue_requests: Vec<&dyn VirtqueueRequest> =
            requests.iter().map(|r| r.as_ref()).collect();

        // Send requests
        let mut queue = self.request_queue.lock();
        queue.send_request(queue_requests.as_ref()).unwrap();
        // Notify the device
        if queue.should_notify() {
            queue.notify_device();
        }

        // Wait for completion
        while !queue.can_pop() {
            core::hint::spin_loop();
        }

        queue.pop_finish_request();

        // Read response
        let resp_read: BlockResp = resp_dma.read_no_offset_val().unwrap();
        if resp_read.status != RespStatus::Ok as u8 {
            error!("Block device read error: {:?}", resp_read.status);
        }
    }

    fn write_block(&self, bio_request: &BioRequest) {}
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod)]
struct BlockReq {
    pub type_: u32,
    pub reserved: u32,
    pub sector: u64,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod)]
struct BlockResp {
    pub status: u8,
}

impl Default for BlockResp {
    fn default() -> Self {
        Self {
            status: RespStatus::NotReady as _,
        }
    }
}

#[repr(u32)]
#[derive(Debug, Copy, Clone)]
pub enum ReqType {
    In = 0,
    Out = 1,
    Flush = 4,
    GetId = 8,
    Discard = 11,
    WriteZeroes = 13,
}

#[repr(u8)]
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum RespStatus {
    Ok = 0,
    IoErr = 1,
    Unsupported = 2,
    NotReady = 3,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod)]
struct VirtioBlkConfig {
    capacity: u64,
    size_max: u32,
    seg_max: u32,
    geometry_cylinders: u16,
    geometry_heads: u8,
    geometry_sectors: u8,
    blk_size: u32,
    physical_block_exp: u8,
    alignment_offset: u8,
    min_io_size: u16,
    opt_io_size: u32,
}
