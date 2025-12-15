use ostd::mm::DmaStream;

use crate::drivers::utils::DmaSlice;

pub const SECTOR_SIZE: usize = 512;
pub trait BlockDevice: Send + Sync {
    fn read_block(&self, index: usize, data: &mut DmaSlice<[u8; SECTOR_SIZE], DmaStream>);

    fn write_block(&self, index: usize, data: &DmaSlice<[u8; SECTOR_SIZE], DmaStream>);
}
