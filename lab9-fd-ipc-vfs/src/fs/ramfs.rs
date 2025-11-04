use alloc::{string::String, sync::Arc, vec::Vec};
use ostd::{
    mm::{FallibleVmRead, FallibleVmWrite, VmReader, VmWriter},
    sync::Mutex,
};

use crate::error::{Errno, Error, Result};
use crate::fs::{Inode, InodeMeta};

pub struct RamInode {
    data: Mutex<Vec<u8>>,
    metadata: InodeMeta,
}

impl Inode for RamInode {
    fn read_at(&self, offset: usize, mut writer: ostd::mm::VmWriter) -> Result<usize> {
        let data = self.data.lock();
        if offset >= data.len() {
            return Ok(0);
        }

        let read_len = core::cmp::min(data.len() - offset, writer.avail());
        writer
            .write_fallible(&mut VmReader::from(
                &(*data.as_slice())[offset..offset + read_len],
            ))
            .unwrap();
        Ok(read_len)
    }

    fn write_at(&self, offset: usize, mut reader: ostd::mm::VmReader) -> Result<usize> {
        let mut data = self.data.lock();
        if offset > data.len() {
            // Fill the gap with zeros
            data.resize(offset, 0);
        }

        // Expand the data vector if necessary
        if offset + reader.remain() > data.len() {
            data.resize(offset + reader.remain(), 0);
        }

        let write_len = core::cmp::min(data.len() - offset, reader.remain());
        reader
            .read_fallible(&mut VmWriter::from(
                &mut (*data.as_mut_slice())[offset..offset + write_len],
            ))
            .unwrap();
        Ok(write_len)
    }

    fn size(&self) -> usize {
        self.data.lock().len()
    }

    fn metadata(&self) -> &InodeMeta {
        &self.metadata
    }

    fn open(self: Arc<Self>, name: String) -> Arc<dyn Inode> {
        self
    }
}

pub struct RamFS {
    root: Arc<RamInode>,
}

impl RamFS {
    pub fn new() -> Self {
        RamFS {
            root: Arc::new(RamInode {
                data: Mutex::new(Vec::new()),
                metadata: InodeMeta {
                    size: 0,
                    atime: core::time::Duration::new(0, 0),
                    mtime: core::time::Duration::new(0, 0),
                    ctime: core::time::Duration::new(0, 0),
                },
            }),
        }
    }
}

impl crate::fs::FileSystem for RamFS {
    fn name(&self) -> &str {
        "ramfs"
    }

    fn root_inode(&self) -> Arc<dyn Inode> {
        self.root.clone()
    }
}
