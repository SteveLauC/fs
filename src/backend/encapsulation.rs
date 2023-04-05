//! Rusty encapsulation for libc-like syscall.

use super::{
    libc_like_syscall,
    major_minor::{major, minor},
};
use crate::non_fs::SystemTime;
use bitflags::bitflags;
use std::{
    collections::VecDeque,
    ffi::{CStr, CString, OsStr, OsString},
    io::{Error, ErrorKind, Result},
    os::unix::{
        ffi::{OsStrExt, OsStringExt},
        io::{AsFd, AsRawFd, FromRawFd, OwnedFd},
    },
    path::{Path, PathBuf},
};

bitflags! {
    pub(crate) struct Flags: libc::c_int {
        const O_ACCMODE = libc::O_ACCMODE;
        /// Open the file in append-only mode.
        const O_APPEND = libc::O_APPEND;
        /// Generate a signal when input or output becomes possible.
        const O_ASYNC = libc::O_ASYNC;
        /// Closes the file descriptor once an `execve` call is made.
        ///
        /// Also sets the file offset to the beginning of the file.
        const O_CLOEXEC = libc::O_CLOEXEC;
        /// Create the file if it does not exist.
        const O_CREAT = libc::O_CREAT;
        /// Try to minimize cache effects of the I/O for this file.
        const O_DIRECT = libc::O_DIRECT;
        /// If the specified path isn't a directory, fail.
        const O_DIRECTORY = libc::O_DIRECTORY;
        /// Implicitly follow each `write()` with an `fdatasync()`.
        const O_DSYNC = libc::O_DSYNC;
        /// Error out if a file was not created.
        const O_EXCL = libc::O_EXCL;
        /// Same as `O_SYNC`.
        const O_FSYNC = libc::O_FSYNC;
        /// Allow files whose sizes can't be represented in an `off_t` to be opened.
        const O_LARGEFILE = libc::O_LARGEFILE;
        /// Do not update the file last access time during `read(2)`s.
        const O_NOATIME = libc::O_NOATIME;
        /// Don't attach the device as the process' controlling terminal.
        const O_NOCTTY = libc::O_NOCTTY;
        /// Same as `O_NONBLOCK`.
        const O_NDELAY = libc::O_NDELAY;
        /// `open()` will fail if the given path is a symbolic link.
        const O_NOFOLLOW = libc::O_NOFOLLOW;
        /// When possible, open the file in nonblocking mode.
        const O_NONBLOCK = libc::O_NONBLOCK;
        /// Obtain a file descriptor for low-level access.
        ///
        /// The file itself is not opened and other file operations will fail.
        const O_PATH = libc::O_PATH;
        /// Only allow reading.
        ///
        /// This should not be combined with `O_WRONLY` or `O_RDWR`.
        const O_RDONLY = libc::O_RDONLY;
        /// Allow both reading and writing.
        ///
        /// This should not be combined with `O_WRONLY` or `O_RDONLY`.
        const O_RDWR = libc::O_RDWR;
        /// Similar to `O_DSYNC` but applies to `read`s instead.
        const O_RSYNC = libc::O_RSYNC;
        /// Implicitly follow each `write()` with an `fsync()`.
        const O_SYNC = libc::O_SYNC;
        /// Create an unnamed temporary file.
        const O_TMPFILE = libc::O_TMPFILE;
        /// Truncate an existing regular file to 0 length if it allows writing.
        const O_TRUNC = libc::O_TRUNC;
        /// Only allow writing.
        ///
        /// This should not be combined with `O_RDONLY` or `O_RDWR`.
        const O_WRONLY = libc::O_WRONLY;
    }
}

bitflags! {
    /// "File permissions" flags.
    ///
    /// Does NOT encode `File Type`
    pub(crate) struct Mode: libc::mode_t {
        const S_IRWXU=libc::S_IRWXU;
        const S_IRUSR=libc::S_IRUSR;
        const S_IWUSR=libc::S_IWUSR;
        const S_IXUSR=libc::S_IXUSR;
        const S_IRWXG=libc::S_IRWXG;
        const S_IRGRP=libc::S_IRGRP;
        const S_IWGRP=libc::S_IWGRP;
        const S_IXGRP=libc::S_IXGRP;
        const S_IRWXO=libc::S_IRWXO;
        const S_IROTH=libc::S_IROTH;
        const S_IWOTH=libc::S_IWOTH;
        const S_IXOTH=libc::S_IXOTH;
        const S_ISUID =libc::S_ISUID;
        const S_ISGID =libc::S_ISGID;
        const S_ISVTX =libc::S_ISVTX;
    }
}

/// Opens a file
///
/// Note: `path` should not contain byte 0, or this function will panic.
pub(crate) fn open<P: AsRef<Path>>(path: P, flag: Flags, mode: Mode) -> Result<OwnedFd> {
    let path = CString::new(path.as_ref().as_os_str().as_bytes()).unwrap();
    let flag = flag.bits();
    let mode = mode.bits();

    match libc_like_syscall::open(path.as_ptr(), flag, mode) {
        Ok(raw_fd) => Ok(unsafe { OwnedFd::from_raw_fd(raw_fd) }),
        Err(errno) => Err(Error::from_raw_os_error(errno)),
    }
}

/// Creates a file.
///
/// Note: `path` should not contain byte 0, or this function will panic.
pub(crate) fn creat<P: AsRef<Path>>(path: P, mode: Mode) -> Result<OwnedFd> {
    let path = CString::new(path.as_ref().as_os_str().as_bytes()).unwrap();
    let mode = mode.bits();

    match libc_like_syscall::creat(path.as_ptr(), mode) {
        Ok(raw_fd) => Ok(unsafe { OwnedFd::from_raw_fd(raw_fd) }),
        Err(errno) => Err(Error::from_raw_os_error(errno)),
    }
}

