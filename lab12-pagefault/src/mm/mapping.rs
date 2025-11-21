use ostd::mm::{PageFlags, Segment, Vaddr};

#[derive(Debug)]
pub struct VmMapping {
    base_vaddr: Vaddr,
    /// Mapping page count with PAGE_SIZE as unit.
    pages: usize,
    frames: Option<Segment<()>>,
    perms: PageFlags,
}

impl VmMapping {
    pub fn new(base_vaddr: Vaddr, pages: usize, perms: PageFlags) -> Self {
        Self {
            base_vaddr,
            pages,
            frames: None,
            perms,
        }
    }

    pub fn set_frames(&mut self, frames: Segment<()>) {
        self.frames = Some(frames);
    }

    pub fn base_vaddr(&self) -> Vaddr {
        self.base_vaddr
    }

    pub fn pages(&self) -> usize {
        self.pages
    }

    pub fn perms(&self) -> PageFlags {
        self.perms
    }

    pub fn frames(&self) -> &Segment<()> {
        self.frames.as_ref().unwrap()
    }
}
