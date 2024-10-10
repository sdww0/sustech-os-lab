//

use crate::{
    error::{Errno, Error},
    mm::VmPerms,
    prelude::*,
    process::current_process,
};
use align_ext::AlignExt;
use log::debug;
use ostd::mm::{Vaddr, PAGE_SIZE};

use super::SyscallReturn;

pub fn sys_mprotect(addr: Vaddr, len: usize, perms: u64) -> Result<SyscallReturn> {
    let vm_perms = VmPerms::from_bits_truncate(perms as u32);
    debug!(
        "addr = 0x{:x}, len = 0x{:x}, perms = {:?}",
        addr, len, vm_perms
    );

    if len == 0 {
        return Ok(SyscallReturn::Return(0));
    }

    let len = len.align_up(PAGE_SIZE);
    let end = addr.checked_add(len).ok_or(Error::with_message(
        Errno::ENOMEM,
        "integer overflow when (addr + len)",
    ))?;
    let range = addr..end;

    let process = current_process().unwrap();
    let user_space = process.user_space().unwrap();
    let mut cursor = user_space.vm_space().cursor_mut(&range).unwrap();

    cursor.protect_next(len, |p| p.flags = vm_perms.into());
    Ok(SyscallReturn::Return(0))
}
