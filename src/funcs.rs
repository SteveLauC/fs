//! Functions exposed by  `std::fs` and `std::os::unix::fs`

use crate::{
    backend::{encapsulation, realpath::realpath},
    dir::ReadDir,
    dirbuilder::DirBuilder,
    metadata::Metadata,
    permissions::Permissions,
};
use std::{
    io::{ErrorKind, Result},
    os::unix::io::AsFd,
    path::{Path, PathBuf},
};

/// Returns Ok(true) if the path points at an existing entity.
pub fn try_exists<P: AsRef<Path>>(path: P) -> Result<bool> {
    match metadata(path) {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error),
    }
}

/// Returns the canonical, absolute form of a path with all
/// intermediate components normalized and symbolic links resolved.
#[inline]
pub fn canonicalize<P: AsRef<Path>>(path: P) -> Result<PathBuf> {
    realpath(path)
}

/// Copies the contents of one file to another. This function will also copy
/// the permission bits of the original file to the destination file.
pub fn copy<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> Result<u64> {
    unimplemented!()
}

/// create_dir: Creates a new, empty directory at the provided path
#[inline]
pub fn create_dir<P: AsRef<Path>>(path: P) -> Result<()> {
    DirBuilder::new().create(path.as_ref())
}

/// Recursively create a directory and all of its parent
/// components if they are missing.
#[inline]
pub fn create_dir_all<P: AsRef<Path>>(path: P) -> Result<()> {
    DirBuilder::new().recursive(true).create(path.as_ref())
}

/// Creates a new hard link on the filesystem.
#[inline]
pub fn hard_link<P: AsRef<Path>, Q: AsRef<Path>>(
    original: P,
    link: Q,
) -> Result<()> {
    encapsulation::link(original.as_ref(), link.as_ref())
}

/// Given a path, query the file system to get information about a
/// file, directory, etc.
pub fn metadata<P: AsRef<Path>>(path: P) -> Result<Metadata> {
    unimplemented!()
}

/// Read the entire contents of a file into a bytes vector.
pub fn read<P: AsRef<Path>>(path: P) -> Result<Vec<u8>> {
    unimplemented!()
}

/// Returns an iterator over the entries within a directory.
pub fn read_dir<P: AsRef<Path>>(path: P) -> Result<ReadDir> {
    unimplemented!()
}

/// Reads a symbolic link, returning the file that the link points to.
pub fn read_link<P: AsRef<Path>>(path: P) -> Result<PathBuf> {
    unimplemented!()
}

/// read_to_string: Read the entire contents of a file into a string.
pub fn read_to_string<P: AsRef<Path>>(path: P) -> Result<String> {
    unimplemented!()
}

/// Removes an empty directory.
pub fn remove_dir<P: AsRef<Path>>(path: P) -> Result<()> {
    unimplemented!()
}

/// Removes a directory at this path, after removing all its contents. Use
/// carefully!
pub fn remove_dir_all<P: AsRef<Path>>(path: P) -> Result<()> {
    unimplemented!()
}

/// Removes a file from the filesystem.
pub fn remove_file<P: AsRef<Path>>(path: P) -> Result<()> {
    unimplemented!()
}

/// Rename a file or directory to a new name, replacing the original file
/// if to already exists.
pub fn rename<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> Result<()> {
    unimplemented!()
}

/// Changes the permissions found on a file or a directory.
pub fn set_permissions<P: AsRef<Path>>(
    path: P,
    perm: Permissions,
) -> Result<()> {
    unimplemented!()
}

/// Query the metadata about a file without following symlinks.
pub fn symlink_metadata<P: AsRef<Path>>(path: P) -> Result<Metadata> {
    unimplemented!()
}

/// Write a slice as the entire contents of a file.
pub fn write<P: AsRef<Path>, C: AsRef<[u8]>>(
    path: P,
    contents: C,
) -> Result<()> {
    unimplemented!()
}

/// Change the owner and group of the specified path.
pub fn chown<P: AsRef<Path>>(
    dir: P,
    uid: Option<u32>,
    gid: Option<u32>,
) -> Result<()> {
    unimplemented!()
}

/// Change the owner and group of the file referenced by the specified open file
/// descriptor.
pub fn fchown<F: AsFd>(
    fd: F,
    uid: Option<u32>,
    gid: Option<u32>,
) -> Result<()> {
    unimplemented!()
}

/// lchown: Change the owner and group of the specified path, without
/// dereferencing symbolic links.
pub fn lchown<P: AsRef<Path>>(
    dir: P,
    uid: Option<u32>,
    gid: Option<u32>,
) -> Result<()> {
    unimplemented!()
}

/// Change the root directory of the current process to the specified path.
pub fn chroot<P: AsRef<Path>>(dir: P) -> Result<()> {
    unimplemented!()
}

/// Creates a new symbolic link on the filesystem.!
pub fn symlink<P: AsRef<Path>, Q: AsRef<Path>>(
    original: P,
    link: Q,
) -> Result<()> {
    unimplemented!()
}
