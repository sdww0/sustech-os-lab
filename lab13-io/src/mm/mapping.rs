use ostd::mm::{Frame, PAGE_SIZE, PageFlags, Vaddr};

#[derive(Debug, Clone)]
pub struct VmMapping {
    base_vaddr: Vaddr,
    frame: Frame<()>,
    perms: PageFlags,
}

impl VmMapping {
    pub fn new(base_vaddr: Vaddr, perms: PageFlags, frame: Frame<()>) -> Self {
        Self {
            base_vaddr,
            frame,
            perms,
        }
    }

    pub fn contains_vaddr(&self, vaddr: Vaddr) -> bool {
        vaddr >= self.base_vaddr && vaddr < self.base_vaddr + PAGE_SIZE
    }

    pub fn base_vaddr(&self) -> Vaddr {
        self.base_vaddr
    }

    pub fn perms(&self) -> PageFlags {
        self.perms
    }

    pub fn remove_perm(&mut self, flag: PageFlags) {
        self.perms.remove(flag);
    }

    pub fn frame(&self) -> &Frame<()> {
        &self.frame
    }
}
