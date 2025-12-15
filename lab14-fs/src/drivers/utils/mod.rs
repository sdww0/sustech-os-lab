use alloc::sync::Arc;
use id_alloc::IdAlloc;
use ostd::{
    Pod,
    mm::{HasDaddr, HasSize, VmIo},
};

pub struct DmaSlice<T: Pod, D: VmIo + HasDaddr + HasSize> {
    dma: Arc<D>,
    offset: usize,
    _phantom: core::marker::PhantomData<T>,
}

impl<T: Pod, D: VmIo + HasDaddr + HasSize> DmaSlice<T, D> {
    pub fn write(&self, data: &T) {
        self.dma
            .write_val(self.offset, data)
            .expect("Failed to write to DmaCoherent");
    }

    pub fn read(&self) -> T {
        self.dma
            .read_val(self.offset)
            .expect("Failed to read from DmaCoherent")
    }

    pub fn dma(&self) -> &Arc<D> {
        &self.dma
    }

    pub fn offset(&self) -> usize {
        self.offset
    }

    pub fn size(&self) -> usize {
        size_of::<T>()
    }
}

impl<T: Pod + Send + Sync, D: VmIo + HasDaddr + HasSize> VmIo for DmaSlice<T, D> {
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

    pub fn alloc(&mut self) -> Option<DmaSlice<T, D>> {
        let alloc_index = self.allocator.alloc()?;
        let offset = alloc_index * size_of::<T>();

        Some(DmaSlice {
            dma: self.dma.clone(),
            offset,
            _phantom: core::marker::PhantomData,
        })
    }

    pub fn dealloc(&mut self, slice: DmaSlice<T, D>) {
        let offset = slice.offset();
        let index = offset / size_of::<T>();
        self.allocator.free(index);
    }
}
