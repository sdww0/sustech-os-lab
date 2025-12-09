use core::{mem::offset_of, sync::atomic::fence};

use align_ext::AlignExt;
use alloc::{sync::Arc, vec::Vec};
use log::debug;
use ostd::{
    Pod,
    io::IoMem,
    mm::{
        DmaCoherent, DmaStream, FrameAllocOptions, HasDaddr, HasSize, PAGE_SIZE, PodOnce, Segment,
        VmIoOnce,
    },
};

use crate::drivers::{
    utils::DmaSlice,
    virtio::mmio::{VirtioMmioLayout, VirtioMmioTransport},
};

pub struct Virtqueue {
    descriptors: Vec<Arc<DescriptorPtr>>,

    available_ring: AvailRingPtr,

    used_ring: UsedRingPtr,

    notify: IoMem,

    queue_index: u32,

    queue_size: u16,
    /// The number of used descriptors
    used_desc: u16,
    /// The head of descriptors
    head: u16,
    /// The next available slot in `available_ring`
    next_avail: u16,
    /// The last used index we have processed
    last_used_idx: u16,
}

impl Virtqueue {
    pub fn new(queue_index: u32, mmio_transport: &VirtioMmioTransport) -> Option<Self> {
        // We only support legacy device for now.
        assert!(mmio_transport.is_legacy());

        let queue_size = QUEUE_SIZE;
        let frames = legacy_queue_size_to_frames(queue_size);

        // First frame: (descriptors + available ring)
        // Second frame: (used ring)
        let (descriptor_frame, used_ring_frame) = frames.split(PAGE_SIZE);

        let desc_dma = Arc::new(DmaCoherent::map(descriptor_frame.into(), true).unwrap());
        let used_ring_dma = Arc::new(DmaCoherent::map(used_ring_frame.into(), true).unwrap());
        debug!(
            "Virtqueue {}: Descriptor DMA at {:#x}, size {}",
            queue_index,
            desc_dma.daddr(),
            desc_dma.size()
        );
        debug!(
            "Virtqueue {}: Used Ring DMA at {:#x}, size {}",
            queue_index,
            used_ring_dma.daddr(),
            used_ring_dma.size()
        );

        let descriptors = (0..queue_size)
            .map(|i| {
                Arc::new(DescriptorPtr::new(
                    desc_dma.clone(),
                    i * size_of::<Descriptor>(),
                ))
            })
            .collect::<Vec<_>>();

        for descriptor_idx in 0..queue_size {
            // Link the descriptors
            let next_descriptor_idx = (descriptor_idx + 1) % queue_size;
            descriptors[descriptor_idx].set_next(next_descriptor_idx as u16);
        }

        let notify_start = offset_of!(VirtioMmioLayout, queue_notify);
        mmio_transport.enable_queue(
            queue_index,
            queue_size as _,
            &desc_dma,
            &desc_dma,
            &used_ring_dma,
        );

        let queue = Self {
            descriptors,
            available_ring: AvailRingPtr {
                dma: desc_dma,
                offset: queue_size * size_of::<Descriptor>(),
            },
            used_ring: UsedRingPtr::new(used_ring_dma),
            notify: mmio_transport
                .layout_io_mem()
                .slice(notify_start..(notify_start + size_of::<u32>())),
            queue_index,
            queue_size: queue_size as _,
            used_desc: 0,
            head: 0,
            next_avail: 0,
            last_used_idx: 0,
        };

        Some(queue)
    }

    /// Sends requests to device, return Ok(start_head) if success.
    pub fn send_request(&mut self, requests: &[&dyn VirtqueueRequest]) -> Option<u16> {
        let total_requests = requests.len();
        assert!(total_requests + self.used_desc as usize <= self.queue_size as usize);

        // 1. Config the descriptors
        let start_head = self.head;
        let mut end_head = start_head;
        for request in requests {
            let desc = &self.descriptors[self.head as usize];
            let mut flags = DescFlags::NEXT;
            if request.device_writable() {
                flags |= DescFlags::WRITE;
            }
            // Set address and length
            desc.set_desc(request.daddr() as _, request.len() as _);
            desc.set_flags(flags);
            end_head = self.head;
            self.head = desc.next();
            debug!(
                "Virtqueue {}: descriptor {} addr {:#x}, len {}, flags {:?}, next {}",
                self.queue_index,
                end_head,
                request.daddr(),
                request.len(),
                flags,
                self.head
            );
        }
        // Remove the NEXT flags of the end head
        {
            let desc = &self.descriptors[end_head as usize];
            let mut flags = desc.flags();
            flags.remove(DescFlags::NEXT);
            desc.set_flags(flags);
            debug!(
                "Virtqueue {}: Remove the next flags of descriptor {}",
                self.queue_index, end_head
            );
        }

        // 2. Setup the available ring
        let slot = self.next_avail & (self.queue_size - 1);
        self.available_ring.set_ring(slot, start_head);
        self.next_avail = self.next_avail.wrapping_add(1);
        self.available_ring.set_next_avail(self.next_avail);
        debug!(
            "Virtqueue {}: send_request with {} descriptors, next avail idx {}",
            self.queue_index, total_requests, self.next_avail
        );

        fence(core::sync::atomic::Ordering::SeqCst);

        self.used_desc += total_requests as u16;
        Some(start_head)
    }