/// Reads from a stream
pub(crate) fn read<Fd: AsFd>(fd: Fd, buf: &mut [u8]) -> Result<usize> {
    let raw_fd = fd.as_fd().as_raw_fd();

    libc_like_syscall::read(raw_fd, buf.as_mut_ptr() as *mut libc::c_void, buf.len())
        .map_err(Error::from_raw_os_error)
}

/// Writes to a stream
pub(crate) fn write<Fd: AsFd>(fd: Fd, buf: &[u8]) -> Result<usize> {
    let raw_fd = fd.as_fd().as_raw_fd();

    libc_like_syscall::write(raw_fd, buf.as_ptr() as *const libc::c_void, buf.len())
        .map_err(Error::from_raw_os_error)
}

/// Read from a file at the given offset
pub(crate) fn pread<Fd: AsFd>(fd: Fd, buf: &mut [u8], offset: u64) -> Result<usize> {
    let raw_fd = fd.as_fd().as_raw_fd();
    let offset = offset as libc::off_t;

    libc_like_syscall::pread(
        raw_fd,
        buf.as_mut_ptr() as *mut libc::c_void,
        buf.len(),
        offset,
    )
    .map_err(Error::from_raw_os_error)
}

/// Write to a file at the given offset
pub(crate) fn pwrite<Fd: AsFd>(fd: &Fd, buf: &[u8], offset: u64) -> Result<usize> {
    let raw_fd = fd.as_fd().as_raw_fd();
    let offset = offset as libc::off_t;

    libc_like_syscall::pwrite(
        raw_fd,
        buf.as_ptr() as *const libc::c_void,
        buf.len(),
        offset,
    )
    .map_err(Error::from_raw_os_error)
}

/// Makes a new name for a file
///
/// Note: `old_path` and `new_path` should not contain byte 0, or this function
/// will panic.
pub(crate) fn link<P: AsRef<Path>, Q: AsRef<Path>>(old_path: P, new_path: Q) -> Result<()> {
    let old_path = CString::new(old_path.as_ref().as_os_str().as_bytes()).unwrap();
    let new_path = CString::new(new_path.as_ref().as_os_str().as_bytes()).unwrap();

    libc_like_syscall::link(old_path.as_ptr(), new_path.as_ptr()).map_err(Error::from_raw_os_error)
}

/// Deletes a name or possibly a file it refers to
///
/// Note: `path_name` should not contain byte 0, or this function will panic.
pub(crate) fn unlink<P: AsRef<Path>>(path_name: P) -> Result<()> {
    let path_name = CString::new(path_name.as_ref().as_os_str().as_bytes()).unwrap();

    libc_like_syscall::unlink(path_name.as_ptr()).map_err(Error::from_raw_os_error)
}

/// Makes a new name for a file
///
/// Note: `target` and `link_path` should not contain byte 0, or this function
/// will panic.
pub(crate) fn symlink<P: AsRef<Path>, Q: AsRef<Path>>(target: P, link_path: Q) -> Result<()> {
    let target = CString::new(target.as_ref().as_os_str().as_bytes()).unwrap();
    let link_path = CString::new(link_path.as_ref().as_os_str().as_bytes()).unwrap();

    libc_like_syscall::symlink(target.as_ptr(), link_path.as_ptr())
        .map_err(Error::from_raw_os_error)
}

/// Creates a directory
///
/// Note: `path_name` should not contain byte 0, or this function will panic.
pub(crate) fn mkdir<P: AsRef<Path>>(path_name: P, mode: Mode) -> Result<()> {
    let path_name = CString::new(path_name.as_ref().as_os_str().as_bytes()).unwrap();

    libc_like_syscall::mkdir(path_name.as_ptr(), mode.bits()).map_err(Error::from_raw_os_error)
}

/// Deletes a directory
///
/// Note: `path_name` should not contain byte 0, or this function will panic.
pub(crate) fn rmdir<P: AsRef<Path>>(path_name: P) -> Result<()> {
    let path_name = CString::new(path_name.as_ref().as_os_str().as_bytes()).unwrap();

    libc_like_syscall::rmdir(path_name.as_ptr()).map_err(Error::from_raw_os_error)
}

/// Changes the name or location of a file
///
/// Note: `old_path` and `new_path` should not contain byte 0, or this function
/// will panic.
pub(crate) fn rename<P: AsRef<Path>, Q: AsRef<Path>>(old_path: P, new_path: Q) -> Result<()> {
    let old_path = CString::new(old_path.as_ref().as_os_str().as_bytes()).unwrap();
    let new_path = CString::new(new_path.as_ref().as_os_str().as_bytes()).unwrap();

    libc_like_syscall::rename(old_path.as_ptr(), new_path.as_ptr())
        .map_err(Error::from_raw_os_error)
}

pub(crate) struct Stat(libc_like_syscall::Stat);

impl Stat {
    /// Returns a tuple (major_dev_id, minor_dev_id).
    #[inline]
    pub(crate) fn dev(&self) -> (u32, u32) {
        let dev_number = self.0.st_dev;
        (major(dev_number), minor(dev_number))
    }

    /// Returns I-node number
    #[inline]
    pub(crate) fn ino(&self) -> libc::ino_t {
        self.0.st_ino
    }

    /// Returns the number of hard links.
    #[inline]
    pub(crate) fn nlink(&self) -> libc::nlink_t {
        self.0.st_nlink
    }

    /// Return a number encoding file type and permission.
    #[inline]
    pub(crate) fn mode(&self) -> u32 {
        self.0.st_mode
    }

    /// Returns a [`FileType`] representing the file type.
    #[inline]
    pub(crate) fn file_type(&self) -> FileType {
        FileType::from(self.0.st_mode)
    }

    /// Returns a [`Mode`] representing the file permission.
    #[inline]
    pub(crate) fn permission(&self) -> Mode {
        Mode::from_bits_truncate(self.0.st_mode)
    }

    /// Returns UID of the file owner.
    #[inline]
    pub(crate) fn uid(&self) -> libc::uid_t {
        self.0.st_uid
    }

