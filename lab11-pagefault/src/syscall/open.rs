use core::ffi::CStr;

use alloc::string::ToString;
use alloc::sync::Arc;
use alloc::vec;
use log::debug;
use ostd::mm::{FallibleVmRead, Vaddr, VmWriter};

use crate::error::{Errno, Error, Result};
use crate::fs::InodeType;
use crate::fs::file_table::FileEntry;
use crate::fs::util::PathString;
use crate::process::Process;
use crate::syscall::SyscallReturn;

bitflags::bitflags! {
    pub struct OpenFlags: u32 {
        const O_CREAT = 1 << 6;
    }
}

pub fn sys_openat(
    dfd: usize,
    file_name: Vaddr,
    flags: usize,
    mode: usize,
    current_process: &Arc<Process>,
) -> Result<SyscallReturn> {
    debug!(
        "[SYS_OPENAT] dfd: {:#x}, file_name: {:#x}, flags: {:#x}, mode: {:#x}",
        dfd, file_name, flags, mode
    );

    // The max file name: 255 bytes + 1(\0)
    const MAX_FILENAME_LENGTH: usize = 256;
    let mut buffer = vec![0u8; MAX_FILENAME_LENGTH];
    current_process
        .memory_space()
        .vm_space()
        .reader(file_name, MAX_FILENAME_LENGTH)
        .unwrap()
        .read_fallible(&mut VmWriter::from(&mut buffer as &mut [u8]))
        .unwrap();

    let file_name = CStr::from_bytes_until_nul(&buffer)
        .unwrap()
        .to_str()
        .unwrap();

    let create = OpenFlags::from_bits_truncate(flags as u32).contains(OpenFlags::O_CREAT);
    let mut path_string = PathString::new(file_name.to_string());
    let current_inode = crate::fs::ROOT.get().unwrap().root_inode();
    if path_string.is_empty() {
        return Err(Error::new(Errno::EINVAL));
    }

    let open_inode = if create {
        path_string.create(current_inode.as_ref(), InodeType::File)?
    } else {
        path_string.lookup(current_inode.as_ref())?
    };

    let file = crate::fs::util::FileInode::new(open_inode);
    let fd = current_process
        .file_table()
        .insert(FileEntry::new(Arc::new(file)));

    Ok(SyscallReturn(fd as _))
}
