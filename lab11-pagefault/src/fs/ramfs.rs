use alloc::{
    collections::btree_map::BTreeMap,
    string::{String, ToString},
    sync::Arc,
    vec::Vec,
};
use ostd::{
    mm::{FallibleVmRead, FallibleVmWrite, VmReader, VmWriter},
    sync::{Mutex, RwMutex},
};

use crate::error::{Errno, Error, Result};
use crate::fs::{Inode, InodeMeta, InodeType};

pub struct RamInode {
    inner: Inner,
    metadata: InodeMeta,
}

enum Inner {
    File(Mutex<Vec<u8>>),
    Directory(RwMutex<BTreeMap<String, Arc<RamInode>>>),
}

impl RamInode {
    fn new_file() -> Arc<Self> {
        Arc::new(RamInode {
            inner: Inner::File(Mutex::new(Vec::new())),
            metadata: InodeMeta {
                size: 0,
                atime: core::time::Duration::new(0, 0),
                mtime: core::time::Duration::new(0, 0),
                ctime: core::time::Duration::new(0, 0),
            },
        })
    }

    fn new_directory() -> Arc<Self> {
        Arc::new(RamInode {
            inner: Inner::Directory(RwMutex::new(BTreeMap::new())),
            metadata: InodeMeta {
                size: 0,
                atime: core::time::Duration::new(0, 0),
                mtime: core::time::Duration::new(0, 0),
                ctime: core::time::Duration::new(0, 0),
            },
        })
    }
}

impl Inode for RamInode {
    fn read_at(&self, offset: usize, mut writer: ostd::mm::VmWriter) -> Result<usize> {
        let Inner::File(data) = &self.inner else {
            return Err(Error::new(Errno::EISDIR));
        };

        let data = data.lock();
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
        let Inner::File(data) = &self.inner else {
            return Err(Error::new(Errno::EISDIR));
        };

        let mut data = data.lock();
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
        match &self.inner {
            Inner::File(data) => data.lock().len(),
            Inner::Directory(_) => 12,
        }
    }

    fn metadata(&self) -> &InodeMeta {
        &self.metadata
    }

    fn lookup(&self, name: &str) -> Result<Arc<dyn Inode>> {
        let Inner::Directory(ref entries) = self.inner else {
            return Err(Error::new(Errno::ENOTDIR));
        };

        let entries = entries.read();
        let inode = entries.get(name).ok_or(Error::new(Errno::ENOENT))?;

        Ok(inode.clone())
    }

    fn create(&self, name: &str, type_: InodeType) -> Result<Arc<dyn Inode>> {
        let Inner::Directory(ref entries) = self.inner else {
            return Err(Error::new(Errno::ENOTDIR));
        };

        let inode = match type_ {
            InodeType::File => RamInode::new_file(),
            InodeType::Directory => RamInode::new_directory(),
            InodeType::SymbolLink => todo!(),
        };

        entries.write().insert(name.to_string(), inode.clone());

        Ok(inode)
    }

    fn read_link(&self) -> Result<String> {
        todo!()
    }

    fn write_link(&self, _target: &str) -> Result<()> {
        todo!()
    }

    fn typ(&self) -> InodeType {
        match &self.inner {
            Inner::Directory(_) => InodeType::Directory,
            Inner::File(_) => InodeType::File,
        }
    }
}

pub struct RamFS {
    root: Arc<RamInode>,
}

impl RamFS {
    pub fn new() -> Self {
        RamFS {
            root: RamInode::new_directory(),
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
