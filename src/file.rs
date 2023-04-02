use crate::{
    backend::encapsulation, filetimes::FileTimes, functions::read_link, metadata::Metadata,
    non_fs::SystemTime, open_option::OpenOptions, permissions::Permissions,
};
use std::{
    fmt::{self, Debug, Formatter},
    io::{Read, Result, Seek, SeekFrom, Write},
    os::{
        fd::{BorrowedFd, FromRawFd, IntoRawFd, RawFd},
        unix::{
            fs::FileExt,
            io::{AsFd, AsRawFd, OwnedFd},
        },
    },
    path::{Path, PathBuf},
    process::Stdio,
};

/// An object providing access to an open file on the filesystem.
/// 
/// An instance of a File can be read and/or written depending on what options 
/// it was opened with. Files also implement Seek to alter the logical cursor 
/// that the file contains internally.
///
/// Files are automatically closed when they go out of scope. Errors detected on
/// closing are ignored by the implementation of Drop. Use the method sync_all if
/// these errors must be manually handled.
pub struct File {
    pub(crate) fd: OwnedFd,
}

impl File {
    /// Attempts to open a file in read-only mode.
    #[inline]
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
    #[inline]
    pub fn sync_all(&self) -> Result<()> {
        encapsulation::fsync(&self.fd.as_fd())
    }

    /// This function is similar to [`sync_all`], except that it might not
    /// synchronize file metadata to the filesystem.
    ///
    /// [`sync_all`]: File::sync_all
    #[inline]
    pub fn sync_data(&self) -> Result<()> {
        encapsulation::fdatasync(&self.fd.as_fd())
    }

    /// Truncates or extends the underlying file, updating the size of
    /// this file to become `size`.
    #[inline]
    pub fn set_len(&self, size: u64) -> Result<()> {
        encapsulation::ftruncate(&self.fd.as_fd(), size)
    }

    /// Queries metadata about the underlying file.
    #[inline]
    pub fn metadata(&self) -> Result<Metadata> {
        let statx = encapsulation::fstatx(&self.fd.as_fd())?;
        Ok(Metadata(statx))
    }

    /// Creates a new `File` instance that shares the same underlying file handle
    /// as the existing `File` instance. Reads, writes, and seeks will affect
    /// both `File` instances simultaneously.
    //
    // Duplicate the underlying file descriptor
    #[inline]
    pub fn try_clone(&self) -> Result<File> {
        Ok(File {
            fd: self.fd.try_clone()?,
        })
    }

    /// Changes the permissions on the underlying file.
    #[inline]
    pub fn set_permissions(&self, perm: Permissions) -> Result<()> {
        encapsulation::fchmod(&self.fd.as_fd(), perm.0)
    }

    /// Changes the timestamps of the underlying file.
    #[inline]
    pub fn set_times(&self, times: FileTimes) -> Result<()> {
        encapsulation::futimens(
            &self.fd.as_fd(),
            &encapsulation::TimestampSpec::Set(times.0[0]),
            &encapsulation::TimestampSpec::Set(times.0[1]),
        )
    }

    /// Changes the modification time of the underlying file.
    ///
    /// This is an alias for `set_times(FileTimes::new().set_modified(time))`.
    #[inline]
    pub fn set_modified(&self, time: SystemTime) -> Result<()> {
        self.set_times(FileTimes::new().set_modified(time))
    }
}

impl Debug for File {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        fn get_path(fd: libc::c_int) -> Option<PathBuf> {
            let mut p = PathBuf::from("/proc/self/fd");
            p.push(&fd.to_string());
            read_link(&p).ok()
        }

        fn get_mode(fd: libc::c_int) -> Option<(bool, bool)> {
            let mode = encapsulation::fcntl_with_two_args(fd, libc::F_GETFL);
            if mode.is_err() {
                return None;
            }
            match mode.unwrap() & libc::O_ACCMODE {
                libc::O_RDONLY => Some((true, false)),
                libc::O_RDWR => Some((true, true)),
                libc::O_WRONLY => Some((false, true)),
                _ => None,
            }
        }

        let fd = self.fd.as_raw_fd();
        let mut b = f.debug_struct("File");
        b.field("fd", &fd);
        if let Some(path) = get_path(fd) {
            b.field("path", &path);
        }
        if let Some((read, write)) = get_mode(fd) {
            b.field("read", &read).field("write", &write);
        }
        b.finish()
    }
}

impl AsFd for File {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.fd.as_fd()
    }
}

impl AsRawFd for File {
    fn as_raw_fd(&self) -> RawFd {
        self.fd.as_raw_fd()
    }
}

