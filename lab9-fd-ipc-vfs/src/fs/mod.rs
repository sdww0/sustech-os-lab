mod file;
pub mod file_table;
pub mod pipe;
pub mod ramfs;
pub mod util;

use crate::error::Result;
use core::time::Duration;

use alloc::{boxed::Box, string::String, sync::Arc};
pub use file::{FileLike, Stderr, Stdin, Stdout};
use ostd::mm::{VmReader, VmWriter};
use spin::Once;

pub static ROOT: Once<Box<dyn FileSystem>> = Once::new();

pub fn init() {
    ROOT.call_once(|| {
        let ramfs = ramfs::RamFS::new();
        Box::new(ramfs) as Box<dyn FileSystem>
    });
}

pub trait FileSystem: Send + Sync {
    fn name(&self) -> &str;

    fn root_inode(&self) -> Arc<dyn Inode>;
}

pub trait Inode: Send + Sync {
    fn open(self: Arc<Self>, name: String) -> Arc<dyn Inode>;
    fn read_at(&self, offset: usize, writer: VmWriter) -> Result<usize>;
    fn write_at(&self, offset: usize, reader: VmReader) -> Result<usize>;
    fn metadata(&self) -> InodeMeta;
    fn size(&self) -> usize;
}

#[derive(Clone)]
pub struct InodeMeta {
    /// File size
    size: usize,
    /// Last access time
    atime: Duration,
    /// Last modification time
    mtime: Duration,
    /// Last status change time
    ctime: Duration,
}
