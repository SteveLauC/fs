//! Functions exposed by  `std::fs` and `std::os::unix::fs`

use crate::{
    backend::{encapsulation, realpath::realpath},
    dir::ReadDir,
    dirbuilder::DirBuilder,
    file::File,
    metadata::Metadata,
    permissions::Permissions,
};
use std::{
    io::{ErrorKind, Read, Result, Write},
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
///
/// This function will overwrite the contents of to.
pub fn copy<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> Result<u64> {
    let from = File::open(from)?;
    let to = File::create(to)?;
    let from_meta = from.metadata()?;
    let from_len = from_meta.len();
    let from_permission = from_meta.permission();

    let num_written =
        encapsulation::copy_file_range(&from, Some(0), &to, Some(0), from_len as usize)?;
    to.set_permissions(from_permission)?;

    Ok(num_written as u64)
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
pub fn hard_link<P: AsRef<Path>, Q: AsRef<Path>>(original: P, link: Q) -> Result<()> {
    encapsulation::link(original.as_ref(), link.as_ref())
}

/// Given a path, query the file system to get information about a
/// file, directory, etc.
pub fn metadata<P: AsRef<Path>>(path: P) -> Result<Metadata> {
    encapsulation::statx(path.as_ref()).map(Metadata)
}

/// Read the entire contents of a file into a bytes vector.
pub fn read<P: AsRef<Path>>(path: P) -> Result<Vec<u8>> {
    let mut file = File::open(path.as_ref())?;
    let metadata = symlink_metadata(path.as_ref())?;
    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    file.read_to_end(&mut bytes)?;
    Ok(bytes)
}

/// Returns an iterator over the entries within a directory.
#[inline]
pub fn read_dir<P: AsRef<Path>>(path: P) -> Result<ReadDir> {
    encapsulation::Dir::opendir(path).map(|dir| ReadDir(dir))
}

/// Reads a symbolic link, returning the file that the link points to.
#[inline]
pub fn read_link<P: AsRef<Path>>(path: P) -> Result<PathBuf> {
    encapsulation::readlink(path.as_ref())
}

/// read_to_string: Read the entire contents of a file into a string.
pub fn read_to_string<P: AsRef<Path>>(path: P) -> Result<String> {
    let mut file = File::open(path)?;
    let mut string = String::new();
    file.read_to_string(&mut string)?;
    Ok(string)
}

/// Removes an empty directory.
#[inline]
pub fn remove_dir<P: AsRef<Path>>(path: P) -> Result<()> {
    encapsulation::rmdir(path)
}

fn _remove_dir_recurisive(path: &Path) -> Result<()> {
    let read_dir = read_dir(path)?;
    for item_res in read_dir {
        let item = item_res?;

        let file_type = item.file_type()?;
        if file_type.is_dir() {
            _remove_dir_recurisive(&item.path())?;
        } else {
            remove_file(item.path())?;
        }
    }

    // remove the directory itself
    remove_dir(path)
}

/// Removes a directory at this path, after removing all its contents. Use
/// carefully!
pub fn remove_dir_all<P: AsRef<Path>>(path: P) -> Result<()> {
    if symlink_metadata(path.as_ref())?.is_symlink() {
        remove_file(path)
    } else {
        _remove_dir_recurisive(path.as_ref())
    }
}

/// Removes a file from the filesystem.
#[inline]
pub fn remove_file<P: AsRef<Path>>(path: P) -> Result<()> {
    encapsulation::unlink(path)
}

/// Rename a file or directory to a new name, replacing the original file
/// if to already exists.
#[inline]
pub fn rename<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> Result<()> {
    encapsulation::rename(from, to)
}

/// Changes the permissions found on a file or a directory.
#[inline]
pub fn set_permissions<P: AsRef<Path>>(path: P, perm: Permissions) -> Result<()> {
    encapsulation::chmod(path, perm.0)
}

/// Query the metadata about a file without following symlinks.
pub fn symlink_metadata<P: AsRef<Path>>(path: P) -> Result<Metadata> {
    encapsulation::lstatx(path.as_ref()).map(Metadata)
}

/// Write a slice as the entire contents of a file.
pub fn write<P: AsRef<Path>, C: AsRef<[u8]>>(path: P, contents: C) -> Result<()> {
    File::create(path)?.write_all(contents.as_ref())
}

/// Change the owner and group of the specified path.
#[inline]
pub fn chown<P: AsRef<Path>>(dir: P, uid: Option<u32>, gid: Option<u32>) -> Result<()> {
    encapsulation::chown(dir, uid, gid)
}

/// Change the owner and group of the file referenced by the specified open file
/// descriptor.
#[inline]
pub fn fchown<F: AsFd>(fd: F, uid: Option<u32>, gid: Option<u32>) -> Result<()> {
    encapsulation::fchown(fd, uid, gid)
}

/// Change the owner and group of the specified path, without
/// dereferencing symbolic links.
#[inline]
pub fn lchown<P: AsRef<Path>>(dir: P, uid: Option<u32>, gid: Option<u32>) -> Result<()> {
    encapsulation::lchown(dir, uid, gid)
}

/// Change the root directory of the current process to the specified path.
#[inline]
pub fn chroot<P: AsRef<Path>>(dir: P) -> Result<()> {
    encapsulation::chroot(dir)
}

/// Creates a new symbolic link on the filesystem.
#[inline]
pub fn symlink<P: AsRef<Path>, Q: AsRef<Path>>(original: P, link: Q) -> Result<()> {
    encapsulation::symlink(original, link)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_copy() {
        let from = "/tmp/test_copy_from";
        let mut from_file = std::fs::File::create(from).unwrap();
        let to = "/tmp/test_copy_to";
        from_file.write(b"hello").unwrap();

        assert_eq!(5, copy(from, to).unwrap());

        remove_file(from).unwrap();
        remove_file(to).unwrap();
    }

    #[test]
    fn test_remove_dir_all() {
        // Create dir
        create_dir("/tmp/dir").unwrap();
        File::create("/tmp/dir/1").unwrap();
        File::create("/tmp/dir/2").unwrap();
        File::create("/tmp/dir/3").unwrap();
        create_dir_all("/tmp/dir/dir2/dir3/dir4").unwrap();

        // delete them
        remove_dir_all("/tmp/dir").unwrap();
    }

    #[test]
    fn test_remove_dir_all_symlink() {
        File::create("/tmp/test_remove_dir_all_symlink").unwrap();
        symlink("/tmp/test_remove_dir_all_symlink", "/tmp/test_remove_dir_all_symlink_link").unwrap();

        remove_dir_all("/tmp/test_remove_dir_all_symlink_link").unwrap();

        remove_file("/tmp/test_remove_dir_all_symlink").unwrap();
    }
}
