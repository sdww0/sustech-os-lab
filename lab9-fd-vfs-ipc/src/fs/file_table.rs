use alloc::{sync::Arc, vec::Vec};

use crate::fs::{FileLike, Stderr, Stdin, Stdout};

pub type FileDescriptor = i32;

/// Represents an open file entry
pub struct FileEntry {
    file: Arc<dyn FileLike>,
}

impl FileEntry {
    pub fn new(file: Arc<dyn FileLike>) -> Self {
        FileEntry { file }
    }

    pub fn file(&self) -> &Arc<dyn FileLike> {
        &self.file
    }
}

/// File table structure
pub struct FileTable {
    table: Vec<Option<FileEntry>>,
    fds_in_use: usize,
}

impl FileTable {
    /// Creates a new file table
    pub fn new() -> Self {
        FileTable {
            table: Vec::new(),
            fds_in_use: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.fds_in_use
    }

    pub fn duplicate(&self) -> Self {
        let mut new_table = Vec::new();
        for entry in &self.table {
            if let Some(e) = entry {
                new_table.push(Some(FileEntry {
                    file: e.file.clone(),
                }));
            } else {
                new_table.push(None);
            }
        }
        FileTable {
            table: new_table,
            fds_in_use: self.fds_in_use,
        }
    }

    pub fn new_with_standard_io() -> Self {
        let mut table = Vec::new();
        table.push(Some(FileEntry {
            file: Arc::new(Stdin),
        }));
        table.push(Some(FileEntry {
            file: Arc::new(Stdout),
        }));
        table.push(Some(FileEntry {
            file: Arc::new(Stderr),
        }));
        FileTable {
            table,
            fds_in_use: 3,
        }
    }

    pub fn insert(&mut self, entry: FileEntry) -> FileDescriptor {
        let fd = if self.fds_in_use == self.table.len() {
            self.table.push(Some(entry));
            self.fds_in_use as FileDescriptor
        } else {
            let index = self.table.iter().position(|e| e.is_none()).unwrap();
            self.table[index] = Some(entry);
            index as FileDescriptor
        };
        self.fds_in_use += 1;
        fd
    }

    pub fn get(&self, fd: FileDescriptor) -> Option<&FileEntry> {
        self.table.get(fd as usize)?.as_ref()
    }

    /// Closes a file descriptor
    pub fn close(&mut self, fd: FileDescriptor) -> Option<FileEntry> {
        let entry = self.table.get_mut(fd as usize)?.take()?;
        self.fds_in_use -= 1;
        Some(entry)
    }
}
