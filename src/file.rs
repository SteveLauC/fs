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
