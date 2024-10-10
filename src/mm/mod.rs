use bitflags::bitflags;
use ostd::mm::PageFlags;

bitflags! {
    pub struct VmPerms: u32 {
        const READ    = 1 << 0;
        const WRITE   = 1 << 1;
        const EXEC   = 1 << 2;
    }
}

impl Into<PageFlags> for VmPerms {
    fn into(self) -> PageFlags {
        PageFlags::from_bits_truncate(self.bits as u8)
    }
}
