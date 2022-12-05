use crate::{
    filetimes::FileTimes, metadata::Metadata, open_option::OpenOptions,
    permissions::Permissions,
};
use std::{io::Result, os::unix::io::OwnedFd, path::Path, time::SystemTime};

#[derive(Debug)]
pub struct File {
    pub(crate) fd: OwnedFd,
}

impl File {
    /// Attempts to open a file in read-only mode.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<File> {
        OpenOptions::new().read(true).open(path.as_ref())
    }

    /// Opens a file in write-only mode.
    pub fn create<P: AsRef<Path>>(path: P) -> Result<File> {
        OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path.as_ref())
    }

    /// Creates a new file in read-write mode; error if the file exists.
    pub fn create_new<P: AsRef<Path>>(path: P) -> Result<File> {
        OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open(path.as_ref())
    }

    /// Returns a new OpenOptions object.
    #[inline]
    pub fn options() -> OpenOptions {
        OpenOptions::new()
    }

    /// Attempts to sync all OS-internal metadata to disk.
    pub fn sync_all(&self) -> Result<()> {
        unimplemented!()
    }

    /// This function is similar to [`sync_all`], except that it might not
    /// synchronize file metadata to the filesystem.
    pub fn sync_data(&self) -> Result<()> {
        unimplemented!()
    }

    /// Truncates or extends the underlying file, updating the size of
    /// this file to become `size`.
    pub fn set_len(&self, size: u64) -> Result<()> {
        unimplemented!()
    }

    /// Queries metadata about the underlying file.
    pub fn metadata(&self) -> Result<Metadata> {
        unimplemented!()
    }

    /// Creates a new `File` instance that shares the same underlying file handle
    /// as the existing `File` instance. Reads, writes, and seeks will affect
    /// both `File` instances simultaneously.
    //
    // Duplicate the underlying file descriptor
    pub fn try_clone(&self) -> Result<File> {
        unimplemented!()
    }

    /// Changes the permissions on the underlying file.
    pub fn set_permissions(&self, perm: Permissions) -> Result<()> {
        unimplemented!()
    }

    /// Changes the timestamps of the underlying file.
    pub fn set_times(&self, times: FileTimes) -> Result<()> {
        unimplemented!()
    }

    /// Changes the modification time of the underlying file.
    ///
    /// This is an alias for `set_times(FileTimes::new().set_modified(time))`.
    #[inline]
    pub fn set_modified(&self, time: SystemTime) -> Result<()> {
        self.set_times(FileTimes::new().set_modified(time))
    }
}
