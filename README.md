## fs

[![BUILD](https://github.com/stevelauc/pup/workflows/Rust/badge.svg)](https://github.com/stevelauc/fs/actions/workflows/rust.yml)
[![License](http://img.shields.io/badge/license-GPL-orange.svg)](https://github.com/SteveLuaC/extattr/blob/main/LICENSE)

A toy `std::fs` implementation that does NOT depend on `libc` (raw syscall). 
This crate is **ONLY** guaranteed to work on `x86_64-unknown-linux-gnu`, playing 
it on other platforms may elicit **Undefined Behavior**.

> Really a toy implementation, certain features may even not work.

## Table Of Contents

* [Getting Started](https://github.com/SteveLauC/fs#getting-started)
* [Project Hierarchy](https://github.com/SteveLauC/fs#project-hierarchy)
    * [Modules](https://github.com/SteveLauC/fs#modules)
    * [Backend](https://github.com/SteveLauC/fs#backend)
* [Benchmark](https://github.com/SteveLauC/fs#benchmark)
* [Why Build this crate](https://github.com/SteveLauC/fs#why-build-this-crate)

## Getting Started

```toml
[dependencies]
fs = { git = "https://github.com/SteveLauC/fs" }
```

```rust
// Then just use it like you are using the `fs` module from stdlib

use fs::OpenOptions;

fn main() {
    let cwd = OpenOptions::new().read(true).open(".").unwrap();
    println!("{:?}", cwd);
}
```

```shell
$ cargo +nightly r
File { fd: 3, path: "/home/steve/Documents/workspace/rust", read: true, write: false }
```

## Project Hierarchy

#### Modules

The source code of this crate is divided by types. For example, anything related
to `struct File` is placed in `file.rs`):

```shell
$ cargo modules generate tree
crate fs
├── mod backend: pub(crate)
│   ├── mod encapsulation: pub(crate)
│   ├── mod libc_like_syscall: pub(self)
│   ├── mod major_minor: pub(crate)
│   └── mod realpath: pub(crate)
├── mod dir: pub(crate)
├── mod dirbuilder: pub(crate)
├── mod file: pub(crate)
├── mod filetimes: pub(crate)
├── mod filetype: pub(crate)
├── mod functions: pub(crate)
├── mod metadata: pub(crate)
├── mod non_fs: pub
├── mod open_option: pub(crate)
└── mod permissions: pub(crate)
```

All the `pub(crate)` modules that are not under `backend` are re-exported in `lib.rs`:

```rust
pub use dir::*;
pub use dirbuilder::*;
pub use file::*;
pub use filetimes::*;
pub use filetype::*;
pub use functions::*;
pub use metadata::*;
pub use open_option::*;
pub use permissions::*;
```

#### Backend

Since we do not use `libc` in our implementation, manually involving syscall becomes
necessary. You can find these code under directory `backend`:

```shell
$ l src/backend
Permissions Links Size User  Group Date Modified Name
.rw-r--r--@     1  41k steve steve 26 Dec 14:58  encapsulation.rs
.rw-r--r--@     1  24k steve steve 26 Dec 14:58  libc_like_syscall.rs
.rw-r--r--@     1  730 steve steve 26 Dec 14:58  major_minor.rs
.rw-r--r--@     1  309 steve steve 26 Dec 14:58  mod.rs
.rw-r--r--@     1 4.7k steve steve 26 Dec 14:58  realpath.rs
```

To make my life easier, the backend of this project is separated into two parts.
First, I implemented some libc-like syscalls in `libc_like_syscall.rs`, take
`open(2)` as an example:

```c
// Interface exposed by glibc

int open(const char *pathname, int flags, mode_t mode);
```

```rust
// open() in libc_like_syscall.rs

#[inline]
pub(crate) fn open(
    pathname: *const c_char,
    flags: c_int,
    mode: mode_t,
) -> Result<RawFd, c_int> {
    let res =
        unsafe { syscall!(OPEN, pathname as usize, flags as usize, mode) };

    syscall_result(res).map(|fd| fd as RawFd)
}
```

You can find that in the perspective of interface, they are basically equivalent
except the return type.

Then to make these interfaces more rusty, I made another Rusty encapsulation layer
(like `nix` or `rustix`) in `encapsulation.rs`:

```rust
// open in encapsulation.rs

/// Opens a file
///
/// Note: `path` should not contain byte 0, or this function will panic.
pub(crate) fn open<P: AsRef<Path>>(
    path: P,
    flag: Flags,
    mode: Mode,
) -> Result<OwnedFd> {
    let path = CString::new(path.as_ref().as_os_str().as_bytes()).unwrap();
    let flag = flag.bits();
    let mode = mode.bits();

    match libc_like_syscall::open(path.as_ptr(), flag, mode) {
        Ok(raw_fd) => Ok(unsafe { OwnedFd::from_raw_fd(raw_fd) }),
        Err(errno) => Err(Error::from_raw_os_error(errno)),
    }
}
```

As you can see, the types of arguments become rusty and thus less error-prone.

## Benchmark

This crate is "unfortunately" much slower than the stdlib:

|         | Root Dir Iterating | Read a 137M file with buffer set to 10B(Kernel buffer cleared) |
|---------|--------------------|----------------------------------------------------------------|
| std::fs | 20.298µs           | 3.44s                                                          |
| My impl | 27.722µs           | 3.92s                                                          |

## Why build this crate
1. They are some voices in the community stating that we should make the stdlib
   do not depend on libc for better performance. And forks at `rustix` are already 
   working on that.
   
   So I am curious if I can do such things...
   
2. For fun.
