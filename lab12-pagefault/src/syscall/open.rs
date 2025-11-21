use core::ffi::CStr;

use alloc::string::ToString;
use alloc::sync::Arc;
use alloc::vec;
use log::debug;
use ostd::mm::{FallibleVmRead, Vaddr, VmWriter};

use crate::error::Result;
use crate::fs::file_table::FileEntry;
use crate::process::Process;
use crate::syscall::SyscallReturn;

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

    let root_inode = crate::fs::ROOT.get().unwrap().root_inode();

    let open_inode = root_inode.open(file_name.to_string());

    let file = crate::fs::util::FileInode::new(open_inode);
    let fd = current_process
        .file_table()
        .insert(FileEntry::new(Arc::new(file)));

    Ok(SyscallReturn(fd as _))
}