    /// Notify the device that there are new available requests.
    pub fn notify_device(&self) {
        self.notify.write_once::<u32>(0, &self.queue_index).unwrap();
    }

    pub fn should_notify(&self) -> bool {
        self.used_ring.should_notify()
    }

    /// Gets one finished request.
    ///
    /// Return (start_head, bytes_written)
    pub fn pop_finish_request(&mut self) -> Option<(u16, u32)> {
        if !self.can_pop() {
            return None;
        }

        let last_used_ring_idx = self.last_used_idx & (self.queue_size - 1);
        let used_elem = self.used_ring.get_used_elem(last_used_ring_idx);
        self.recycle_descriptors(used_elem.id as u16);

        self.last_used_idx = self.last_used_idx.wrapping_add(1);

        Some((used_elem.id as u16, used_elem.len))
    }

    /// Checks if there is finished request.
    pub fn can_pop(&self) -> bool {
        let used_idx: u16 = self.used_ring.idx();
        used_idx != self.last_used_idx
    }

    /// Recycles the descriptors starting from `start_head`.
    fn recycle_descriptors(&mut self, mut start_head: u16) {
        let current_free_head = self.head;
        self.head = start_head;

        loop {
            let desc = &self.descriptors[start_head as usize];
            desc.set_desc(0, 0);
            self.used_desc -= 1;

            let flags = desc.flags();
            if flags.contains(DescFlags::NEXT) {
                // Not the end yet
                desc.set_flags(DescFlags::empty());
                start_head = desc.next();
            } else {
                // Reached the end, link the last descriptor to current_free_head
                desc.set_flags(DescFlags::empty());
                self.descriptors[current_free_head as usize].set_next(start_head);
                break;
            }
        }
    }
}

pub trait VirtqueueRequest {
    fn daddr(&self) -> usize;

    fn len(&self) -> usize;

    fn device_writable(&self) -> bool;
}

pub struct VirtqueueCoherentRequest<'a> {
    bind_dma: &'a Arc<DmaCoherent>,
    offset: usize,
    len: usize,
    device_writable: bool,
}

impl<'a> VirtqueueCoherentRequest<'a> {
    pub fn from_dma_slice(
        slice: &'a DmaSlice<impl Pod, DmaCoherent>,
        device_writable: bool,
    ) -> Self {
        Self::new(slice.dma(), slice.offset(), slice.size(), device_writable)
    }

    pub fn new(
        bind_dma: &'a Arc<DmaCoherent>,
        offset: usize,
        len: usize,
        device_writable: bool,
    ) -> Self {
        assert!(offset + len < PAGE_SIZE);

        Self {
            bind_dma,
            offset,
            len,
            device_writable,
        }
    }
}

impl VirtqueueRequest for VirtqueueCoherentRequest<'_> {
    fn daddr(&self) -> usize {
        self.bind_dma.daddr() + self.offset
    }

    fn len(&self) -> usize {
        self.len
    }

    fn device_writable(&self) -> bool {
        self.device_writable
    }
}

pub struct VirtqueueStreamRequest<'a> {
    bind_dma: &'a Arc<DmaStream>,
    offset: usize,
    len: usize,
    device_writable: bool,
}

impl<'a> VirtqueueStreamRequest<'a> {
    pub fn from_dma_slice(slice: &'a DmaSlice<impl Pod, DmaStream>, device_writable: bool) -> Self {
        Self::new(slice.dma(), slice.offset(), slice.size(), device_writable)
    }

    pub fn new(
        bind_dma: &'a Arc<DmaStream>,
        offset: usize,
        len: usize,
        device_writable: bool,
    ) -> Self {
        assert!(offset + len < PAGE_SIZE);

        Self {
            bind_dma,
            offset,
            len,
            device_writable,
        }
    }
}

impl VirtqueueRequest for VirtqueueStreamRequest<'_> {
    fn daddr(&self) -> usize {
        self.bind_dma.daddr() + self.offset
    }

    fn len(&self) -> usize {
        self.len
    }

    fn device_writable(&self) -> bool {
        self.device_writable
    }
}

const QUEUE_SIZE: usize = 64;