    /// Returns GID of the file owner.
    #[inline]
    pub(crate) fn gid(&self) -> libc::gid_t {
        self.0.st_gid
    }

    /// Returns a tuple (major_rdev_id, minor_rdev_id).
    #[inline]
    pub(crate) fn rdev(&self) -> (u32, u32) {
        let rdev_number = self.0.st_rdev;
        (major(rdev_number), minor(rdev_number))
    }

    /// Returns file size (in bytes).
    #[inline]
    pub(crate) fn size(&self) -> libc::off_t {
        self.0.st_size
    }

    /// Returns the block size for the file system I/O.
    #[inline]
    pub(crate) fn blksize(&self) -> libc::blksize_t {
        self.0.st_blksize
    }

    /// Returns the number of 512B blocks allocated.
    #[inline]
    pub(crate) fn blocks(&self) -> libc::blkcnt64_t {
        self.0.st_blocks
    }

    /// Returns the time of last access.
    #[inline]
    pub(crate) fn atime(&self) -> (libc::time_t, i64) {
        (self.0.st_atime, self.0.st_atime_nsec)
    }

    /// Returns the time of last modification.
    #[inline]
    pub(crate) fn mtime(&self) -> (libc::time_t, i64) {
        (self.0.st_mtime, self.0.st_mtime_nsec)
    }

    /// Returns the time of last status (metadata) change.
    #[inline]
    pub(crate) fn ctime(&self) -> (libc::time_t, i64) {
        (self.0.st_ctime, self.0.st_ctime_nsec)
    }
}

impl From<libc_like_syscall::Stat> for Stat {
    fn from(value: libc_like_syscall::Stat) -> Self {
        Self(value)
    }
}

/// Get file status
///
/// Note: `path_name` should not contain byte 0, or this function will panic.
pub(crate) fn stat<P: AsRef<Path>>(path_name: P) -> Result<Stat> {
    let path_name = CString::new(path_name.as_ref().as_os_str().as_bytes()).unwrap();
    let mut stat_buf = libc_like_syscall::Stat::default();

    match libc_like_syscall::stat(
        path_name.as_ptr(),
        &mut stat_buf as *mut libc_like_syscall::Stat,
    ) {
        Ok(()) => Ok(Stat::from(stat_buf)),
        Err(errno) => Err(Error::from_raw_os_error(errno)),
    }
}

/// Get file status
///
/// Note: `path_name` should not contain byte 0, or this function will panic.
pub(crate) fn lstat<P: AsRef<Path>>(path_name: P) -> Result<Stat> {
    let path_name = CString::new(path_name.as_ref().as_os_str().as_bytes()).unwrap();
    let mut stat_buf = libc_like_syscall::Stat::default();

    match libc_like_syscall::lstat(
        path_name.as_ptr(),
        &mut stat_buf as *mut libc_like_syscall::Stat,
    ) {
        Ok(()) => Ok(Stat::from(stat_buf)),
        Err(errno) => Err(Error::from_raw_os_error(errno)),
    }
}

/// Get file status
pub(crate) fn fstat<Fd: AsFd>(fd: Fd) -> Result<Stat> {
    let mut stat_buf = libc_like_syscall::Stat::default();

    match libc_like_syscall::fstat(
        fd.as_fd().as_raw_fd(),
        &mut stat_buf as *mut libc_like_syscall::Stat,
    ) {
        Ok(()) => Ok(Stat::from(stat_buf)),
        Err(errno) => Err(Error::from_raw_os_error(errno)),
    }
}

#[derive(Clone)]
pub(crate) struct Statx(libc_like_syscall::Statx);

impl From<libc_like_syscall::Statx> for Statx {
    fn from(value: libc_like_syscall::Statx) -> Self {
        Self(value)
    }
}

impl Statx {
    /// Returns the block size for the file system I/O.
    #[inline]
    pub(crate) fn blksize(&self) -> u32 {
        self.0.stx_blksize
    }

    /// Returns extra file attribute indicators.
    #[inline]
    pub(crate) fn attributes(&self) -> u64 {
        self.0.stx_attributes
    }

    /// Returns the number of hard links.
    #[inline]
    pub(crate) fn nlink(&self) -> u32 {
        self.0.stx_nlink
    }

    /// Returns UID of the file owner.
    #[inline]
    pub(crate) fn uid(&self) -> u32 {
        self.0.stx_uid
    }

    /// Returns GID of the file owner.
    #[inline]
    pub(crate) fn gid(&self) -> u32 {
        self.0.stx_gid
    }

    /// Returns a number encoding file type and permission.
    #[inline]
    pub(crate) fn mode(&self) -> u32 {
        self.0.stx_mode as u32
    }

    /// Returns a [`Mode`] representing the file permission.
    #[inline]
    pub(crate) fn permission(&self) -> Mode {
        Mode::from_bits_truncate(self.0.stx_mode as libc::mode_t)
    }

    /// Returns a [`FileType`] representing the file type.
    #[inline]
    pub(crate) fn file_type(&self) -> FileType {
        FileType::from(self.0.stx_mode as libc::mode_t)
    }

    /// Returns I-node number
    #[inline]
    pub(crate) fn ino(&self) -> u64 {
        self.0.stx_ino
    }

    /// Returns file size (in bytes).
    #[inline]
    pub(crate) fn size(&self) -> u64 {
        self.0.stx_size
    }

    /// Returns the number of blocks allocated for this file.
    #[inline]
    pub(crate) fn blocks(&self) -> u64 {
        self.0.stx_blocks
    }

    /// Returns the time of last access.
    #[inline]
    pub(crate) fn atime(&self) -> (i64, u32) {
        let atime = self.0.stx_atime;
        (atime.tv_sec, atime.tv_nsec)
    }

    /// Returns the time of creation.
    #[inline]
    pub(crate) fn btime(&self) -> (i64, u32) {
        let btime = self.0.stx_btime;
        (btime.tv_sec, btime.tv_nsec)
    }

