use crate::{
    backend::{encapsulation::Statx, major_minor::makedev},
    filetype::FileType,
    non_fs::SystemTime,
    permissions::Permissions,
};
#[allow(deprecated)]
use std::os::linux::raw::stat;
use std::{
    io::Result,
    os::{linux::fs::MetadataExt, unix::fs::PermissionsExt},
};

/// Metadata information about a file.
///
/// This structure is returned from the metadata or symlink_metadata function
/// or method and represents known metadata about a file such as its permissions,
/// size, modification times, etc.
#[derive(Clone)]
pub struct Metadata(pub(crate) Statx);

impl Metadata {
    /// Returns the file type for this metadata.
    #[inline]
    pub fn file_type(&self) -> FileType {
        FileType(self.0.file_type())
    }

    /// Returns true if this metadata is for a directory. The result is mutually
    /// exclusive to the result of `Metadata::is_file`, and will be false for
    /// symlink metadata obtained from `symlink_metadata`.
    #[inline]
    pub fn is_dir(&self) -> bool {
        self.file_type().is_dir()
    }

    /// Returns true if this metadata is for a regular file. The result is
    /// mutually exclusive to the result of `Metadata::is_dir`, and will be
    /// false for symlink metadata obtained from `symlink_metadata`.
    ///
    /// When the goal is simply to read from (or write to) the source, the
    /// most reliable way to test the source can be read (or written to) is
    /// to open it. Only using `is_file` can break workflows like
    /// `diff <( prog_a )` on a Unix-like system for example. See `File::open`
    /// or `OpenOptions::open` for more information.
    #[inline]
    pub fn is_file(&self) -> bool {
        self.file_type().is_file()
    }

    /// Returns true if this metadata is for a symbolic link.
    #[inline]
    pub fn is_symlink(&self) -> bool {
        self.file_type().is_symlink()
    }

    /// Returns the size of the file, in bytes, this metadata is for.
    #[inline]
    pub fn len(&self) -> u64 {
        self.0.size()
    }

    /// Returns the permissions of the file this metadata is for.
    pub fn permission(&self) -> Permissions {
        Permissions::from_mode(self.0.mode())
    }

    /// Returns the last modification time listed in this metadata.

    /// The returned value corresponds to the mtime field of stat on Unix platforms
    /// and the ftLastWriteTime field on Windows platforms.
    ///
    /// # Errors
    /// This field might not be available on all platforms, and will return an
    /// Err on platforms where it is not available.
    #[inline]
    pub fn modified(&self) -> Result<SystemTime> {
        Ok(SystemTime::new(self.0.mtime().0, self.0.mtime().1 as i64))
    }

    /// Returns the last access time of this metadata.

    /// The returned value corresponds to the `atime` field of `stat` on Unix platforms
    /// and the `ftLastAccessTime` field on Windows platforms.
    ///
    /// Note that not all platforms will keep this field update in a file’s
    /// metadata, for example Windows has an option to disable updating this
    /// time when files are accessed and Linux similarly has noatime.
    ///
    /// # Errors
    /// This field might not be available on all platforms, and will return an
    /// Err on platforms where it is not available.
    #[inline]
    pub fn accessed(&self) -> Result<SystemTime> {
        Ok(SystemTime::new(self.0.atime().0, self.0.atime().1 as i64))
    }

    /// Returns the creation time listed in this metadata.
    ///
    /// The returned value corresponds to the `btime` field of `statx` on Linux
    /// kernel starting from to 4.11, the `birthtime` field of `stat` on other
    /// Unix platforms, and the `ftCreationTime` field on Windows platforms.
    ///
    /// # Errors
    /// This field might not be available on all platforms, and will return an
    /// Err on platforms or filesystems where it is not available.
    #[inline]
    pub fn created(&self) -> Result<SystemTime> {
        Ok(SystemTime::new(self.0.ctime().0, self.0.ctime().1 as i64))
    }
}

impl MetadataExt for Metadata {
    #[allow(deprecated)]
    fn as_raw_stat(&self) -> &stat {
        unimplemented!("This API is deprecated!")
    }

    #[inline]
    fn st_dev(&self) -> u64 {
        makedev(self.0.dev().0, self.0.dev().1)
    }

    #[inline]
    fn st_ino(&self) -> u64 {
        self.0.ino()
    }

    #[inline]
    fn st_mode(&self) -> u32 {
        self.0.mode()
    }

    #[inline]
    fn st_nlink(&self) -> u64 {
        self.0.nlink() as u64
    }

    #[inline]
    fn st_uid(&self) -> u32 {
        self.0.uid()
    }

    #[inline]
    fn st_gid(&self) -> u32 {
        self.0.gid()
    }

    #[inline]
    fn st_rdev(&self) -> u64 {
        makedev(self.0.rdev().0, self.0.rdev().1)
    }

    #[inline]
    fn st_size(&self) -> u64 {
        self.0.size()
    }

    #[inline]
    fn st_atime(&self) -> i64 {
        self.0.atime().0
    }

    #[inline]
    fn st_atime_nsec(&self) -> i64 {
        self.0.atime().1 as i64
    }

    #[inline]
    fn st_mtime(&self) -> i64 {
        self.0.mtime().0
    }

    #[inline]
    fn st_mtime_nsec(&self) -> i64 {
        self.0.mtime().1 as i64
    }

    #[inline]
    fn st_ctime(&self) -> i64 {
        self.0.ctime().0
    }

    #[inline]
    fn st_ctime_nsec(&self) -> i64 {
        self.0.ctime().1 as i64
    }

    #[inline]
    fn st_blksize(&self) -> u64 {
        self.0.blksize() as u64
    }

    #[inline]
    fn st_blocks(&self) -> u64 {
        self.0.blocks()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::File;

    #[test]
    fn metadata_ext() {
        let stat = nix::sys::stat::stat("Cargo.toml").unwrap();
        let metadata = File::open("Cargo.toml").unwrap().metadata().unwrap();

        assert_eq!(stat.st_dev, metadata.st_dev());
        assert_eq!(stat.st_ino, metadata.st_ino());
        assert_eq!(stat.st_mode, metadata.st_mode());
        assert_eq!(stat.st_nlink, metadata.st_nlink());
        assert_eq!(stat.st_uid, metadata.st_uid());
        assert_eq!(stat.st_gid, metadata.st_gid());
        assert_eq!(stat.st_rdev, metadata.st_rdev());
        assert_eq!(stat.st_size as u64, metadata.st_size());
        assert_eq!(stat.st_atime, metadata.st_atime());
        assert_eq!(stat.st_atime_nsec, metadata.st_atime_nsec());
        assert_eq!(stat.st_mtime, metadata.st_mtime());
        assert_eq!(stat.st_mtime_nsec, metadata.st_mtime_nsec());
        assert_eq!(stat.st_ctime, metadata.st_ctime());
        assert_eq!(stat.st_ctime_nsec, metadata.st_ctime_nsec());
        assert_eq!(stat.st_blksize as u64, metadata.st_blksize());
        assert_eq!(stat.st_blocks as u64, metadata.st_blocks());
    }
}
