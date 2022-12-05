use crate::{
    backend::{encapsulation::Statx, major_minor::makedev},
    filetype::FileType,
    non_fs::SystemTime,
    permissions::Permissions,
};
use std::{
    io::Result,
    os::{
        linux::{fs::MetadataExt, raw::stat},
        unix::fs::PermissionsExt,
    },
};

#[derive(Clone)]
pub struct Metadata(pub(crate) Statx);

impl Metadata {
    #[inline]
    pub fn file_type(&self) -> FileType {
        FileType(self.0.file_type())
    }

    #[inline]
    pub fn is_dir(&self) -> bool {
        self.file_type().is_dir()
    }

    #[inline]
    pub fn is_file(&self) -> bool {
        self.file_type().is_file()
    }

    #[inline]
    pub fn is_symlink(&self) -> bool {
        self.file_type().is_symlink()
    }

    #[inline]
    pub fn len(&self) -> u64 {
        self.0.size()
    }

    pub fn permission(&self) -> Permissions {
        Permissions::from_mode(self.0.mode())
    }

    #[inline]
    pub fn modified(&self) -> Result<SystemTime> {
        Ok(SystemTime::new(self.0.mtime().0, self.0.mtime().1 as i64))
    }

    #[inline]
    pub fn accessed(&self) -> Result<SystemTime> {
        Ok(SystemTime::new(self.0.atime().0, self.0.atime().1 as i64))
    }

    #[inline]
    pub fn created(&self) -> Result<SystemTime> {
        Ok(SystemTime::new(self.0.ctime().0, self.0.ctime().1 as i64))
    }
}

impl MetadataExt for Metadata {
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
