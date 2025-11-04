use alloc::sync::Arc;

use crate::fs::{FileLike, Inode};

pub struct FileInode {
    inode: Arc<dyn Inode>,
}

impl FileInode {
    pub fn new(inode: Arc<dyn Inode>) -> Self {
        Self { inode }
    }
}

impl FileLike for FileInode {
    fn read(&self, writer: ostd::mm::VmWriter) -> crate::error::Result<usize> {
        self.inode.read_at(0, writer)
    }

    fn write(&self, reader: ostd::mm::VmReader) -> crate::error::Result<usize> {
        self.inode.write_at(0, reader)
    }

    fn as_inode(&self) -> Option<&dyn Inode> {
        Some(self.inode.as_ref())
    }
}