    /// Returns the time of last status (metadata) change.
    #[inline]
    pub(crate) fn ctime(&self) -> (i64, u32) {
        let ctime = self.0.stx_ctime;
        (ctime.tv_sec, ctime.tv_nsec)
    }

    /// Returns the time of last modification.
    #[inline]
    pub(crate) fn mtime(&self) -> (i64, u32) {
        let mtime = self.0.stx_mtime;
        (mtime.tv_sec, mtime.tv_nsec)
    }

    /// Returns a tuple (major_rdev_id, minor_rdev_id).
    #[inline]
    pub(crate) fn rdev(&self) -> (u32, u32) {
        (self.0.stx_rdev_major, self.0.stx_rdev_minor)
    }

    /// Returns a tuple (major_dev_id, minor_dev_id).
    #[inline]
    pub(crate) fn dev(&self) -> (u32, u32) {
        (self.0.stx_dev_major, self.0.stx_dev_minor)
    }

    /// Returns mount id.
    #[inline]
    pub(crate) fn mnt_id(&self) -> u64 {
        self.0.stx_mnt_id
    }
}

pub(crate) fn statx<P: AsRef<Path>>(path: P) -> Result<Statx> {
    let pathname = CString::new(path.as_ref().as_os_str().as_bytes()).unwrap();
    let mut statx_buf = libc_like_syscall::Statx::default();

    match libc_like_syscall::statx(
        libc::AT_FDCWD,
        pathname.as_ptr(),
        0,
        libc::STATX_ALL,
        &mut statx_buf as *mut libc_like_syscall::Statx,
    ) {
        Ok(()) => Ok(Statx::from(statx_buf)),
        Err(errno) => Err(Error::from_raw_os_error(errno)),
    }
}

pub(crate) fn lstatx<P: AsRef<Path>>(path: P) -> Result<Statx> {
    let pathname = CString::new(path.as_ref().as_os_str().as_bytes()).unwrap();
    let mut statx_buf = libc_like_syscall::Statx::default();

    match libc_like_syscall::statx(
        libc::AT_FDCWD,
        pathname.as_ptr(),
        libc::AT_SYMLINK_NOFOLLOW,
        libc::STATX_ALL,
        &mut statx_buf as *mut libc_like_syscall::Statx,
    ) {
        Ok(()) => Ok(Statx::from(statx_buf)),
        Err(errno) => Err(Error::from_raw_os_error(errno)),
    }
}

pub(crate) fn fstatx<Fd: AsFd>(fd: Fd) -> Result<Statx> {
    let mut statx_buf = libc_like_syscall::Statx::default();

    match libc_like_syscall::statx(
        fd.as_fd().as_raw_fd(),
        "\0".as_ptr().cast(),
        libc::AT_EMPTY_PATH,
        libc::STATX_ALL,
        &mut statx_buf as *mut libc_like_syscall::Statx,
    ) {
        Ok(()) => Ok(Statx::from(statx_buf)),
        Err(errno) => Err(Error::from_raw_os_error(errno)),
    }
}

/// Gets directory entries
pub(crate) fn getdents64<Fd: AsFd>(fd: Fd, dirp: &mut [u8]) -> Result<usize> {
    libc_like_syscall::getdents64(
        fd.as_fd().as_raw_fd(),
        dirp.as_mut_ptr() as *mut libc::c_void,
        dirp.len(),
    )
    .map_err(Error::from_raw_os_error)
}

#[repr(C)]
struct LinuxDirent64 {
    /// Inode number
    d_ino: libc::ino_t,
    /// Offset to the next `LinuxDirent64`
    d_off: libc::off64_t,
    /// Size of this `LinuxDirent64`
    d_reclen: libc::c_ushort,
    /// File type
    d_type: libc::c_uchar,
    // The last field of the `linux_dirent64` in C is a
    // [`flexible array member`](https://en.wikipedia.org/wiki/Flexible_array_member)
    // We don't have this in Rust.
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) enum FileType {
    RegularFile,
    Directory,
    Socket,
    Fifo,
    CharDev,
    BlkDev,
    Symlink,
}

impl From<libc::c_uchar> for FileType {
    fn from(file_type_char: libc::c_uchar) -> Self {
        if file_type_char == libc::DT_REG {
            FileType::RegularFile
        } else if file_type_char == libc::DT_DIR {
            FileType::Directory
        } else if file_type_char == libc::DT_SOCK {
            FileType::Socket
        } else if file_type_char == libc::DT_FIFO {
            FileType::Fifo
        } else if file_type_char == libc::DT_CHR {
            FileType::CharDev
        } else if file_type_char == libc::DT_BLK {
            FileType::BlkDev
        } else {
            FileType::Symlink
        }
    }
}

impl From<libc::mode_t> for FileType {
    fn from(value: libc::mode_t) -> Self {
        match value & libc::S_IFMT {
            libc::S_IFDIR => FileType::Directory,
            libc::S_IFCHR => FileType::CharDev,
            libc::S_IFBLK => FileType::BlkDev,
            libc::S_IFREG => FileType::RegularFile,
            libc::S_IFIFO => FileType::Fifo,
            libc::S_IFLNK => FileType::Symlink,
            libc::S_IFSOCK => FileType::Socket,
            _ => unreachable!("Linux supports only 7 file types, this should not happen"),
        }
    }
}

/// Offset of field `d_name` in `struct linux_dirent64`
///
/// I am not sure about if this offset value is correct, it just works.
const OFFSET_D_NAME: usize = 19;
const BUF_SIZE: usize = 1024;

#[derive(Debug)]
pub(crate) struct Dir {
    fd: OwnedFd,
    root: PathBuf,
    buf: [u8; BUF_SIZE],
    entries: VecDeque<Dirent>,
}

#[derive(Debug)]
pub(crate) struct Dirent {
    pub(crate) ino: u64,
    pub(crate) file_type: FileType,
    pub(crate) name: OsString,
    pub(crate) path: PathBuf,
}

