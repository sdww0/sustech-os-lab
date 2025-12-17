use ostd::Pod;

use crate::fs::ext2::Ext2Bid;

#[expect(unused)]
pub struct BlockGroup {
    bitmap_start_bid: u32,
    inode_start_bid: u32,
    inode_table_start_bid: u32,
    free_blocks: u32,
    free_inodes: u32,
    direntries: u32,
}

impl BlockGroup {
    pub fn new(raw_descriptor: RawGroupDescriptor) -> Self {
        Self {
            bitmap_start_bid: raw_descriptor.block_bitmap,
            inode_start_bid: raw_descriptor.inode_bitmap,
            inode_table_start_bid: raw_descriptor.inode_table,
            free_blocks: raw_descriptor.free_blocks_count as _,
            free_inodes: raw_descriptor.free_inodes_count as _,
            direntries: raw_descriptor.dirs_count as _,
        }
    }

    pub fn inode_table_start_bid(&self) -> Ext2Bid {
        self.inode_table_start_bid.into()
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod)]
pub(super) struct RawGroupDescriptor {
    block_bitmap: u32,
    inode_bitmap: u32,
    inode_table: u32,
    free_blocks_count: u16,
    free_inodes_count: u16,
    dirs_count: u16,
    pad: u16,
    reserved: [u32; 3],
}

impl From<RawGroupDescriptor> for BlockGroup {
    fn from(value: RawGroupDescriptor) -> Self {
        Self {
            bitmap_start_bid: value.block_bitmap,
            inode_start_bid: value.inode_bitmap,
            inode_table_start_bid: value.inode_table,
            free_blocks: value.free_blocks_count as _,
            free_inodes: value.free_inodes_count as _,
            direntries: value.dirs_count as _,
        }
    }
}
