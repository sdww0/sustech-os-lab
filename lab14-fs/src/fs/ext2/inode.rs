use alloc::sync::Weak;

use crate::fs::{InodeType, ext2::Ext2Fs};

pub struct Inode {
    inode_id: u32,
    type_: InodeType,
    block_groupd_idx: usize,
    inner: Inner,
    fs: Weak<Ext2Fs>,
}

struct Inner {}

impl super::super::Inode for Inode {
    fn lookup(&self, name: &str) -> crate::error::Result<alloc::sync::Arc<dyn crate::fs::Inode>> {
        todo!()
    }

    fn create(
        &self,
        name: &str,
        type_: InodeType,
    ) -> crate::error::Result<alloc::sync::Arc<dyn crate::fs::Inode>> {
        todo!()
    }

    fn read_link(&self) -> crate::error::Result<alloc::string::String> {
        todo!()
    }

    fn write_link(&self, target: &str) -> crate::error::Result<()> {
        todo!()
    }

    fn read_at(&self, offset: usize, writer: ostd::mm::VmWriter) -> crate::error::Result<usize> {
        todo!()
    }

    fn write_at(&self, offset: usize, reader: ostd::mm::VmReader) -> crate::error::Result<usize> {
        todo!()
    }

    fn metadata(&self) -> &crate::fs::InodeMeta {
        todo!()
    }

    fn size(&self) -> usize {
        todo!()
    }

    fn typ(&self) -> InodeType {
        todo!()
    }
}
