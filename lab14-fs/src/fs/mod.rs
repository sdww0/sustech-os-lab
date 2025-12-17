#![expect(unused)]

pub mod ext2;
mod file;
pub mod file_table;
pub mod pipe;
pub mod ramfs;
pub mod util;

use crate::error::Result;
use core::{ffi::CStr, time::Duration};

use alloc::{boxed::Box, string::String, sync::Arc};
pub use file::{FileLike, Stderr, Stdin, Stdout};
use ostd::{
    early_println,
    mm::{VmReader, VmWriter},
};
use spin::Once;

pub static ROOT: Once<Box<dyn FileSystem>> = Once::new();

pub static EXT2_FS: Once<Arc<dyn FileSystem>> = Once::new();

pub fn init() {
    ROOT.call_once(|| {
        let ramfs = ramfs::RamFS::new();
        Box::new(ramfs) as Box<dyn FileSystem>
    });

    for blk_device in crate::drivers::BLOCK_DEVICES.get().unwrap().lock().iter() {
        if let Ok(fs) = ext2::Ext2Fs::new(blk_device.clone()) {
            EXT2_FS.call_once(|| fs as Arc<dyn FileSystem>);
            break;
        }
    }

    if let Some(fs) = EXT2_FS.get() {
        fs.root_inode(); // Warm up inode cache
        ext2_test();
    }
}

fn ext2_test() {
    if let Some(fs) = EXT2_FS.get() {
        let root_inode = fs.root_inode();
        let result = root_inode.lookup("hello_ext2.txt").unwrap();

        let mut buf: [u8; 128] = [0; 128];
        result
            .read_at(0, VmWriter::from(buf.as_mut()).to_fallible())
            .unwrap();

        early_println!(
            "Read from ext2: {}",
            CStr::from_bytes_until_nul(buf.as_ref())
                .unwrap()
                .to_str()
                .unwrap()
        );
    } else {
        early_println!("No Ext2 filesystem found.");
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
