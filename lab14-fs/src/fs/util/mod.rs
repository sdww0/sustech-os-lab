pub mod sector_ptr;

use alloc::{string::String, sync::Arc};

use crate::error::Result;
use crate::fs::{FileLike, Inode, InodeType};

pub struct FileInode {
    inode: Arc<dyn Inode>,
}

impl FileInode {
    pub fn new(inode: Arc<dyn Inode>) -> Self {
        Self { inode }
    }
}

impl FileLike for FileInode {
    fn read(&self, writer: ostd::mm::VmWriter) -> crate::error::Result<usize> {
        self.inode.read_at(0, writer)
    }

    fn write(&self, reader: ostd::mm::VmReader) -> crate::error::Result<usize> {
        self.inode.write_at(0, reader)
    }

    fn as_inode(&self) -> Option<Arc<dyn Inode>> {
        Some(self.inode.clone())
    }
}

#[derive(Debug)]
pub struct PathString {
    inner: String,
    location: usize,
}

impl PathString {
    pub fn new(mut s: String) -> Self {
        while s.starts_with('/') {
            s.remove(0);
        }

        while s.ends_with('/') {
            s.pop();
        }

        Self {
            inner: s,
            location: 0,
        }
    }

    pub fn lookup<'a>(&mut self, start: &'a dyn Inode) -> Result<Arc<dyn Inode>> {
        let mut current = if self.peek().is_none() {
            return Ok(start.lookup("")?);
        } else {
            start.lookup(&self.next().unwrap())?
        };

        while let Some(name) = self.next() {
            let next_inode = current.lookup(&name)?;
            current = next_inode;
        }
        Ok(current)
    }

    pub fn create<'a>(&mut self, start: &'a dyn Inode, type_: InodeType) -> Result<Arc<dyn Inode>> {
        let mut last_name = String::new();
        let mut current = start;
        let mut next_inode;
        while let Some(name) = self.next() {
            if self.peek().is_none() {
                last_name = name;
                break;
            }
            next_inode = current.lookup(&name)?;
            current = next_inode.as_ref();
        }

        let new_inode = current.create(&last_name, type_)?;
        Ok(new_inode)
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn peek(&self) -> Option<String> {
        if self.location >= self.inner.len() {
            return None;
        }
        let bytes = self.inner.as_bytes();
        let mut next_location = self.location;
        while next_location < bytes.len() && bytes[next_location] != b'/' {
            next_location += 1;
        }
        let part = String::from(&self.inner[self.location..next_location]);
        Some(part)
    }
}

impl Iterator for PathString {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        if self.location >= self.inner.len() {
            return None;
        }
        let bytes = self.inner.as_bytes();
        let mut next_location = self.location;
        while next_location < bytes.len() && bytes[next_location] != b'/' {
            next_location += 1;
        }
        let part = String::from(&self.inner[self.location..next_location]);
        self.location = next_location + 1; // Skip the '/'
        Some(part)
    }
}

impl From<String> for PathString {
    fn from(s: String) -> Self {
        PathString::new(s)
    }
}
