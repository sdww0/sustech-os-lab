#![allow(unused_imports)]

pub(crate) use crate::{error::Errno, error::Error, syscall::SyscallReturn};

pub(crate) type Result<T> = core::result::Result<T, Error>;
pub(crate) use ostd::mm::Vaddr;

pub(crate) use log::{debug, error, info, trace, warn};
