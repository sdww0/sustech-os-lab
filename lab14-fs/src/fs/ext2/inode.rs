#![expect(unused_variables)]

use alloc::{
    sync::{Arc, Weak},
    vec::Vec,
};
use log::debug;
use ostd::Pod;

use crate::{
    drivers::blk::SECTOR_SIZE,
    fs::{
        InodeType,
        ext2::{Ext2Bid, Ext2Fs, dir_entry::Ext2DirEntry},
        util::sector_ptr::SectorPtr,
    },
};

#[expect(unused)]
pub struct Inode {
    sector_ptr: SectorPtr<RawInode>,

    inode_id: u32,
    type_: InodeType,
    block_group_idx: usize,
    inner: Inner,
    fs: Weak<Ext2Fs>,
}

enum Inner {
    File,
    Directory(Vec<Ext2DirEntry>),
}

impl Inode {
    pub fn new(
        sector_ptr: SectorPtr<RawInode>,
        inode_id: u32,
        block_group_idx: usize,
        fs: Weak<Ext2Fs>,
    ) -> Arc<Self> {
        let raw_inode: RawInode = sector_ptr.read();

        let type_ = match raw_inode.mode & 0xF000 {
            0x4000 => InodeType::Directory,
            0x8000 => InodeType::File,
            0xA000 => InodeType::SymbolLink,
            _ => panic!("Unsupported inode type"),
        };

        debug!("Inode {} type: {:?}", inode_id, type_);
        debug!("Raw inode data: {:#x?}", raw_inode);

        let inner = match type_ {
            InodeType::Directory => {
                Inner::Directory(read_directory(type_, &raw_inode, fs.clone()).unwrap())
            }
            InodeType::File | InodeType::SymbolLink => Inner::File,
        };

        let inode = Arc::new(Inode {
            inode_id,
            type_,
            block_group_idx,
            inner,
            fs,
            sector_ptr,
        });
        inode
    }
}

fn read_directory(
    type_: InodeType,
    raw_inode: &RawInode,
    fs: Weak<Ext2Fs>,
) -> Option<Vec<Ext2DirEntry>> {
    if type_ != InodeType::Directory {
        return None;
    }

    // Read directory entries
    let mut dir_entries = Vec::new();
    for &block_ptr in &raw_inode.block_ptrs.direct_pointers {
        if block_ptr.0 == 0 {
            continue;
        }

        let fs = fs.upgrade().expect("Filesystem has been dropped");
        let block_size = fs.block_size as usize;
        let sector = fs.bid_to_sector(block_ptr);

        let mut offset = 0;
        while offset < block_size {
            let dir_entry: Ext2DirEntry = fs
                .blk_device
                .read_val_offset(sector + offset / SECTOR_SIZE, offset % SECTOR_SIZE);

            if dir_entry.inode() == 0 {
                break;
            }

            offset += dir_entry.length() as usize;
            dir_entries.push(dir_entry);

            debug!(
                "Dir Entry: inode={}, rec_len={}, name_len={}, name={}",
                dir_entry.inode(),
                dir_entry.length(),
                dir_entry.name_length(),
                dir_entry.name()
            );
        }
    }

    Some(dir_entries)
}

impl super::super::Inode for Inode {
    fn lookup(&self, name: &str) -> crate::error::Result<alloc::sync::Arc<dyn crate::fs::Inode>> {
        if self.type_ != InodeType::Directory {
            return Err(crate::error::Error::new(crate::error::Errno::ENOTDIR));
        }

        if let Inner::Directory(ref entries) = self.inner {
            for entry in entries {
                if entry.name() == name {
                    let fs = self.fs.upgrade().expect("Filesystem has been dropped");
                    let inode = fs.lookup_inode(entry.inode())?;
                    return Ok(inode);
                }
            }
        }
        Err(crate::error::Error::new(crate::error::Errno::ENOENT))
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

    fn read_at(
        &self,
        offset: usize,
        mut writer: ostd::mm::VmWriter,
    ) -> crate::error::Result<usize> {
        if self.type_ != InodeType::File {
            return Err(crate::error::Error::new(crate::error::Errno::EISDIR));
        }

        let sector_ptr = &self.sector_ptr;
        let raw_inode: RawInode = sector_ptr.read();
        let fs = self.fs.upgrade().expect("Filesystem has been dropped");
        let block_size = fs.block_size as usize;

        let mut bytes_read = 0;
        let mut current_offset = offset;

        // Find start block and offset within block
        let mut block_index = current_offset / block_size;
        let mut offset_in_block = current_offset % block_size;

        // Read data block by block
        while writer.avail() > 0 {
            let block_ptr = if block_index < 12 {
                raw_inode.block_ptrs.direct_pointers[block_index as usize]
            } else {
                // For simplicity, we only handle direct pointers here
                break;
            };
            if block_ptr.0 == 0 {
                break;
            }
            let sector = fs.bid_to_sector(block_ptr);
            let to_read = core::cmp::min(block_size - offset_in_block, writer.avail() - bytes_read);

            debug!(
                "Reading block_index: {}, block_ptr: {:?}, sector: {}, offset_in_block: {}, to_read: {}",
                block_index, block_ptr, sector, offset_in_block, to_read
            );
            fs.blk_device.read_to_vm_writer(
                sector + offset_in_block / SECTOR_SIZE,
                (to_read + SECTOR_SIZE - 1) / SECTOR_SIZE,
                &mut writer,
            );

            bytes_read += to_read;
            current_offset += to_read;
            offset_in_block = 0; // After first block, offset is 0
            block_index += 1;
        }

        Ok(bytes_read)
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
        self.type_
    }
}

#[repr(C)]
#[derive(Clone, Copy, Default, Debug, Pod)]
pub(super) struct RawInode {
    /// File mode (type and permissions).
    pub mode: u16,
    /// Low 16 bits of User Id.
    pub uid: u16,
    /// Lower 32 bits of size in bytes.
    pub size_low: u32,
    /// Access time.
    pub atime: u32,
    /// Change time.
    pub ctime: u32,
    /// Modification time.
    pub mtime: u32,
    /// Deletion time.
    pub dtime: u32,
    /// Low 16 bits of Group Id.
    pub gid: u16,
    pub hard_links: u16,
    pub blocks_count: u32,
    /// File flags.
    pub flags: u32,
    /// OS dependent Value 1.
    reserved1: u32,
    /// Pointers to blocks.
    pub block_ptrs: BlockPointers,
    /// File version (for NFS).
    pub generation: u32,
    /// In revision 0, this field is reserved.
    /// In revision 1, File ACL.
    pub file_acl: u32,
    /// In revision 0, this field is reserved.
    /// In revision 1, Upper 32 bits of file size (if feature bit set)
    /// if it's a file, Directory ACL if it's a directory.
    pub size_high: u32,
    /// Fragment address.
    pub frag_addr: u32,
    /// OS dependent 2.
    pub os_dependent_2: OsDependent2,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Default)]
pub struct BlockPointers {
    direct_pointers: [Ext2Bid; 12],
    single_indirect_pointer: Ext2Bid,
    double_indirect_pointer: Ext2Bid,
    triple_indirect_pointer: Ext2Bid,
}

/// OS dependent 2.
///
/// Here we use the Linux definition.
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Default)]
pub struct OsDependent2 {
    pub fragment_number: u8,
    pub fragment_size: u8,
    _pad: [u8; 2],
    pub uid_high: u16,
    pub gid_high: u16,
    _reserved: u32,
}