impl Dirent {
    pub(crate) fn new(
        ino: libc::ino64_t,
        file_type: libc::c_uchar,
        name_start_ptr: *const libc::c_char,
        root: &Path,
    ) -> Self {
        let name =
            OsStr::from_bytes(unsafe { CStr::from_ptr(name_start_ptr) }.to_bytes()).to_owned();
        let path = root.join(name.as_os_str());

        Self {
            ino,
            file_type: FileType::from(file_type),
            name,
            path,
        }
    }
}

impl Dir {
    #[inline]
    pub(crate) fn root(&self) -> PathBuf {
        self.root.clone()
    }

    pub(crate) fn opendir<P: AsRef<Path>>(name: P) -> Result<Dir> {
        let fd = open(
            name.as_ref(),
            Flags::O_RDONLY | Flags::O_DIRECTORY,
            Mode::empty(),
        )?;
        Ok(Self {
            fd,
            root: name.as_ref().to_owned(),
            buf: [0; BUF_SIZE],
            entries: VecDeque::with_capacity(5),
        })
    }

    pub(crate) fn readdir(&mut self) -> Option<Result<Dirent>> {
        if self.entries.is_empty() {
            let num_read = match getdents64(&self.fd.as_fd(), &mut self.buf) {
                Err(e) => return Some(Err(e)),
                Ok(n) => n,
            };

            if num_read == 0 {
                return None;
            }

            let mut cursor = 0_usize;
            while cursor < num_read {
                unsafe {
                    let ptr_to_d_entry = self.buf.as_ptr().add(cursor) as *const LinuxDirent64;

                    let entry = Dirent::new(
                        (*ptr_to_d_entry).d_ino,
                        (*ptr_to_d_entry).d_type,
                        (ptr_to_d_entry as *const libc::c_char).add(OFFSET_D_NAME),
                        self.root.as_path(),
                    );

                    // skip "." and ".."
                    if entry.name != "." && entry.name != ".." {
                        self.entries.push_back(entry);
                    }

                    cursor += (*ptr_to_d_entry).d_reclen as usize;
                }
            }
        }

        if let Some(entry) = self.entries.pop_front() {
            Some(Ok(entry))
        } else {
            None
        }
    }
}

/// Change Root Directory.
///
/// Note: `path_name` should not contain byte 0, or this function will panic.
pub(crate) fn chroot<P: AsRef<Path>>(path: P) -> Result<()> {
    let path = CString::new(path.as_ref().as_os_str().as_bytes()).unwrap();
    libc_like_syscall::chroot(path.as_ptr()).map_err(Error::from_raw_os_error)
}

/// `whence` argument of `lseek64(2)`
#[repr(i32)]
pub(crate) enum Whence {
    Set = libc::SEEK_SET,
    Cur = libc::SEEK_CUR,
    End = libc::SEEK_END,
}

/// reposition read/write file offset
pub(crate) fn lseek64<Fd: AsFd>(fd: Fd, offset: i64, whence: Whence) -> Result<u64> {
    let raw_fd = fd.as_fd().as_raw_fd();
    let whence = whence as libc::c_int;

    libc_like_syscall::lseek64(raw_fd, offset, whence).map_err(Error::from_raw_os_error)
}

/// Read value of a symbolic link
pub(crate) fn readlink<P: AsRef<Path>>(pathname: P) -> Result<PathBuf> {
    let pathname = CString::new(pathname.as_ref().as_os_str().as_bytes()).unwrap();
    let mut buf: Vec<u8> = Vec::with_capacity(libc::PATH_MAX as usize);

    let bytes_read = libc_like_syscall::readlink(
        pathname.as_ptr(),
        buf.as_mut_ptr().cast(),
        libc::PATH_MAX as _,
    )
    .map_err(Error::from_raw_os_error)?;

    unsafe {
        buf.set_len(bytes_read as usize);
    }

    Ok(PathBuf::from(OsString::from_vec(buf)))
}

/// A simplified version of `fcntl(2)`, supports only two arguments.
//
// Currently, this will be only used in the `Debug` implementation for `File`,
// so this simple wrapper would suffice.
pub(crate) use libc_like_syscall::fcntl_with_two_args;

/// Transfers  ("flushes") all modified in-core data of (i.e., modified buffer
/// cache pages for) the file referred to by the file descriptor fd to the
/// disk device
pub(crate) fn fsync<Fd: AsFd>(fd: Fd) -> Result<()> {
    libc_like_syscall::fsync(fd.as_fd().as_raw_fd()).map_err(Error::from_raw_os_error)
}
/// `fdatasync()` is similar to [`fsync()`], but does not flush modified metadata
/// unless that metadata  is needed in order to allow a subsequent data retrieval
/// to be correctly handled
pub(crate) fn fdatasync<Fd: AsFd>(fd: Fd) -> Result<()> {
    libc_like_syscall::fdatasync(fd.as_fd().as_raw_fd()).map_err(Error::from_raw_os_error)
}

/// Truncate a file to a specified length
///
/// If the file previously was larger than this size, the extra data is lost.
/// If the file previously was shorter, it is extended, and the extended part
/// reads as null bytes ('\0').
pub(crate) fn ftruncate<Fd: AsFd>(fd: Fd, length: u64) -> Result<()> {
    let length = length
        .try_into()
        .map_err(|e| Error::new(ErrorKind::InvalidInput, e))?;

    libc_like_syscall::ftruncate(fd.as_fd().as_raw_fd(), length).map_err(Error::from_raw_os_error)
}

/// Changes permissions of a file
pub(crate) fn chmod<P: AsRef<Path>>(pathname: P, mode: Mode) -> Result<()> {
    let pathname = CString::new(pathname.as_ref().as_os_str().as_bytes()).unwrap();
    let mode = mode.bits();
    libc_like_syscall::chmod(pathname.as_ptr(), mode).map_err(Error::from_raw_os_error)
}

