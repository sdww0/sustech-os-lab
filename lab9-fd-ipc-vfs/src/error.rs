#![allow(dead_code)]

use int_to_c_enum::TryFromInt;

pub type Result<T, E = Error> = core::result::Result<T, E>;

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromInt)]
pub enum Errno {
    EPERM = 1,         // Operation not permitted
    ENOENT = 2,        // No such file or directory
    ESRCH = 3,         // No such process
    EINTR = 4,         // Interrupted system call
    EIO = 5,           // I/O error
    ENXIO = 6,         // No such device or address
    E2BIG = 7,         // Argument list too long
    ENOEXEC = 8,       // Exec format error
    EBADF = 9,         // Bad file number
    ECHILD = 10,       // No child processes
    EAGAIN = 11,       // Try again
    ENOMEM = 12,       // Out of memory
    EACCES = 13,       // Permission denied
    EFAULT = 14,       // Bad address
    ENOTBLK = 15,      // Block device required
    EBUSY = 16,        // Device or resource busy
    EEXIST = 17,       // File exists
    EXDEV = 18,        // Cross-device link
    ENODEV = 19,       // No such device
    ENOTDIR = 20,      // Not a directory
    EISDIR = 21,       // Is a directory
    EINVAL = 22,       // Invalid argument
    ENFILE = 23,       // File table overflow
    EMFILE = 24,       // Too many open files
    ENOTTY = 25,       // Not a typewriter
    ETXTBSY = 26,      // Text file busy
    EFBIG = 27,        // File too large
    ENOSPC = 28,       // No space left on device
    ESPIPE = 29,       // Illegal seek
    EROFS = 30,        // Read-only file system
    EMLINK = 31,       // Too many links
    EPIPE = 32,        // Broken pipe
    EDOM = 33,         // Math argument out of domain of func
    ERANGE = 34,       // Math result not representable
    EDEADLK = 35,      // Resource deadlock would occur
    ENAMETOOLONG = 36, // File name too long
    ENOLCK = 37,       // No record locks available
    ENOSYS = 38,       // Function not implemented
    ENOTEMPTY = 39,    // Directory not empty
    ELOOP = 40,        // Too many symbolic links encountered
    ENOMSG = 42,       // No message of desired type
    EIDRM = 43,        // Identifier removed
    ECHRNG = 44,       // Channel number out of range
    EL2NSYNC = 45,     // Level 2 not synchronized
    EL3HLT = 46,       // Level 3 halted
    EL3RST = 47,       // Level 3 reset
    ELNRNG = 48,       // Link number out of range
    EUNATCH = 49,      // Protocol driver not attached
    ENOCSI = 50,       // No CSI structure available
    EL2HLT = 51,       // Level 2 halted
}

#[derive(Debug)]
pub struct Error {
    pub code: Errno,
    pub message: &'static str,
}

impl Error {
    pub fn new(code: Errno) -> Self {
        Self { code, message: "" }
    }

    pub fn new_with_message(code: Errno, message: &'static str) -> Self {
        Self { code, message }
    }

    pub fn code(&self) -> i32 {
        self.code as i32
    }

    pub fn message(&self) -> &'static str {
        self.message
    }
}
