//! Rusty encapsulation for libc-like syscall.

use super::libc_like_syscall;
use bitflags::bitflags;
use std::{
    collections::VecDeque,
    ffi::{CStr, CString, OsStr, OsString},
    io::{Error, Result},
    os::unix::{
        ffi::OsStrExt,
        io::{AsFd, AsRawFd, FromRawFd, OwnedFd},
    },
    path::{Path, PathBuf},
};

bitflags! {
    pub(crate) struct Flags: libc::c_int {
        const O_ACCMODE = libc::O_ACCMODE;
        /// Open the file in append-only mode.
        const    O_APPEND = libc::O_APPEND;
        /// Generate a signal when input or output becomes possible.
        const  O_ASYNC = libc::O_ASYNC;
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
    /// "File mode / permissions" flags.
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
pub(crate) fn open<P: AsRef<Path>>(
    path: P,
    flag: Flags,
    mode: Mode,
) -> Result<OwnedFd> {
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

    libc_like_syscall::read(
        raw_fd,
        buf.as_mut_ptr() as *mut libc::c_void,
        buf.len(),
    )
    .map_err(Error::from_raw_os_error)
}

/// Writes to a stream
pub(crate) fn write<Fd: AsFd>(fd: Fd, buf: &[u8]) -> Result<usize> {
    let raw_fd = fd.as_fd().as_raw_fd();

    libc_like_syscall::write(
        raw_fd,
        buf.as_ptr() as *const libc::c_void,
        buf.len(),
    )
    .map_err(Error::from_raw_os_error)
}

/// Read from a file at the given offset
pub(crate) fn pread<Fd: AsFd>(
    fd: Fd,
    buf: &mut [u8],
    offset: u64,
) -> Result<usize> {
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
pub(crate) fn pwrite<Fd: AsFd>(
    fd: Fd,
    buf: &[u8],
    offset: u64,
) -> Result<usize> {
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
pub(crate) fn link<P: AsRef<Path>>(old_path: P, new_path: P) -> Result<()> {
    let old_path =
        CString::new(old_path.as_ref().as_os_str().as_bytes()).unwrap();
    let new_path =
        CString::new(new_path.as_ref().as_os_str().as_bytes()).unwrap();

    libc_like_syscall::link(old_path.as_ptr(), new_path.as_ptr())
        .map_err(Error::from_raw_os_error)
}

/// Deletes a name or possibly a file it refers to
///
/// Note: `path_name` should not contain byte 0, or this function will panic.
pub(crate) fn unlink<P: AsRef<Path>>(path_name: P) -> Result<()> {
    let path_name =
        CString::new(path_name.as_ref().as_os_str().as_bytes()).unwrap();

    libc_like_syscall::unlink(path_name.as_ptr())
        .map_err(Error::from_raw_os_error)
}

/// Makes a new name for a file
///
/// Note: `target` and `link_path` should not contain byte 0, or this function
/// will panic.
pub(crate) fn symlink<P: AsRef<Path>>(target: P, link_path: P) -> Result<()> {
    let target = CString::new(target.as_ref().as_os_str().as_bytes()).unwrap();
    let link_path =
        CString::new(link_path.as_ref().as_os_str().as_bytes()).unwrap();

    libc_like_syscall::symlink(target.as_ptr(), link_path.as_ptr())
        .map_err(Error::from_raw_os_error)
}

/// Creates a directory
///
/// Note: `path_name` should not contain byte 0, or this function will panic.
pub(crate) fn mkdir<P: AsRef<Path>>(path_name: P, mode: Mode) -> Result<()> {
    let path_name =
        CString::new(path_name.as_ref().as_os_str().as_bytes()).unwrap();

    libc_like_syscall::mkdir(path_name.as_ptr(), mode.bits())
        .map_err(Error::from_raw_os_error)
}

/// Deletes a directory
///
/// Note: `path_name` should not contain byte 0, or this function will panic.
pub(crate) fn rmdir<P: AsRef<Path>>(path_name: P) -> Result<()> {
    let path_name =
        CString::new(path_name.as_ref().as_os_str().as_bytes()).unwrap();

    libc_like_syscall::rmdir(path_name.as_ptr())
        .map_err(Error::from_raw_os_error)
}

/// Changes the name or location of a file
///
/// Note: `old_path` and `new_path` should not contain byte 0, or this function
/// will panic.
pub(crate) fn rename<P: AsRef<Path>>(old_path: P, new_path: P) -> Result<()> {
    let old_path =
        CString::new(old_path.as_ref().as_os_str().as_bytes()).unwrap();
    let new_path =
        CString::new(new_path.as_ref().as_os_str().as_bytes()).unwrap();

    libc_like_syscall::rename(old_path.as_ptr(), new_path.as_ptr())
        .map_err(Error::from_raw_os_error)
}

/// Get file status
///
/// Note: `path_name` should not contain byte 0, or this function will panic.
pub(crate) fn stat<P: AsRef<Path>>(
    path_name: P,
) -> Result<libc_like_syscall::Stat> {
    let path_name =
        CString::new(path_name.as_ref().as_os_str().as_bytes()).unwrap();
    let mut stat_buf = libc_like_syscall::Stat::default();

    match libc_like_syscall::stat(
        path_name.as_ptr(),
        &mut stat_buf as *mut libc_like_syscall::Stat,
    ) {
        Ok(()) => Ok(stat_buf),
        Err(errno) => Err(Error::from_raw_os_error(errno)),
    }
}

/// Get file status
///
/// Note: `path_name` should not contain byte 0, or this function will panic.
pub(crate) fn lstat<P: AsRef<Path>>(
    path_name: P,
) -> Result<libc_like_syscall::Stat> {
    let path_name =
        CString::new(path_name.as_ref().as_os_str().as_bytes()).unwrap();
    let mut stat_buf = libc_like_syscall::Stat::default();

    match libc_like_syscall::lstat(
        path_name.as_ptr(),
        &mut stat_buf as *mut libc_like_syscall::Stat,
    ) {
        Ok(()) => Ok(stat_buf),
        Err(errno) => Err(Error::from_raw_os_error(errno)),
    }
}

/// Get file status
pub(crate) fn fstat<Fd: AsFd>(fd: Fd) -> Result<libc_like_syscall::Stat> {
    let mut stat_buf = libc_like_syscall::Stat::default();

    match libc_like_syscall::fstat(
        fd.as_fd().as_raw_fd(),
        &mut stat_buf as *mut libc_like_syscall::Stat,
    ) {
        Ok(()) => Ok(stat_buf),
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

#[derive(Debug)]
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
pub struct Dirent {
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
        let name = OsStr::from_bytes(
            unsafe { CStr::from_ptr(name_start_ptr) }.to_bytes(),
        )
        .to_owned();
        let path = root.join(name.as_os_str());

        Self {
            ino: ino as u64,
            file_type: FileType::from(file_type),
            name,
            path,
        }
    }
}

impl Dir {
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

    pub(crate) fn readdir(&mut self) -> Result<Option<Dirent>> {
        if self.entries.is_empty() {
            let num_read = getdents64(self.fd.as_fd(), &mut self.buf)?;

            if num_read == 0 {
                return Ok(None);
            }

            let mut cursor = 0_usize;
            while cursor < num_read {
                unsafe {
                    let ptr_to_d_entry =
                        self.buf.as_ptr().add(cursor) as *const LinuxDirent64;

                    self.entries.push_back(Dirent::new(
                        (*ptr_to_d_entry).d_ino,
                        (*ptr_to_d_entry).d_type,
                        (ptr_to_d_entry as *const libc::c_char)
                            .add(OFFSET_D_NAME),
                        self.root.as_path(),
                    ));

                    cursor += (*ptr_to_d_entry).d_reclen as usize;
                }
            }
        }

        assert!(!self.entries.is_empty());
        Ok(self.entries.pop_front())
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
pub(crate) enum Whence {
    SeekSet,
    SeekCur,
    SeekEnd,
}

impl Whence {
    fn bits(&self) -> libc::c_int {
        match self {
            Whence::SeekSet => libc::SEEK_SET,
            Whence::SeekCur => libc::SEEK_CUR,
            Whence::SeekEnd => libc::SEEK_END,
        }
    }
}

/// reposition read/write file offset
pub(crate) fn lseek64<Fd: AsFd>(
    fd: Fd,
    offset: i64,
    whence: Whence,
) -> Result<u64> {
    let raw_fd = fd.as_fd().as_raw_fd();
    let whence = whence.bits();

    libc_like_syscall::lseek64(raw_fd, offset, whence)
        .map_err(Error::from_raw_os_error)
}

#[cfg(test)]
mod test {
    use super::*;

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
        let fd_with_read_permission =
            creat(file, Mode::from_bits(0o644).unwrap()).unwrap();
        let fd_with_write_permission =
            open(file, Flags::O_WRONLY, Mode::empty()).unwrap();
        assert_eq!(
            write(fd_with_write_permission.as_fd(), b"hello").unwrap(),
            5
        );

        let mut buffer = [0_u8; 5];
        assert_eq!(
            read(fd_with_read_permission.as_fd(), &mut buffer).unwrap(),
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
        write(fd.as_fd(), b"hello world").unwrap();

        let mut buf = [0_u8; 5];
        assert_eq!(pread(fd.as_fd(), &mut buf, 6).unwrap(), 5);

        assert_eq!(&buf, b"world");

        unlink(file).unwrap();
    }

    #[test]
    fn test_pwrite() {
        let file = "/tmp/test_pwrite";
        creat(file, Mode::from_bits(0o644).unwrap()).unwrap();

        let fd = open(file, Flags::O_RDWR, Mode::empty()).unwrap();
        write(fd.as_fd(), b"hello world").unwrap();

        assert_eq!(pwrite(fd.as_fd(), b"steve", 6).unwrap(), 5);

        let mut buf = [0_u8; 11];
        assert_eq!(pread(fd.as_fd(), &mut buf, 0).unwrap(), 11);

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

        assert_eq!(stat_buf.st_mode & libc::S_IFMT, libc::S_IFREG);
        unlink(file).unwrap();
    }

    #[test]
    fn test_fstat() {
        let file = "/tmp/test_fstat";
        let fd = creat(file, Mode::from_bits(0o644).unwrap()).unwrap();

        let stat_buf = fstat(fd.as_fd()).unwrap();

        assert_eq!(stat_buf.st_mode & libc::S_IFMT, libc::S_IFREG);
        unlink(file).unwrap();
    }

    #[test]
    fn test_lstat() {
        let file = "/tmp/test_lstat";
        let soft_link = "/tmp/test_lstat_link";
        creat(file, Mode::from_bits(0o644).unwrap()).unwrap();
        symlink(file, soft_link).unwrap();

        let stat_buf = lstat(soft_link).unwrap();

        assert_eq!(stat_buf.st_mode & libc::S_IFMT, libc::S_IFLNK);

        unlink(file).unwrap();
        unlink(soft_link).unwrap();
    }

    #[test]
    fn test_getdents64() {
        let tmp_dir = "/tmp";
        let tmp_dir_fd = open(tmp_dir, Flags::O_RDONLY, Mode::empty()).unwrap();
        let mut buf = [0_u8; 100];
        getdents64(tmp_dir_fd.as_fd(), &mut buf).unwrap();
    }

    #[test]
    fn test_opendir_readdir() {
        let output = std::process::Command::new("ls")
            .args(["-al", "/"])
            .output()
            .unwrap();
        assert!(output.stderr.is_empty());
        let num_of_file =
            output.stdout.iter().filter(|&&b| b == b'\n').count() - 1;

        let mut dir = Dir::opendir("/").unwrap();
        let mut n_files = 0;
        while let Ok(Some(_)) = dir.readdir() {
            n_files += 1;
        }

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
        write(fd.as_fd(), b"hello").unwrap();

        assert_eq!(lseek64(fd.as_fd(), 0, Whence::SeekSet).unwrap(), 0);

        unlink(file).unwrap();
    }
}
