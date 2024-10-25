use ostd::cpu::UserContext;

use crate::{
    prelude::*,
    process::clone::{clone_current_process, CloneArgs},
};

pub fn sys_clone(
    clone_flags: u64,
    child_stack: u64,
    parent_tidptr: Vaddr,
    tls: u64,
    child_tidptr: Vaddr,
    parent_context: &UserContext,
) -> Result<SyscallReturn> {
    let args = CloneArgs::for_clone(clone_flags, parent_tidptr, child_tidptr, tls, child_stack)?;
    debug!("flags = {:?}, child_stack_ptr = 0x{:x}, parent_tid_ptr = 0x{:x?}, child tid ptr = 0x{:x}, tls = 0x{:x}", args.flags, args.stack, args.parent_tid, args.child_tid, args.tls);
    let process = clone_current_process(parent_context, args)?;
    process.run();
    Ok(SyscallReturn::Return(process.pid() as isize))
}