impl FileExt for File {
    fn read_at(&self, buf: &mut [u8], offset: u64) -> Result<usize> {
        encapsulation::pread(self, buf, offset)
    }

    fn write_at(&self, buf: &[u8], offset: u64) -> Result<usize> {
        encapsulation::pwrite(self, buf, offset)
    }
}

impl From<File> for OwnedFd {
    fn from(value: File) -> Self {
        value.fd
    }
}

impl From<File> for Stdio {
    fn from(value: File) -> Self {
        Stdio::from(value.fd)
    }
}

impl From<OwnedFd> for File {
    fn from(value: OwnedFd) -> Self {
        Self { fd: value }
    }
}

impl FromRawFd for File {
    unsafe fn from_raw_fd(fd: RawFd) -> Self {
        Self {
            fd: OwnedFd::from_raw_fd(fd),
        }
    }
}

impl IntoRawFd for File {
    fn into_raw_fd(self) -> RawFd {
        self.fd.as_raw_fd()
    }
}

impl Read for File {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        encapsulation::read(self, buf)
    }
}

impl Read for &File {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        encapsulation::read(self, buf)
    }
}

impl Write for File {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        encapsulation::write(self, buf)
    }

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

impl Write for &File {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        encapsulation::write(self, buf)
    }

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

impl Seek for File {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        let (whence, offset) = match pos {
            SeekFrom::Current(offset) => (encapsulation::Whence::Cur, offset),
            SeekFrom::Start(offset) => (encapsulation::Whence::Set, offset as i64),
            SeekFrom::End(offset) => (encapsulation::Whence::End, offset),
        };

        encapsulation::lseek64(self, offset, whence)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::io::ErrorKind;

    #[test]
    fn open() {
        let _file = File::open("Cargo.toml").unwrap();
    }

    #[test]
    fn create() {
        let name = "file_create";
        let _file = File::create(name).unwrap();

        crate::functions::remove_file(name).unwrap();
    }

    #[test]
    fn create_new_already_exists() {
        let err = File::create_new("Cargo.toml").unwrap_err();
        assert_eq!(err.kind(), ErrorKind::AlreadyExists);
    }

    #[test]
    fn metadata() {
        let cargo_toml = File::open("Cargo.toml").unwrap();
        let metadata = cargo_toml.metadata().unwrap();

        assert!(metadata.is_file());
        assert!(!metadata.is_dir());
        assert!(!metadata.is_symlink());
    }

    #[test]
    fn try_clone() {
        let file = File::open("Cargo.toml").unwrap();
        let another_file = file.try_clone().unwrap();

        assert_ne!(file.as_raw_fd(), another_file.as_raw_fd());
    }

    #[test]
    fn sync_data() {
        let name = "file_sync_data";
        let mut file = File::create_new(name).unwrap();
        file.write(b"hello").unwrap();
        file.sync_data().unwrap();

        crate::functions::remove_file(name).unwrap();
    }

    #[test]
    fn sync_all() {
        let name = "file_sync_all";
        let mut file = File::create_new(name).unwrap();
        file.write(b"hello").unwrap();
        file.sync_data().unwrap();

        crate::functions::remove_file(name).unwrap();
    }

    #[test]
    fn set_len() {
        let name = "file_set_len";
        const SIZE: u64 = 5;

        let mut file = File::create_new(name).unwrap();
        file.write(b"hello world").unwrap();
        assert_eq!(file.metadata().unwrap().len(), 11);

        file.set_len(SIZE).unwrap();
        assert_eq!(file.metadata().unwrap().len(), SIZE);
        let mut buf = [0_u8; SIZE as usize];
        assert_eq!(file.read_at(&mut buf, 0).unwrap(), SIZE as usize);
        assert_eq!(buf, "hello".as_bytes());

        crate::functions::remove_file(name).unwrap();
    }

    #[test]
    fn set_permission() {
        let name = "file_set_permission";

        let file = File::create_new(name).unwrap();
        let mut perm = file.metadata().unwrap().permission();

        assert!(!perm.readonly());
        perm.set_readonly(true);

        file.set_permissions(perm).unwrap();

        perm = file.metadata().unwrap().permission();
        assert!(perm.readonly());

        crate::functions::remove_file(name).unwrap();
    }

    #[test]
    fn set_times() {
        let name = "file_set_times";

        let file = File::create_new(name).unwrap();
        let file_times = FileTimes::new();
        file.set_times(file_times).unwrap();

        let atime = file.metadata().unwrap().accessed().unwrap();
        let mtime = file.metadata().unwrap().modified().unwrap();
        assert_eq!(atime, SystemTime::default());
        assert_eq!(mtime, SystemTime::default());

        crate::functions::remove_file(name).unwrap();
    }
}
