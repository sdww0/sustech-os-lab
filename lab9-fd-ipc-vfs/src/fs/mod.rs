mod file;
pub mod file_table;
pub mod pipe;
pub mod ramfs;
pub mod util;

use crate::error::Result;
use core::time::Duration;

pub use file::{FileLike, Stderr, Stdin, Stdout};
use ostd::mm::{VmReader, VmWriter};

pub trait FileSystem {
    fn name(&self) -> &str;
}

pub trait Inode: Send + Sync {
    fn read_at(&self, offset: usize, writer: VmWriter) -> Result<usize>;
    fn write_at(&self, offset: usize, reader: VmReader) -> Result<usize>;
    fn metadata(&self) -> &InodeMeta;
    fn size(&self) -> usize;
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
