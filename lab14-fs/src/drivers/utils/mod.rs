use alloc::sync::Arc;
use id_alloc::IdAlloc;
use ostd::{
    Pod,
    mm::{HasDaddr, HasSize, VmIo},
};

pub struct DmaSlice<D: VmIo + HasDaddr + HasSize> {
    dma: Arc<D>,
    offset: usize,
    size: usize,
}

impl<D: VmIo + HasDaddr + HasSize> DmaSlice<D> {
    pub fn write_no_offset_val<T: Pod>(&self, val: &T) -> ostd::Result<()> {
        self.write_val(0, val)
    }

    pub fn read_no_offset_val<T: Pod>(&self) -> ostd::Result<T> {
        self.read_val(0)
    }

    pub fn dma(&self) -> &Arc<D> {
        &self.dma
    }

    pub fn offset(&self) -> usize {
        self.offset
    }

    pub fn size(&self) -> usize {
        self.size
    }
}

impl<D: VmIo + HasDaddr + HasSize> VmIo for DmaSlice<D> {
    fn read(&self, offset: usize, writer: &mut ostd::mm::VmWriter) -> ostd::Result<()> {
        if offset + writer.avail() > self.size() {
            return Err(ostd::Error::AccessDenied);
        }
        self.dma.read(offset + self.offset, writer)
    }

    fn write(&self, offset: usize, reader: &mut ostd::mm::VmReader) -> ostd::Result<()> {
        if offset + reader.remain() > self.size() {
            return Err(ostd::Error::AccessDenied);
        }
        self.dma.write(offset + self.offset, reader)
    }
}

pub struct DmaSliceAlloc<T: Pod, D: VmIo + HasDaddr + HasSize> {
    dma: Arc<D>,
    allocator: IdAlloc,
    _phantom: core::marker::PhantomData<T>,
}

impl<T: Pod, D: VmIo + HasDaddr + HasSize> DmaSliceAlloc<T, D> {
    pub fn new(dma: D) -> Self {
        let capacity = dma.size() / core::mem::size_of::<T>();

        Self {
            dma: Arc::new(dma),
            allocator: IdAlloc::with_capacity(capacity),
            _phantom: core::marker::PhantomData,
        }
    }

    pub fn alloc(&mut self) -> Option<DmaSlice<D>> {
        let alloc_index = self.allocator.alloc()?;
        let offset = alloc_index * size_of::<T>();

        Some(DmaSlice {
            dma: self.dma.clone(),
            offset,
            size: size_of::<T>(),
        })
    }

    pub fn dealloc(&mut self, slice: DmaSlice<D>) {
        let offset = slice.offset();
        let index = offset / size_of::<T>();
        self.allocator.free(index);
    }
}
