//! Backend for our fs implementation
//!
//! Our backend is divided into two parts: 
//! 1. libc-like syscalls.
//! 2. Rusty encapsulations for those libc-like syscalls.

mod encapsulation;
mod libc_like_syscall;
