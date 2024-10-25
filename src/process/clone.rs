// SPDX-License-Identifier: MPL-2.0
// Ref: asterinas: kernel/src/process/clone.rs

use core::num::NonZeroU64;

use crate::prelude::*;
use alloc::sync::Arc;
use bitflags::bitflags;
use ostd::cpu::UserContext;

use super::{current_process, signal::SIGCHLD, Process};

bitflags! {
    #[derive(Default)]
    pub struct CloneFlags: u32 {
        const CLONE_VM      = 0x00000100;       /* Set if VM shared between processes.  */
        const CLONE_FS      = 0x00000200;       /* Set if fs info shared between processes.  */
        const CLONE_FILES   = 0x00000400;       /* Set if open files shared between processes.  */
        const CLONE_SIGHAND = 0x00000800;       /* Set if signal handlers shared.  */
        const CLONE_PIDFD   = 0x00001000;       /* Set if a pidfd should be placed in parent.  */
        const CLONE_PTRACE  = 0x00002000;       /* Set if tracing continues on the child.  */
        const CLONE_VFORK   = 0x00004000;       /* Set if the parent wants the child to wake it up on mm_release.  */
        const CLONE_PARENT  = 0x00008000;       /* Set if we want to have the same parent as the cloner.  */
        const CLONE_THREAD  = 0x00010000;       /* Set to add to same thread group.  */
        const CLONE_NEWNS   = 0x00020000;       /* Set to create new namespace.  */
        const CLONE_SYSVSEM = 0x00040000;       /* Set to shared SVID SEM_UNDO semantics.  */
        const CLONE_SETTLS  = 0x00080000;       /* Set TLS info.  */
        const CLONE_PARENT_SETTID = 0x00100000; /* Store TID in userlevel buffer before MM copy.  */
        const CLONE_CHILD_CLEARTID = 0x00200000;/* Register exit futex and memory location to clear.  */
        const CLONE_DETACHED = 0x00400000;      /* Create clone detached.  */
        const CLONE_UNTRACED = 0x00800000;      /* Set if the tracing process can't force CLONE_PTRACE on this clone.  */
        const CLONE_CHILD_SETTID = 0x01000000;  /* Store TID in userlevel buffer in the child.  */
        const CLONE_NEWCGROUP   = 0x02000000;	/* New cgroup namespace.  */
        const CLONE_NEWUTS	= 0x04000000;	    /* New u tsname group.  */
        const CLONE_NEWIPC	= 0x08000000;	    /* New ipcs.  */
        const CLONE_NEWUSER	= 0x10000000;	    /* New user namespace.  */
        const CLONE_NEWPID	= 0x20000000;	    /* New pid namespace.  */
        const CLONE_NEWNET	= 0x40000000;	    /* New network namespace.  */
        const CLONE_IO	= 0x80000000;	        /* Clone I/O context.  */
    }
}

/// An internal structure to homogenize the arguments for `clone` and
/// `clone3`.
///
/// From the clone(2) man page:
///
/// ```
/// The following table shows the equivalence between the arguments
/// of clone() and the fields in the clone_args argument supplied to
/// clone3():
///     clone()         clone3()        Notes
///                     cl_args field
///     flags & ~0xff   flags           For most flags; details
///                                     below
///     parent_tid      pidfd           See CLONE_PIDFD
///     child_tid       child_tid       See CLONE_CHILD_SETTID
///     parent_tid      parent_tid      See CLONE_PARENT_SETTID
///     flags & 0xff    exit_signal
///     stack           stack
///     ---             stack_size
///     tls             tls             See CLONE_SETTLS
///     ---             set_tid         See below for details
///     ---             set_tid_size
///     ---             cgroup          See CLONE_INTO_CGROUP
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct CloneArgs {
    pub flags: CloneFlags,
    pub _pidfd: Option<u64>,
    pub child_tid: Vaddr,
    pub parent_tid: Option<Vaddr>,
    pub exit_signal: Option<u8>,
    pub stack: u64,
    pub stack_size: Option<NonZeroU64>,
    pub tls: u64,
    pub _set_tid: Option<u64>,
    pub _set_tid_size: Option<u64>,
    pub _cgroup: Option<u64>,
}

impl CloneArgs {
    /// Prepares a new [`CloneArgs`] based on the arguments for clone(2).
    pub fn for_clone(
        raw_flags: u64,
        parent_tid: Vaddr,
        child_tid: Vaddr,
        tls: u64,
        stack: u64,
    ) -> Result<Self> {
        const FLAG_MASK: u64 = 0xff;
        let flags = CloneFlags::from(raw_flags & !FLAG_MASK);
        let exit_signal = raw_flags & FLAG_MASK;
        // Disambiguate the `parent_tid` parameter. The field is used
        // both for `CLONE_PIDFD` and `CLONE_PARENT_SETTID`, so at
        // most only one can be specified.
        let (pidfd, parent_tid) = match (
            flags.contains(CloneFlags::CLONE_PIDFD),
            flags.contains(CloneFlags::CLONE_PARENT_SETTID),
        ) {
            (false, false) => (None, None),
            (true, false) => (Some(parent_tid as u64), None),
            (false, true) => (None, Some(parent_tid)),
            (true, true) => {
                return Err(Error::new(Errno::EINVAL));
            }
        };

        Ok(Self {
            flags,
            _pidfd: pidfd,
            child_tid,
            parent_tid,
            exit_signal: Some(exit_signal as u8),
            stack,
            tls,
            ..Default::default()
        })
    }

    pub fn for_fork() -> Self {
        Self {
            exit_signal: Some(SIGCHLD),
            ..Default::default()
        }
    }
}

impl From<u64> for CloneFlags {
    fn from(flags: u64) -> Self {
        // We use the lower 32 bits
        let clone_flags = (flags & 0xffff_ffff) as u32;
        CloneFlags::from_bits_truncate(clone_flags)
    }
}

impl CloneFlags {
    fn check_unsupported_flags(&self) -> Result<()> {
        let supported_flags = CloneFlags::CLONE_CHILD_SETTID | CloneFlags::CLONE_CHILD_CLEARTID;
        let unsupported_flags = *self - supported_flags;
        if !unsupported_flags.is_empty() {
            panic!("contains unsupported clone flags: {:?}", unsupported_flags);
        }
        Ok(())
    }
}

pub fn clone_current_process(
    parent_context: &UserContext,
    clone_args: CloneArgs,
) -> Result<Arc<Process>> {
    let current_process = current_process().unwrap();

    // Clone VM
    let new_memory_space = current_process.memory_space.duplicate_with_new_frames();

    // Clone User Context
    let mut context = *parent_context;
    context.set_a0(0);
    debug!("User context:{:#x?}", context);

    let process = Process::raw_new_user_process(
        context,
        new_memory_space,
        &current_process.heap,
        current_process.name(),
    );

    // Deal with the Clone flags
    let flags = clone_args.flags;
    flags.check_unsupported_flags()?;
    let threads = process.threads();
    if flags.contains(CloneFlags::CLONE_CHILD_CLEARTID) {
        let main_thread = threads.first().unwrap();
        *main_thread.clear_child_tid() = clone_args.child_tid;
    }
    if flags.contains(CloneFlags::CLONE_CHILD_SETTID) {
        let main_thread = threads.first().unwrap();
        *main_thread.set_child_tid() = clone_args.child_tid;
    }
    drop(threads);

    process.set_parent_process(&current_process);

    Ok(process)
}
