pub mod ext2;
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

pub static EXT2_FS: Once<Box<dyn FileSystem>> = Once::new();

pub fn init() {
    ROOT.call_once(|| {
        let ramfs = ramfs::RamFS::new();
        Box::new(ramfs) as Box<dyn FileSystem>
    });

    for blk_device in crate::drivers::BLOCK_DEVICES.get().unwrap().lock().iter() {
        if let Ok(fs) = ext2::Ext2Fs::new(blk_device.clone()) {
            EXT2_FS.call_once(|| Box::new(fs) as Box<dyn FileSystem>);
            return;
        }
    }
}

pub trait FileSystem: Send + Sync {
    fn name(&self) -> &str;

    fn root_inode(&self) -> Arc<dyn Inode>;
}

pub trait Inode: Send + Sync {
    fn lookup(&self, name: &str) -> Result<Arc<dyn Inode>>;
    fn create(&self, name: &str, type_: InodeType) -> Result<Arc<dyn Inode>>;

    fn read_link(&self) -> Result<String>;
    fn write_link(&self, target: &str) -> Result<()>;

    fn read_at(&self, offset: usize, writer: VmWriter) -> Result<usize>;
    fn write_at(&self, offset: usize, reader: VmReader) -> Result<usize>;
    fn metadata(&self) -> &InodeMeta;
    fn size(&self) -> usize;

    fn typ(&self) -> InodeType;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InodeType {
    File,
    Directory,
    SymbolLink,
}

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
