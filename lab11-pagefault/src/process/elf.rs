use align_ext::AlignExt;
use alloc::sync::Arc;
use log::debug;
use ostd::{
    arch::cpu::context::UserContext,
    mm::{PAGE_SIZE, PageFlags, VmIo},
    user::UserContextApi,
};

use crate::{
    mm::{MemorySpace, area::VmArea, fault::AllocationPageFaultHandler},
    process::USER_STACK_SIZE,
};

pub fn load_user_space(program: &[u8], memory_space: &MemorySpace) -> UserContext {
    let mut user_context = UserContext::default();
    parse_elf(program, &memory_space, &mut user_context);
    user_context
}

pub fn create_user_space(program: &[u8]) -> (MemorySpace, UserContext) {
    let memory_space = MemorySpace::new();
    let mut user_context = UserContext::default();

    parse_elf(program, &memory_space, &mut user_context);
    (memory_space, user_context)
}

fn parse_elf(input: &[u8], memory_space: &MemorySpace, user_cpu_state: &mut UserContext) {
    let header = xmas_elf::header::parse_header(input).unwrap();

    let pt2 = header.pt2;
    let ph_count = pt2.ph_count();

    // First, map each ph
    for index in 0..ph_count {
        let program_header = xmas_elf::program::parse_program_header(input, header, index).unwrap();
        let ph64 = match program_header {
            xmas_elf::program::ProgramHeader::Ph64(ph64) => *ph64,
            xmas_elf::program::ProgramHeader::Ph32(_) => {
                todo!("Not 64 byte executable")
            }
        };
        if let Ok(typ) = ph64.get_type() {
            if typ == xmas_elf::program::Type::Load {
                let raw_start_addr = ph64.virtual_addr;
                let raw_end_addr = ph64.virtual_addr + ph64.mem_size;

                let start_addr = (raw_start_addr as usize).align_down(PAGE_SIZE);
                let end_addr = (raw_end_addr as usize).align_up(PAGE_SIZE);

                debug!(
                    "Mapping elf, raw_start_addr: {:x?}, raw_end_addr: {:x?}, mem_size: {:x?}, file_size: {:x?}",
                    raw_start_addr, raw_end_addr, ph64.mem_size, ph64.file_size
                );

                let mut perms = PageFlags::empty();
                if ph64.flags.is_execute() {
                    perms |= PageFlags::X;
                }
                if ph64.flags.is_read() {
                    perms |= PageFlags::R;
                }
                if ph64.flags.is_write() {
                    perms |= PageFlags::W;
                }
                // __stack_chk_fail
                let nframes = (end_addr - start_addr) / PAGE_SIZE;
                let frames = memory_space.map(VmArea::new(start_addr, nframes, perms));

                let copy_bytes =
                    &input[ph64.offset as usize..(ph64.offset + ph64.file_size) as usize];

                frames
                    .write_bytes(raw_start_addr as usize - start_addr, copy_bytes)
                    .unwrap();
            }
        }
    }

    // Second, init the user stack with addr: 0x40_0000_0000 - 10 * PAGE_SIZE.
    let stack_low = 0x40_0000_0000 - 10 * PAGE_SIZE - USER_STACK_SIZE;
    memory_space.add_area(VmArea::new_with_handler(
        stack_low,
        USER_STACK_SIZE / PAGE_SIZE,
        PageFlags::RW,
        Arc::new(AllocationPageFaultHandler),
    ));
    user_cpu_state.set_stack_pointer(0x40_0000_0000 - 10 * PAGE_SIZE - 32);
    user_cpu_state.set_instruction_pointer(header.pt2.entry_point() as usize);

    // Third, map the 0 address
    memory_space.map(VmArea::new(0, 1, PageFlags::RW));
}
