//! Functions exposed by  `std::fs` and `std::os::unix::fs`

//! try_exists: Returns Ok(true) if the path points at an existing entity.
//!
//! canonicalize: Returns the canonical, absolute form of a path with all
//! intermediate components normalized and symbolic links resolved.
//!
//! copy: Copies the contents of one file to another. This function will also copy
//! the permission bits of the original file to the destination file.
//!
//! create_dir: Creates a new, empty directory at the provided path
//!
//! create_dir_all: Recursively create a directory and all of its parent
//! components if they are missing.
//!
//! hard_link: Creates a new hard link on the filesystem.
//!
//! metadata: Given a path, query the file system to get information about a
//! file, directory, etc.
//!
//! read: Read the entire contents of a file into a bytes vector.
//!
//! read_dir: Returns an iterator over the entries within a directory.
//!
//! read_link: Reads a symbolic link, returning the file that the link points
//! to.
//!
//! read_to_string: Read the entire contents of a file into a string.
//!
//! remove_dir: Removes an empty directory.
//!
//! remove_dir_all: Removes a directory at this path, after removing all its
//! contents. Use carefully!
//!
//! remove_file: Removes a file from the filesystem.
//!
//! rename: Rename a file or directory to a new name, replacing the original file
//! if to already exists.
//!
//! set_permissions: Changes the permissions found on a file or a directory.
//!
//! symlink_metadata: Query the metadata about a file without following symlinks.
//!
//! write:  Write a slice as the entire contents of a file.
//!
//! chown: Change the owner and group of the specified path.
//!
//! fchown: Change the owner and group of the file referenced by the specified open file descriptor.
//!
//! lchown: Change the owner and group of the specified path, without dereferencing symbolic links.
//!
//! chroot:	Change the root directory of the current process to the specified path.
//!
//! symlink: Creates a new symbolic link on the filesystem.!

use crate::{dir::ReadDir, metadata::Metadata, permissions::Permissions};
use std::{
    io::Result,
    os::unix::io::AsFd,
    path::{Path, PathBuf},
};

pub fn try_exists<P: AsRef<Path>>(path: P) -> Result<bool> {
    unimplemented!()
}

pub fn canonicalize<P: AsRef<Path>>(path: P) -> Result<PathBuf> {
    unimplemented!()
}

pub fn copy<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> Result<u64> {
    unimplemented!()
}

pub fn create_dir<P: AsRef<Path>>(path: P) -> Result<()> {
    unimplemented!()
}

pub fn create_dir_all<P: AsRef<Path>>(path: P) -> Result<()> {
    unimplemented!()
}

pub fn hard_link<P: AsRef<Path>, Q: AsRef<Path>>(
    original: P,
    link: Q,
) -> Result<()> {
    unimplemented!()
}

pub fn metadata<P: AsRef<Path>>(path: P) -> Result<Metadata> {
    unimplemented!()
}

pub fn read<P: AsRef<Path>>(path: P) -> Result<Vec<u8>> {
    unimplemented!()
}

pub fn read_dir<P: AsRef<Path>>(path: P) -> Result<ReadDir> {
    unimplemented!()
}

pub fn read_link<P: AsRef<Path>>(path: P) -> Result<PathBuf> {
    unimplemented!()
}

pub fn read_to_string<P: AsRef<Path>>(path: P) -> Result<String> {
    unimplemented!()
}

pub fn remove_dir<P: AsRef<Path>>(path: P) -> Result<()> {
    unimplemented!()
}

pub fn remove_dir_all<P: AsRef<Path>>(path: P) -> Result<()> {
    unimplemented!()
}

pub fn remove_file<P: AsRef<Path>>(path: P) -> Result<()> {
    unimplemented!()
}

pub fn rename<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> Result<()> {
    unimplemented!()
}

pub fn set_permissions<P: AsRef<Path>>(
    path: P,
    perm: Permissions,
) -> Result<()> {
    unimplemented!()
}

pub fn symlink_metadata<P: AsRef<Path>>(path: P) -> Result<Metadata> {
    unimplemented!()
}

pub fn write<P: AsRef<Path>, C: AsRef<[u8]>>(
    path: P,
    contents: C,
) -> Result<()> {
    unimplemented!()
}

pub fn chown<P: AsRef<Path>>(
    dir: P,
    uid: Option<u32>,
    gid: Option<u32>,
) -> Result<()> {
    unimplemented!()
}

pub fn fchown<F: AsFd>(
    fd: F,
    uid: Option<u32>,
    gid: Option<u32>,
) -> Result<()> {
    unimplemented!()
}

pub fn lchown<P: AsRef<Path>>(
    dir: P,
    uid: Option<u32>,
    gid: Option<u32>,
) -> Result<()> {
    unimplemented!()
}

pub fn chroot<P: AsRef<Path>>(dir: P) -> Result<()> {
    unimplemented!()
}

pub fn symlink<P: AsRef<Path>, Q: AsRef<Path>>(
    original: P,
    link: Q,
) -> Result<()> {
    unimplemented!()
}
