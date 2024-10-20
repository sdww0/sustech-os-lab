// SPDX-License-Identifier: MPL-2.0

use ostd::Pod;

use crate::{prelude::*, process::current_process, time::timespec_t};

pub fn sys_fstat(fd: i32, stat_buf_ptr: Vaddr) -> Result<SyscallReturn> {
    if fd < 0 {
        return Err(Error::new(Errno::EINVAL));
    }
    if fd > 2 {
        panic!("We don't support file system now");
    }

    let stdio_fstat = Stat {
        st_dev: 25,
        st_ino: 0x1b,
        st_nlink: 1,
        st_mode: 0x3e8,
        st_uid: 0,
        st_gid: 0,
        __pad0: 0,
        st_rdev: 0,
        st_size: 0,
        st_blksize: 1024,
        st_blocks: 0,
        st_atime: timespec_t::default(),
        st_mtime: timespec_t::default(),
        st_ctime: timespec_t::default(),
        __unused: [0; 3],
    };
    let process = current_process().unwrap();
    let user_space = process.user_space().unwrap();

    user_space
        .vm_space()
        .writer(stat_buf_ptr, size_of::<Stat>())
        .unwrap()
        .write_val(&stdio_fstat)
        .unwrap();

    Ok(SyscallReturn::Return(0))
}

/// File Stat
#[derive(Debug, Clone, Copy, Pod, Default)]
#[repr(C)]
pub struct Stat {
    /// ID of device containing file
    st_dev: u64,
    /// Inode number
    st_ino: u64,
    /// Number of hard links
    st_nlink: usize,
    /// File type and mode
    st_mode: u32,
    /// User ID of owner
    st_uid: u32,
    /// Group ID of owner
    st_gid: u32,
    /// Padding bytes
    __pad0: u32,
    /// Device ID (if special file)
    st_rdev: u64,
    /// Total size, in bytes
    st_size: isize,
    /// Block size for filesystem I/O
    st_blksize: isize,
    /// Number of 512-byte blocks allocated
    st_blocks: isize,
    /// Time of last access
    st_atime: timespec_t,
    /// Time of last modification
    st_mtime: timespec_t,
    /// Time of last status change
    st_ctime: timespec_t,
    /// Unused field
    __unused: [i64; 3],
}