/// Changes permissions of a file
pub(crate) fn fchmod<Fd: AsFd>(fd: Fd, mode: Mode) -> Result<()> {
    let mode = mode.bits();
    libc_like_syscall::fchmod(fd.as_fd().as_raw_fd(), mode).map_err(Error::from_raw_os_error)
}

/// Time operation used in [`futimens()`].
pub(crate) enum TimestampSpec {
    Omit,
    SetToNow,
    Set(SystemTime),
}

impl From<&TimestampSpec> for libc_like_syscall::Timespec {
    fn from(value: &TimestampSpec) -> Self {
        let mut default = libc_like_syscall::Timespec {
            tv_sec: 0,
            tv_nsec: 0,
        };

        match value {
            TimestampSpec::Omit => default.tv_nsec = libc::UTIME_OMIT,
            TimestampSpec::SetToNow => default.tv_nsec = libc::UTIME_NOW,
            TimestampSpec::Set(time) => {
                default.tv_sec = time.sec;
                default.tv_nsec = time.nsec;
            }
        }
        default
    }
}

/// Changes file timestamps with nanosecond precision
//
// This syscall is implemented on the top of `utimensat(2)`, for more information, see
// https://man7.org/linux/man-pages/man2/utimensat.2.html
pub(crate) fn futimens<Fd: AsFd>(
    fd: Fd,
    atime: &TimestampSpec,
    mtime: &TimestampSpec,
) -> Result<()> {
    // atime and mtime
    let times = [atime.into(), mtime.into()];

    libc_like_syscall::utimensat(
        fd.as_fd().as_raw_fd(),
        std::ptr::null(),
        &times as *const libc_like_syscall::Timespec,
        0,
    )
    .map_err(Error::from_raw_os_error)
}

/// Change ownership of a file
pub(crate) fn chown<P: AsRef<Path>>(
    pathname: P,
    owner: Option<u32>,
    group: Option<u32>,
) -> Result<()> {
    let pathname = CString::new(pathname.as_ref().as_os_str().as_bytes()).unwrap();
    // libc::uid_t and libc::gid_t are unsigned number, -1 = MAX
    let owner = owner.unwrap_or(u32::MAX);
    let group = group.unwrap_or(u32::MAX);

    libc_like_syscall::chown(pathname.as_ptr(), owner, group).map_err(Error::from_raw_os_error)
}

/// Change ownership of the file that are specified by the open file descriptor `fd`
pub(crate) fn fchown<Fd: AsFd>(fd: Fd, owner: Option<u32>, group: Option<u32>) -> Result<()> {
    let fd = fd.as_fd().as_raw_fd();
    let owner = owner.unwrap_or(u32::MAX);
    let group = group.unwrap_or(u32::MAX);
    libc_like_syscall::fchown(fd, owner, group).map_err(Error::from_raw_os_error)
}

/// Change ownership of a file
///
/// If `pathname` refers to a symlink, then the ownership of the link **itself**
/// will be changed.
pub(crate) fn lchown<P: AsRef<Path>>(
    pathname: P,
    owner: Option<u32>,
    group: Option<u32>,
) -> Result<()> {
    let pathname = CString::new(pathname.as_ref().as_os_str().as_bytes()).unwrap();
    // libc::uid_t and libc::gid_t are unsigned number, -1 = MAX
    let owner = owner.unwrap_or(u32::MAX);
    let group = group.unwrap_or(u32::MAX);

    libc_like_syscall::lchown(pathname.as_ptr(), owner, group).map_err(Error::from_raw_os_error)
}

#[cfg(test)]
mod test {
    use super::*;
    use std::os::fd::BorrowedFd;

    #[test]
    fn test_open() {
        open("/tmp", Flags::O_RDONLY, Mode::empty()).unwrap();
    }

    #[test]
    fn test_creat_unlink() {
        let file = "/tmp/test_creat_unlink";
        creat(file, Mode::from_bits(0o644).unwrap()).unwrap();

        unlink(file).unwrap();
    }

    #[test]
    fn test_read_write() {
        let file = "/tmp/test_read_write";
        let fd_with_read_permission = creat(file, Mode::from_bits(0o644).unwrap()).unwrap();
        let fd_with_write_permission = open(file, Flags::O_WRONLY, Mode::empty()).unwrap();
        assert_eq!(
            write(&fd_with_write_permission.as_fd(), b"hello").unwrap(),
            5
        );

        let mut buffer = [0_u8; 5];
        assert_eq!(
            read(&fd_with_read_permission.as_fd(), &mut buffer).unwrap(),
            5
        );
        assert_eq!(&buffer, b"hello");

        unlink(file).unwrap();
    }

    #[test]
    fn test_pread() {
        let file = "/tmp/test_pread";
        creat(file, Mode::from_bits(0o644).unwrap()).unwrap();

        let fd = open(file, Flags::O_RDWR, Mode::empty()).unwrap();
        write(&fd.as_fd(), b"hello world").unwrap();

        let mut buf = [0_u8; 5];
        assert_eq!(pread(&fd.as_fd(), &mut buf, 6).unwrap(), 5);

        assert_eq!(&buf, b"world");

        unlink(file).unwrap();
    }

    #[test]
    fn test_pwrite() {
        let file = "/tmp/test_pwrite";
        creat(file, Mode::from_bits(0o644).unwrap()).unwrap();

        let fd = open(file, Flags::O_RDWR, Mode::empty()).unwrap();
        write(&fd.as_fd(), b"hello world").unwrap();

        assert_eq!(pwrite(&fd.as_fd(), b"steve", 6).unwrap(), 5);

        let mut buf = [0_u8; 11];
        assert_eq!(pread(&fd.as_fd(), &mut buf, 0).unwrap(), 11);

        assert_eq!(&buf, b"hello steve");
        unlink(file).unwrap();
    }

    #[test]
    fn test_link() {
        let file = "/tmp/test_link";
        let ln = "/tmp/test_link_ln";

        creat(file, Mode::from_bits(0o644).unwrap()).unwrap();
        link(file, ln).unwrap();
        unlink(file).unwrap();
        unlink(ln).unwrap();
    }

