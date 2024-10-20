use crate::{
    context::ReadCString,
    prelude::*,
    process::{current_process, process::load_elf_to_vm_and_context, Process},
};
use alloc::sync::Arc;
use ostd::cpu::UserContext;

const MAX_FILENAME_LEN: usize = 4096;

pub fn sys_execve(
    filename_ptr: Vaddr,
    _argv_ptr_ptr: Vaddr,
    _envp_ptr_ptr: Vaddr,
    user_context: &mut UserContext,
) -> Result<SyscallReturn> {
    let current_process = current_process().unwrap();
    let file_name = {
        let mut reader = current_process
            .user_space()
            .unwrap()
            .vm_space()
            .reader(filename_ptr, MAX_FILENAME_LEN)?;
        reader.read_cstring()?.into_string().unwrap()
    };

    // TODO: Support argv and envp.
    debug!("[SYS_EXECVE] filename: {:?}", file_name);
    let execute_binary = crate::fs::lookup_file(file_name)?;

    do_execve(current_process, execute_binary, user_context)?;
    Ok(SyscallReturn::NoReturn)
}

fn do_execve(
    current_process: Arc<Process>,
    execute_binary: &'static [u8],
    current_user_context: &mut UserContext,
) -> Result<()> {
    let memory_space = current_process.memory_space();
    load_elf_to_vm_and_context(execute_binary, memory_space, current_user_context);
    Ok(())
}
