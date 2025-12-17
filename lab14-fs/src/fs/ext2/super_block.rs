#![expect(unused)]

use ostd::Pod;

use crate::fs::ext2::Ext2Bid;

pub const EXT2_FIRST_SUPERBLOCK_OFFSET: usize = 1024;
pub const EXT2_SUPERBLOCK_SIZE: usize = 1024;

#[derive(Debug)]
pub struct SuperBlock {
    pub idx: u32,

    pub block_size: u32,

    pub inodes_count: u32,
    pub blocks_count: u32,
    pub reserved_blocks_count: u32,
    pub free_blocks_count: u32,
    pub free_inodes_count: u32,

    pub first_data_block: u32,
    pub log_frag_size: u32,

    pub blocks_per_group: u32,
    pub frags_per_group: u32,
    pub inodes_per_group: u32,
    pub mnt_count: u16,
    pub max_mnt_count: u16,
    pub first_ino: u32,
    pub inode_size: u16,
}

impl SuperBlock {
    pub fn group_descriptor_table_bid(&self) -> Ext2Bid {
        (self.idx * self.blocks_per_group + 1).into()
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Default)]
pub struct RawSuperBlock {
    pub inodes_count: u32,
    pub blocks_count: u32,
    pub reserved_blocks_count: u32,
    pub free_blocks_count: u32,
    pub free_inodes_count: u32,
    pub first_data_block: u32,
    pub log_block_size: u32,
    pub log_frag_size: u32,
    pub blocks_per_group: u32,
    pub frags_per_group: u32,
    pub inodes_per_group: u32,
    pub mtime: u32,
    pub wtime: u32,
    pub mnt_count: u16,
    pub max_mnt_count: u16,
    pub magic: u16,
    pub state: u16,
    pub errors: u16,
    pub min_rev_level: u16,
    pub last_check_time: u32,
    pub check_interval: u32,
    pub creator_os: u32,
    pub rev_level: u32,
    pub def_resuid: u16,
    pub def_resgid: u16,
    pub first_ino: u32,
    pub inode_size: u16,
    pub block_group_idx: u16,
    pub feature_compat: u32,
    pub feature_incompat: u32,
    pub feature_ro_compat: u32,
    pub uuid: [u8; 16],
    pub volume_name: [u8; 16],
}

impl From<RawSuperBlock> for SuperBlock {
    fn from(value: RawSuperBlock) -> Self {
        Self {
            block_size: 1024 << value.log_block_size,
            inodes_count: value.inodes_count,
            blocks_count: value.blocks_count,
            reserved_blocks_count: value.reserved_blocks_count,
            free_blocks_count: value.free_blocks_count,
            free_inodes_count: value.free_inodes_count,
            first_data_block: value.first_data_block,
            log_frag_size: value.log_frag_size,
            blocks_per_group: value.blocks_per_group,
            frags_per_group: value.frags_per_group,
            inodes_per_group: value.inodes_per_group,
            mnt_count: value.mnt_count,
            max_mnt_count: value.max_mnt_count,
            first_ino: value.first_ino,
            inode_size: value.inode_size,
            idx: value.block_group_idx as u32,
        }
    }
}