    #[test]
    fn test_mkdir() {
        let dir = "/tmp/test_mkdir";
        mkdir(dir, Mode::from_bits(0o777).unwrap()).unwrap();

        rmdir(dir).unwrap();
    }

    #[test]
    fn test_rename() {
        let old_path = "/tmp/test_rename_old_path";
        let new_path = "/tmp/test_rename_new_path";
        creat(old_path, Mode::from_bits(0o644).unwrap()).unwrap();

        rename(old_path, new_path).unwrap();

        unlink(new_path).unwrap();
    }

    #[test]
    fn test_symlink() {
        let file = "/tmp/test_symlink";
        let soft_link = "/tmp/test_symlink_soft_link";
        creat(file, Mode::from_bits(0o644).unwrap()).unwrap();

        symlink(file, soft_link).unwrap();

        unlink(soft_link).unwrap();
        unlink(file).unwrap();
    }

    #[test]
    fn test_stat() {
        let file = "/tmp/test_stat";
        creat(file, Mode::from_bits(0o644).unwrap()).unwrap();

        let stat_buf = stat(file).unwrap();

        assert_eq!(stat_buf.file_type(), FileType::RegularFile);
        unlink(file).unwrap();
    }

    #[test]
    fn test_fstat() {
        let file = "/tmp/test_fstat";
        let fd = creat(file, Mode::from_bits(0o644).unwrap()).unwrap();

        let stat_buf = fstat(&fd.as_fd()).unwrap();

        assert_eq!(stat_buf.file_type(), FileType::RegularFile);
        unlink(file).unwrap();
    }

    #[test]
    fn test_lstat() {
        let file = "/tmp/test_lstat";
        let soft_link = "/tmp/test_lstat_link";
        creat(file, Mode::from_bits(0o644).unwrap()).unwrap();
        symlink(file, soft_link).unwrap();

        let stat_buf = lstat(soft_link).unwrap();

        assert_eq!(stat_buf.file_type(), FileType::Symlink);

        unlink(file).unwrap();
        unlink(soft_link).unwrap();
    }

    #[test]
    fn test_statx() {
        let file = "/tmp/test_statx";
        creat(file, Mode::from_bits(0o644).unwrap()).unwrap();

        let statx_buf = statx(file).unwrap();

        assert_eq!(statx_buf.file_type(), FileType::RegularFile);
        unlink(file).unwrap();
    }
    #[test]
    fn test_lstatx() {
        let file = "/tmp/test_lstatx";
        let soft_link = "/tmp/test_lstatx_link";
        creat(file, Mode::from_bits(0o644).unwrap()).unwrap();
        symlink(file, soft_link).unwrap();

        let statx_buf = lstatx(soft_link).unwrap();

        assert_eq!(statx_buf.file_type(), FileType::Symlink);

        unlink(file).unwrap();
        unlink(soft_link).unwrap();
    }
    #[test]
    fn test_fstatx() {
        let file = "/tmp/test_fstatx";
        let fd = creat(file, Mode::from_bits(0o644).unwrap()).unwrap();

        let statx_buf = fstatx(&fd.as_fd()).unwrap();

        assert_eq!(statx_buf.file_type(), FileType::RegularFile);
        unlink(file).unwrap();
    }

    #[test]
    fn test_getdents64() {
        let tmp_dir = "/tmp";
        let tmp_dir_fd = open(tmp_dir, Flags::O_RDONLY, Mode::empty()).unwrap();
        let mut buf = [0_u8; 100];
        getdents64(&tmp_dir_fd.as_fd(), &mut buf).unwrap();
    }

    #[test]
    fn test_opendir_readdir() {
        let output = std::process::Command::new("ls")
            .args(["-al", "/"])
            .output()
            .unwrap();
        assert!(output.stderr.is_empty());
        let num_of_file = output.stdout.iter().filter(|&&b| b == b'\n').count() - 1;

        let mut dir = Dir::opendir("/").unwrap();
        let mut n_files = 0;
        while let Some(Ok(_)) = dir.readdir() {
            n_files += 1;
        }

        // add "." and ".."
        n_files += 2;

        assert_eq!(num_of_file, n_files);
    }

    #[test]
    fn test_chroot() {
        let error = chroot(".").unwrap_err();
        assert_eq!(error.raw_os_error().unwrap(), libc::EPERM);
    }

    #[test]
    fn test_lseek64() {
        let file = "/tmp/test_lseek64";
        creat(file, Mode::from_bits(0o644).unwrap()).unwrap();
        let fd = open(file, Flags::O_RDWR, Mode::empty()).unwrap();
        write(&fd.as_fd(), b"hello").unwrap();

        assert_eq!(lseek64(&fd.as_fd(), 0, Whence::Set).unwrap(), 0);

        unlink(file).unwrap();
    }

    #[test]
    fn test_readlink() {
        let file = "/tmp/test_readlink";
        let link = "/tmp/test_readlink_link";
        creat(file, Mode::from_bits(0o644).unwrap()).unwrap();
        symlink(file, link).unwrap();
        let link_contents = readlink(link).unwrap();

        assert_eq!(Path::new(file), link_contents.as_path());

        unlink(file).unwrap();
        unlink(link).unwrap();
    }

    #[test]
    fn test_fsync() {
        let file = "/tmp/test_fsync";
        let fd = creat(file, Mode::from_bits(0o644).unwrap()).unwrap();
        fsync(&fd.as_fd()).unwrap();
        unlink(file).unwrap();
    }

    #[test]
    fn test_fdatasync() {
        let file = "/tmp/test_fdatasync";
        let fd = creat(file, Mode::from_bits(0o644).unwrap()).unwrap();
        fdatasync(&fd.as_fd()).unwrap();
        unlink(file).unwrap();
    }

