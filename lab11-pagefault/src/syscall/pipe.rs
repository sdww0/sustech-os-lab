use alloc::sync::Arc;
use log::debug;
use ostd::Pod;
use ostd::mm::Vaddr;

use crate::error::Result;
use crate::fs::file_table::FileEntry;
use crate::fs::pipe::Pipe;
use crate::process::Process;
use crate::syscall::SyscallReturn;

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod)]
struct PipeFds {
    read_fd: i32,
    write_fd: i32,
}

pub fn sys_pipe2(
    pipe_address: Vaddr,
    flags: i32,
    current_process: &Arc<Process>,
) -> Result<SyscallReturn> {
    debug!(
        "[SYS_PIPE2] address: {:#x}, flags: {:#x}",
        pipe_address, flags
    );

    let (reader, writer) = Pipe::new_pair();

    let mut file_table = current_process.file_table();
    let read_fd = file_table.insert(FileEntry::new(reader));
    let write_fd = file_table.insert(FileEntry::new(writer));

    let vm_space = current_process.memory_space().vm_space();
    let mut writer = vm_space.writer(pipe_address, size_of::<PipeFds>()).unwrap();

    writer.write_val(&PipeFds { read_fd, write_fd }).unwrap();

    Ok(SyscallReturn(0))
}
