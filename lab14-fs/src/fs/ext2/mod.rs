//! Ext2 file system implementation
//!
//! References: https://wiki.osdev.org/Ext2

use core::fmt::Debug;
use core::ops::Add;

use alloc::sync::Weak;
use alloc::{collections::btree_map::BTreeMap, sync::Arc, vec::Vec};
use log::{debug, info};
use ostd::Pod;
use ostd::{early_println, sync::Mutex};

use crate::fs::ext2::inode::RawInode;
use crate::fs::ext2::super_block::EXT2_FIRST_SUPERBLOCK_OFFSET;
use crate::fs::util::sector_ptr::SectorPtr;
use crate::{
    drivers::blk::{BlockDevice, SECTOR_SIZE},
    error::{Error, Result},
    fs::{
        FileSystem,
        ext2::{
            block_group::BlockGroup,
            inode::Inode,
            super_block::{RawSuperBlock, SuperBlock},
        },
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
    block_groups: Vec<BlockGroup>,

    inode_cache: Mutex<BTreeMap<u32, Arc<Inode>>>,
    inodes_per_group: u32,
    blocks_per_group: u32,
    inode_size: usize,
    block_size: usize,

    self_ref: Weak<Ext2Fs>,
}

impl Ext2Fs {
    pub fn new(blk_device: Arc<dyn BlockDevice>) -> Result<Arc<Self>> {
        let raw_super_block: RawSuperBlock =
            blk_device.read_val(EXT2_FIRST_SUPERBLOCK_OFFSET / SECTOR_SIZE);

        if raw_super_block.magic != EXT2_MAGIC {
            return Err(Error::new(crate::error::Errno::EACCES));
        }

        debug!("Ext2 raw super block:{:#x?}", raw_super_block);

        let super_block = SuperBlock::from(raw_super_block);

        // We currently only support exactly one block group.
        assert!(super_block.inodes_per_group == super_block.inodes_count);
        assert!(super_block.blocks_per_group == super_block.blocks_count);
        // We currently only support 4KB block size.
        assert!(super_block.block_size == 4096);

        let first_group_bid = super_block.group_descriptor_table_bid();

        let raw_descriptor: block_group::RawGroupDescriptor = blk_device
            .read_val(first_group_bid.0 as usize * super_block.block_size as usize / SECTOR_SIZE);

        let mut blk_groups = Vec::new();
        blk_groups.push(BlockGroup::new(raw_descriptor));

        let fs = Arc::new_cyclic(|fs| Ext2Fs {
            blk_device,
            inodes_per_group: super_block.inodes_per_group,
            blocks_per_group: super_block.blocks_per_group,
            block_size: super_block.block_size as usize,
            inode_size: super_block.inode_size as usize,
            super_block: SuperBlock::from(super_block),
            inode_cache: Mutex::new(BTreeMap::new()),
            block_groups: blk_groups,
            self_ref: fs.clone(),
        });

        Ok(fs)
    }

    fn lookup_inode(&self, inode_number: u32) -> Result<Arc<Inode>> {
        let idx = inode_number - 1;
        if let Some(inode) = self.inode_cache.lock().get(&inode_number) {
            return Ok(inode.clone());
        }

        if idx >= self.super_block.inodes_count {
            return Err(Error::new(crate::error::Errno::ENOENT));
        }

        let inode_table_block =
            self.block_groups[(idx / self.inodes_per_group) as usize].inode_table_start_bid();
        let inodes_per_block = (self.block_size / self.inode_size) as u32;
        let bid_offset = Ext2Bid::from(idx / inodes_per_block);
        let offset_in_block = idx % inodes_per_block;
        let bid_num = inode_table_block + bid_offset;

        debug!(
            "inode_table_block: {:?}, inodes_per_block: {:?}, bid_offset: {:?}, offset_in_block: {:?}, bid_num: {:?}",
            inode_table_block, inodes_per_block, bid_offset, offset_in_block, bid_num
        );

        // Convert to sector number and offset within sector
        let sector =
            self.bid_to_sector(bid_num) + offset_in_block as usize * self.inode_size / SECTOR_SIZE;
        let sector_offset = (offset_in_block as usize * self.inode_size) % SECTOR_SIZE;

        let sector_ptr: SectorPtr<RawInode> =
            SectorPtr::new(sector, sector_offset, &self.blk_device);

        let inode = Inode::new(
            sector_ptr,
            inode_number,
            (idx / self.inodes_per_group) as usize,
            self.self_ref.clone(),
        );

        Ok(inode)
    }

    pub fn bid_to_sector(&self, bid: Ext2Bid) -> usize {
        bid.0 as usize * self.block_size / SECTOR_SIZE
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

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Pod, Default)]
pub struct Ext2Bid(u32);

impl From<u32> for Ext2Bid {
    fn from(value: u32) -> Self {
        Ext2Bid(value)
    }
}

impl Add for Ext2Bid {
    type Output = Ext2Bid;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}