/// Allocates a contiguous memory region for a legacy virtqueue with the given size in number of descriptors.
///
/// For legacy device, the structure is organized as follows:
/// [Descriptor Table] [Available Ring] [padding to 4096] [Used Ring]
fn legacy_queue_size_to_frames(queue_size: usize) -> Segment<()> {
    let descriptor_table_size = core::mem::size_of::<Descriptor>() * queue_size;
    let avail_ring_size = core::mem::size_of::<AvailRing>();
    let used_ring_size = core::mem::size_of::<UsedRing>();

    let total_size = (descriptor_table_size + avail_ring_size).align_up(4096) + used_ring_size;
    FrameAllocOptions::new()
        .alloc_segment(total_size.align_up(PAGE_SIZE) / PAGE_SIZE)
        .unwrap()
}

#[repr(C, align(16))]
#[derive(Debug, Default, Copy, Clone, Pod)]
pub struct Descriptor {
    addr: u64,
    len: u32,
    flags: DescFlags,
    next: u16,
}

bitflags::bitflags! {
    /// Descriptor flags
    #[derive(Pod, Default)]
    #[repr(C)]
    struct DescFlags: u16 {
        const NEXT = 1;
        const WRITE = 2;
        const INDIRECT = 4;
    }
}

impl PodOnce for DescFlags {}

#[repr(C, align(2))]
#[derive(Debug, Copy, Clone, Pod)]
pub struct AvailRing {
    flags: u16,
    idx: u16,
    ring: [u16; QUEUE_SIZE],
    used_event: u16,
}

#[repr(C, align(4))]
#[derive(Debug, Copy, Clone, Pod)]
pub struct UsedRing {
    flags: u16,
    idx: u16,
    ring: [UsedElem; QUEUE_SIZE],
    avail_event: u16,
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, Pod)]
pub struct UsedElem {
    id: u32,
    len: u32,
}

impl PodOnce for UsedElem {}

struct AvailRingPtr {
    dma: Arc<DmaCoherent>,
    offset: usize,
}

impl AvailRingPtr {
    fn set_ring(&self, slot: u16, descriptor_head: u16) {
        self.write_once(
            offset_of!(AvailRing, ring) + slot as usize * size_of::<u16>(),
            &descriptor_head,
        )
        .unwrap();
    }

    fn set_next_avail(&self, next_slot: u16) {
        self.write_once(offset_of!(AvailRing, idx), &next_slot)
            .unwrap();
    }
}

impl VmIoOnce for AvailRingPtr {
    fn read_once<T: PodOnce>(&self, offset: usize) -> ostd::Result<T> {
        self.dma.read_once(offset + self.offset)
    }

    fn write_once<T: PodOnce>(&self, offset: usize, new_val: &T) -> ostd::Result<()> {
        self.dma.write_once(offset + self.offset, new_val)
    }
}

struct DescriptorPtr {
    dma: Arc<DmaCoherent>,
    offset: usize,
}

impl DescriptorPtr {
    fn new(dma: Arc<DmaCoherent>, offset: usize) -> Self {
        Self { dma, offset }
    }

    fn set_desc(&self, addr: u64, len: u32) {
        self.dma
            .write_once(self.offset + offset_of!(Descriptor, addr), &addr)
            .unwrap();
        self.dma
            .write_once(self.offset + offset_of!(Descriptor, len), &len)
            .unwrap();
    }

    fn set_next(&self, next: u16) {
        self.dma
            .write_once(self.offset + offset_of!(Descriptor, next), &next)
            .unwrap();
    }

    fn set_flags(&self, flags: DescFlags) {
        self.dma
            .write_once(self.offset + offset_of!(Descriptor, flags), &flags)
            .unwrap();
    }

    fn next(&self) -> u16 {
        self.dma
            .read_once(self.offset + offset_of!(Descriptor, next))
            .unwrap()
    }

    fn flags(&self) -> DescFlags {
        self.dma
            .read_once(self.offset + offset_of!(Descriptor, flags))
            .unwrap()
    }
}

struct UsedRingPtr {
    dma: Arc<DmaCoherent>,
}

impl UsedRingPtr {
    fn new(dma: Arc<DmaCoherent>) -> Self {
        Self { dma }
    }

    fn idx(&self) -> u16 {
        self.dma.read_once(offset_of!(UsedRing, idx)).unwrap()
    }

    fn should_notify(&self) -> bool {
        let flags: u16 = self.dma.read_once(offset_of!(UsedRing, flags)).unwrap();
        flags & 1 == 0
    }

    fn get_used_elem(&self, index: u16) -> UsedElem {
        self.dma
            .read_once(offset_of!(UsedRing, ring) + index as usize * size_of::<UsedElem>())
            .unwrap()
    }
}