    #[test]
    fn test_ftruncate() {
        let file = "/tmp/ftruncate";
        let fd = open(
            file,
            Flags::O_RDWR | Flags::O_CREAT,
            Mode::from_bits(0o644).unwrap(),
        )
        .unwrap();

        assert_eq!(5, write(&fd.as_fd(), b"hello").unwrap());

        ftruncate(&fd.as_fd(), 3).unwrap();

        let stat = fstat(&fd.as_fd()).unwrap();
        assert_eq!(stat.size(), 3);

        unlink(file).unwrap();
    }

    #[test]
    fn test_ftruncate_with_too_large_length() {
        let file = "/tmp/ftruncate_with_too_large_length";
        let fd = open(
            file,
            Flags::O_RDWR | Flags::O_CREAT,
            Mode::from_bits(0o644).unwrap(),
        )
        .unwrap();

        assert_eq!(
            ErrorKind::InvalidInput,
            ftruncate(&fd.as_fd(), u64::MAX).unwrap_err().kind()
        );

        unlink(file).unwrap();
    }

    #[test]
    fn test_chmod() {
        let file = "/tmp/test_chmod_encap";
        open(
            file,
            Flags::O_CREAT | Flags::O_RDWR,
            Mode::from_bits(0o644).unwrap(),
        )
        .unwrap();
        let target_mode = unsafe { Mode::from_bits_unchecked(0o000) };

        chmod(file, target_mode).unwrap();

        let statx = statx(file).unwrap();

        assert_eq!(statx.permission(), target_mode);
        unlink(file).unwrap();
    }

    #[test]
    fn test_fchmod() {
        let file = "/tmp/test_fchmod_encap";
        let fd = open(
            file,
            Flags::O_CREAT | Flags::O_RDWR,
            Mode::from_bits(0o644).unwrap(),
        )
        .unwrap();

        let target_mode = unsafe { Mode::from_bits_unchecked(0o000) };

        fchmod(&fd.as_fd(), target_mode).unwrap();

        let statx = fstatx(&fd.as_fd()).unwrap();

        assert_eq!(statx.permission(), target_mode);
        unlink(file).unwrap();
    }

    #[test]
    fn test_futimens_omit() {
        let file = "/tmp/test_futimens_omit_encap";
        let fd = open(
            file,
            Flags::O_CREAT | Flags::O_RDWR,
            Mode::from_bits(0o644).unwrap(),
        )
        .unwrap();

        futimens(&fd.as_fd(), &TimestampSpec::Omit, &TimestampSpec::Omit).unwrap();
        unlink(file).unwrap();
    }

    #[test]
    fn test_futimens_set_now() {
        let file = "/tmp/test_futimens_set_now_encap";
        let fd = open(
            file,
            Flags::O_CREAT | Flags::O_RDWR,
            Mode::from_bits(0o644).unwrap(),
        )
        .unwrap();

        futimens(
            &fd.as_fd(),
            &TimestampSpec::SetToNow,
            &TimestampSpec::SetToNow,
        )
        .unwrap();
        unlink(file).unwrap();
    }

    #[test]
    fn test_futimens_set_to_a_specific_value() {
        let file = "/tmp/test_futimens_set_to_a_specific_value_encap";
        let fd = open(
            file,
            Flags::O_CREAT | Flags::O_RDWR,
            Mode::from_bits(0o644).unwrap(),
        )
        .unwrap();

        let atime = SystemTime::new(0, 1);
        let mtime = SystemTime::new(1, 0);
        futimens(
            &fd.as_fd(),
            &TimestampSpec::Set(atime),
            &TimestampSpec::Set(mtime),
        )
        .unwrap();

        let statx = fstatx(&fd.as_fd()).unwrap();

        assert_eq!(statx.atime(), (0, 1));
        assert_eq!(statx.mtime(), (1, 0));

        unlink(file).unwrap();
    }

    #[test]
    fn test_chown() {
        let file = "/tmp/test_chown_encap";
        open(
            file,
            Flags::O_CREAT | Flags::O_RDWR,
            Mode::from_bits(0o644).unwrap(),
        )
        .unwrap();
        let statx = statx(file).unwrap();
        let uid = Some(statx.uid());
        let gid = Some(statx.gid());

        chown(file, uid, gid).unwrap();
        chown(file, None, None).unwrap();
        chown(file, uid, None).unwrap();
        chown(file, None, gid).unwrap();
        unlink(file).unwrap();
        chown(file, uid, gid).unwrap_err();
    }

    #[test]
    fn test_fchown() {
        let file = "/tmp/test_fchown_encap";
        let fd = open(
            file,
            Flags::O_CREAT | Flags::O_RDWR,
            Mode::from_bits(0o644).unwrap(),
        )
        .unwrap();

        let statx = fstatx(&fd).unwrap();
        let uid = Some(statx.uid());
        let gid = Some(statx.gid());

        fchown(&fd, uid, gid).unwrap();
        fchown(&fd, None, None).unwrap();
        fchown(&fd, uid, None).unwrap();
        fchown(&fd, None, gid).unwrap();
        unlink(file).unwrap();
        let fd_that_is_not_open = unsafe { BorrowedFd::borrow_raw(9999) };
        fchown(fd_that_is_not_open, uid, gid).unwrap_err();
    }

    #[test]
    fn test_lchown() {
        let file = "/tmp/test_lchown_encap";
        let link = "/tmp/test_lchown_link_encap";
        open(
            file,
            Flags::O_CREAT | Flags::O_RDWR,
            Mode::from_bits(0o644).unwrap(),
        )
        .unwrap();
        symlink(file, link).unwrap();

        let statx = lstatx(link).unwrap();
        let uid = Some(statx.uid());
        let gid = Some(statx.gid());

        lchown(link, uid, gid).unwrap();
        lchown(link, None, None).unwrap();
        lchown(link, uid, None).unwrap();
        lchown(link, None, gid).unwrap();
        unlink(file).unwrap();
        unlink(link).unwrap();
        lchown(link, uid, gid).unwrap_err();
    }
}
