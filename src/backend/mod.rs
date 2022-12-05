//! Backend for our fs implementation
//!
//! Our backend is divided into three parts:
//! 1. libc-like syscalls.
//! 2. Rusty encapsulations for those libc-like syscalls.
//! 3. Some library functions

pub(crate) mod encapsulation;
mod libc_like_syscall;
pub(crate) mod major_minor;
pub(crate) mod realpath;
