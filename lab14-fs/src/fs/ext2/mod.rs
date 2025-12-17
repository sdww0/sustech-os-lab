//! Ext2 file system implementation
//!

use core::fmt::Debug;

use alloc::sync::Arc;
use ostd::early_println;

use crate::{
    drivers::blk::BlockDevice,
    error::{Error, Result},
    fs::{
        FileSystem,
        ext2::{inode::Inode, super_block::SuperBlock},
    },
};

mod block_group;
mod dir_entry;
mod inode;
mod super_block;

const EXT2_MAGIC: u16 = 0xEF53;
/// The root inode number.
const ROOT_INO: u32 = 2;

pub struct Ext2Fs {
    blk_device: Arc<dyn BlockDevice>,
    super_block: SuperBlock,
    inodes_per_group: u32,
    blocks_per_group: u32,
    inode_size: usize,
    block_size: usize,
}

impl Ext2Fs {
    pub fn new(blk_device: Arc<dyn BlockDevice>) -> Result<Self> {
        let super_block: SuperBlock = blk_device.read_val(2);

        if super_block.magic != EXT2_MAGIC {
            return Err(Error::new(crate::error::Errno::EACCES));
        }

        let block_size = 1024 << super_block.log_block_size;

        early_println!("Ext2 fs:{:#x?}", super_block);
        early_println!("Block size: {}", block_size);

        Ok(Ext2Fs {
            blk_device,
            super_block,
            inodes_per_group: super_block.inodes_per_group,
            blocks_per_group: super_block.blocks_per_group,
            inode_size: super_block.inode_size as usize,
            block_size: block_size as usize,
        })
    }

    fn lookup_inode(&self, inode_number: u32) -> Result<Arc<Inode>> {
        todo!()
    }
}

impl Debug for Ext2Fs {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Ext2Fs")
            .field("super_block", &self.super_block)
            .field("inodes_per_group", &self.inodes_per_group)
            .field("blocks_per_group", &self.blocks_per_group)
            .field("inode_size", &self.inode_size)
            .field("block_size", &self.block_size)
            .finish()
    }
}

impl FileSystem for Ext2Fs {
    fn name(&self) -> &str {
        "ext2"
    }

    fn root_inode(&self) -> Arc<dyn crate::fs::Inode> {
        self.lookup_inode(ROOT_INO).unwrap()
    }
}
