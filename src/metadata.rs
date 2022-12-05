use crate::{
    backend::encapsulation::Statx, filetype::FileType, permissions::Permissions,
};
use std::{
    io::Result,
    os::linux::{fs::MetadataExt, raw::stat},
    time::SystemTime,
};

pub struct Metadata(pub(crate) Statx);

impl Metadata {
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
        unimplemented!()
    }

    pub fn modified(&self) -> Result<SystemTime> {
        unimplemented!()
    }

    pub fn accessed(&self) -> Result<SystemTime> {
        unimplemented!()
    }

    pub fn created(&self) -> Result<SystemTime> {
        unimplemented!()
    }
}

impl MetadataExt for Metadata {
    fn as_raw_stat(&self) -> &stat {
        todo!()
    }
    fn st_dev(&self) -> u64 {
        todo!()
    }

    fn st_ino(&self) -> u64 {
        todo!()
    }

    fn st_mode(&self) -> u32 {
        todo!()
    }
    fn st_nlink(&self) -> u64 {
        todo!()
    }
    fn st_uid(&self) -> u32 {
        todo!()
    }

    fn st_gid(&self) -> u32 {
        todo!()
    }
    fn st_rdev(&self) -> u64 {
        todo!()
    }

    fn st_size(&self) -> u64 {
        todo!()
    }
    fn st_atime(&self) -> i64 {
        todo!()
    }
    fn st_atime_nsec(&self) -> i64 {
        todo!()
    }
    fn st_mtime(&self) -> i64 {
        todo!()
    }
    fn st_mtime_nsec(&self) -> i64 {
        todo!()
    }
    fn st_ctime(&self) -> i64 {
        todo!()
    }
    fn st_ctime_nsec(&self) -> i64 {
        todo!()
    }
    fn st_blksize(&self) -> u64 {
        todo!()
    }
    fn st_blocks(&self) -> u64 {
        todo!()
    }
}
