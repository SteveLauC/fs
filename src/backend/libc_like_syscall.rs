//! libc-like syscall bindings
//!
//! Different from `libc`, we don't have `errno`, so we can't return `-1` and set
//! `errno` to indicate the specific error case when on error.
//!
//! Instead, All bindings in this module return a customized `Result<T, c_int>`
//! type, where `T` is the successful return value of a specific syscall, `c_int`
//! is the value of `errno`. For example, `open(2)` returns `Ok(an_open_fd)` on
//! success, `Err(errno_value)` on error. `read(2)` returns the
//! `Ok(the_num_of_bytes_read)` on success, `Err(errno_value)` on error.

use libc::{
    blkcnt64_t, blksize_t, c_char, c_int, c_long, c_uint, c_void, dev_t, gid_t, ino64_t, mode_t,
    nlink_t, off64_t, off_t, size_t, time_t, uid_t, O_CREAT, O_RDONLY, O_TRUNC,
};
use sc::syscall;
use std::os::unix::io::RawFd;

/// A helper function to handle the return value of a raw syscall
#[inline]
fn syscall_result(ret_val: usize) -> Result<isize, c_int> {
    match ret_val as isize {
        minus_errno if (-4095..=-1).contains(&minus_errno) => Err(-minus_errno as c_int),
        success_ret_value => Ok(success_ret_value),
    }
}

#[inline]
pub(crate) fn open(pathname: *const c_char, flags: c_int, mode: mode_t) -> Result<RawFd, c_int> {
    let res = unsafe { syscall!(OPEN, pathname as usize, flags as usize, mode) };

    syscall_result(res).map(|fd| fd as RawFd)
}

#[inline]
pub(crate) fn creat(pathname: *const c_char, mode: mode_t) -> Result<RawFd, c_int> {
    open(pathname, O_RDONLY | O_CREAT | O_TRUNC, mode)
}

// Only used in test.
#[inline]
#[cfg(test)]
fn close(fd: c_int) -> Result<(), c_int> {
    let res = unsafe { syscall!(CLOSE, fd as usize) };

    syscall_result(res).map(drop)
}

#[inline]
pub(crate) fn read(fd: c_int, buf: *mut c_void, count: size_t) -> Result<usize, c_int> {
    let res = unsafe { syscall!(READ, fd as usize, buf as usize, count) };

    syscall_result(res).map(|num_read| num_read as usize)
}

#[inline]
pub(crate) fn write(fd: c_int, buf: *const c_void, count: size_t) -> Result<usize, c_int> {
    let res = unsafe { syscall!(WRITE, fd as usize, buf as usize, count) };

    syscall_result(res).map(|num_read| num_read as usize)
}

#[inline]
pub(crate) fn pread(
    fd: c_int,
    buf: *mut c_void,
    count: size_t,
    offset: off_t,
) -> Result<usize, c_int> {
    let res = unsafe { syscall!(PREAD64, fd as usize, buf as usize, count, offset as usize) };

    syscall_result(res).map(|num_read| num_read as usize)
}

#[inline]
pub(crate) fn pwrite(
    fd: c_int,
    buf: *const c_void,
    count: size_t,
    offset: off_t,
) -> Result<usize, c_int> {
    let res = unsafe { syscall!(PWRITE64, fd as usize, buf as usize, count, offset as usize) };

    syscall_result(res).map(|num_written| num_written as usize)
}

#[inline]
pub(crate) fn link(oldpath: *const c_char, newpath: *const c_char) -> Result<(), c_int> {
    let res = unsafe { syscall!(LINK, oldpath as usize, newpath as usize) };

    syscall_result(res).map(drop)
}

#[inline]
pub(crate) fn unlink(pathname: *const c_char) -> Result<(), c_int> {
    let res = unsafe { syscall!(UNLINK, pathname as usize) };

    syscall_result(res).map(drop)
}

#[inline]
pub(crate) fn symlink(target: *const c_char, linkpath: *const c_char) -> Result<(), c_int> {
    let res = unsafe { syscall!(SYMLINK, target as usize, linkpath as usize) };

    syscall_result(res).map(drop)
}

#[inline]
pub(crate) fn mkdir(pathname: *const c_char, mode: mode_t) -> Result<(), c_int> {
    let res = unsafe { syscall!(MKDIR, pathname as usize, mode as usize) };

    syscall_result(res).map(drop)
}

#[inline]
pub(crate) fn rmdir(pathname: *const c_char) -> Result<(), c_int> {
    let res = unsafe { syscall!(RMDIR, pathname as usize) };

    syscall_result(res).map(drop)
}

#[inline]
pub(crate) fn rename(oldpath: *const c_char, newpath: *const c_char) -> Result<(), c_int> {
    let res = unsafe { syscall!(RENAME, oldpath as usize, newpath as usize) };

    syscall_result(res).map(drop)
}

#[repr(C)]
#[derive(Default, Debug)]
pub(crate) struct Stat {
    pub(crate) st_dev: dev_t,
    pub(crate) st_ino: ino64_t,
    pub(crate) st_nlink: nlink_t,
    pub(crate) st_mode: mode_t,
    pub(crate) st_uid: uid_t,
    pub(crate) st_gid: gid_t,
    __pad0: c_int,
    pub(crate) st_rdev: dev_t,
    pub(crate) st_size: off_t,
    pub(crate) st_blksize: blksize_t,
    pub(crate) st_blocks: blkcnt64_t,
    pub(crate) st_atime: time_t,
    pub(crate) st_atime_nsec: i64,
    pub(crate) st_mtime: time_t,
    pub(crate) st_mtime_nsec: i64,
    pub(crate) st_ctime: time_t,
    pub(crate) st_ctime_nsec: i64,
    __unused: [i64; 3],
}

#[inline]
pub(crate) fn stat(pathname: *const c_char, statbuf: *mut Stat) -> Result<(), c_int> {
    let res = unsafe { syscall!(STAT, pathname as usize, statbuf as usize) };

    syscall_result(res).map(drop)
}

#[inline]
pub(crate) fn fstat(fd: c_int, statbuf: *mut Stat) -> Result<(), c_int> {
    let res = unsafe { syscall!(FSTAT, fd as usize, statbuf as usize) };

    syscall_result(res).map(drop)
}

#[inline]
pub(crate) fn lstat(pathname: *const c_char, statbuf: *mut Stat) -> Result<(), c_int> {
    let res = unsafe { syscall!(LSTAT, pathname as usize, statbuf as usize) };

    syscall_result(res).map(drop)
}

#[repr(C)]
#[derive(Default, Debug, Clone)]
pub(crate) struct Statx {
    pub(crate) stx_mask: u32,
    pub(crate) stx_blksize: u32,
    pub(crate) stx_attributes: u64,
    pub(crate) stx_nlink: u32,
    pub(crate) stx_uid: u32,
    pub(crate) stx_gid: u32,
    pub(crate) stx_mode: u16,
    __statx_pad1: [u16; 1],
    pub(crate) stx_ino: u64,
    pub(crate) stx_size: u64,
    pub(crate) stx_blocks: u64,
    pub(crate) stx_attributes_mask: u64,
    pub(crate) stx_atime: StatxTimestamp,
    pub(crate) stx_btime: StatxTimestamp,
    pub(crate) stx_ctime: StatxTimestamp,
    pub(crate) stx_mtime: StatxTimestamp,
    pub(crate) stx_rdev_major: u32,
    pub(crate) stx_rdev_minor: u32,
    pub(crate) stx_dev_major: u32,
    pub(crate) stx_dev_minor: u32,
    pub(crate) stx_mnt_id: u64,
    __statx_pad2: u64,
    __statx_pad3: [u64; 12],
}

#[repr(C)]
#[derive(Default, Debug, Copy, Clone)]
pub(crate) struct StatxTimestamp {
    pub(crate) tv_sec: i64,
    pub(crate) tv_nsec: u32,
    __statx_timestamp_pad1: [i32; 1],
}

pub(crate) fn statx(
    dirfd: c_int,
    pathname: *const c_char,
    flags: c_int,
    mask: c_uint,
    statx_buf: *mut Statx,
) -> Result<(), c_int> {
    let res = unsafe {
        syscall!(
            STATX,
            dirfd as usize,
            pathname as usize,
            flags as usize,
            mask as usize,
            statx_buf as usize
        )
    };

    syscall_result(res).map(drop)
}

#[inline]
pub(crate) fn getdents64(fd: c_int, dirp: *mut c_void, count: size_t) -> Result<usize, c_int> {
    let res = unsafe { syscall!(GETDENTS64, fd as usize, dirp as usize, count) };

    syscall_result(res).map(|num_read| num_read as usize)
}

#[inline]
pub(crate) fn chroot(path: *const c_char) -> Result<(), c_int> {
    let res = unsafe { syscall!(CHROOT, path as usize) };

    syscall_result(res).map(drop)
}

#[inline]
pub(crate) fn lseek64(fd: c_int, offset: off64_t, whence: c_int) -> Result<u64, c_int> {
    let res = unsafe { syscall!(LSEEK, fd as usize, offset as usize, whence as usize) };

    syscall_result(res).map(|new_offset| new_offset as u64)
}

#[inline]
pub(crate) fn readlink(
    pathname: *const c_char,
    buf: *mut c_char,
    bufsiz: size_t,
) -> Result<u64, c_int> {
    let res = unsafe { syscall!(READLINK, pathname as usize, buf as usize, bufsiz) };

    syscall_result(res).map(|bytes_read| bytes_read as u64)
}

/// A simplified version of `fcntl(2)`, supports only two arguments.
#[inline]
pub(crate) fn fcntl_with_two_args(fd: c_int, cmd: c_int) -> Result<c_int, c_int> {
    let res = unsafe { syscall!(FCNTL, fd as usize, cmd as usize) };

    syscall_result(res).map(|res| res as c_int)
}

#[inline]
pub(crate) fn fsync(fd: c_int) -> Result<(), c_int> {
    let res = unsafe { syscall!(FSYNC, fd as usize) };
    syscall_result(res).map(drop)
}

#[inline]
pub(crate) fn fdatasync(fd: c_int) -> Result<(), c_int> {
    let res = unsafe { syscall!(FDATASYNC, fd as usize) };
    syscall_result(res).map(drop)
}

#[inline]
pub(crate) fn ftruncate(fd: c_int, length: off_t) -> Result<(), c_int> {
    let res = unsafe { syscall!(FTRUNCATE, fd as usize, length as usize) };
    syscall_result(res).map(drop)
}

#[inline]
pub(crate) fn chmod(pathname: *const c_char, mode: mode_t) -> Result<(), c_int> {
    let res = unsafe { syscall!(CHMOD, pathname as usize, mode as usize) };
    syscall_result(res).map(drop)
}

#[inline]
pub(crate) fn fchmod(fd: c_int, mode: mode_t) -> Result<(), c_int> {
    let res = unsafe { syscall!(FCHMOD, fd as usize, mode as usize) };
    syscall_result(res).map(drop)
}

#[repr(C)]
#[derive(Default, Copy, Clone)]
pub(crate) struct Timespec {
    pub(crate) tv_sec: time_t,
    pub(crate) tv_nsec: c_long,
}

#[inline]
pub(crate) fn utimensat(
    dirfd: c_int,
    pathname: *const c_char,
    times: *const Timespec,
    flags: c_int,
) -> Result<(), c_int> {
    let res = unsafe {
        syscall!(
            UTIMENSAT,
            dirfd as usize,
            pathname as usize,
            times as usize,
            flags as usize
        )
    };
    syscall_result(res).map(drop)
}

#[inline]
pub(crate) fn chown(pathname: *const c_char, owner: uid_t, group: gid_t) -> Result<(), c_int> {
    let res = unsafe { syscall!(CHOWN, pathname as usize, owner as usize, group as usize) };
    syscall_result(res).map(drop)
}

#[inline]
pub(crate) fn fchown(fd: c_int, owner: uid_t, group: gid_t) -> Result<(), c_int> {
    let res = unsafe { syscall!(FCHOWN, fd as usize, owner as usize, group as usize) };
    syscall_result(res).map(drop)
}

#[inline]
pub(crate) fn lchown(pathname: *const c_char, owner: uid_t, group: gid_t) -> Result<(), c_int> {
    let res = unsafe { syscall!(LCHOWN, pathname as usize, owner as usize, group as usize) };
    syscall_result(res).map(drop)
}

#[cfg(test)]
mod test {
    use super::*;
    use libc::{
        EISDIR, ENOENT, ENOTDIR, O_RDWR, O_WRONLY, SEEK_SET, STATX_ALL, S_IFLNK, S_IFMT, S_IFREG,
    };

    #[test]
    fn test_open_close() {
        let fd = open("/proc/self/mounts\0".as_ptr().cast(), O_RDONLY, 0).unwrap();

        close(fd).unwrap();
    }

    #[test]
    fn test_creat_unlink() {
        let file = "/tmp/test_creat_unlink\0";
        let fd = creat(file.as_ptr().cast(), 0o644).unwrap();
        close(fd).unwrap();
        unlink(file.as_ptr().cast()).unwrap();
    }

    #[test]
    fn test_unlink_is_a_dir() {
        let dir = "/tmp/test_unlink_is_a_dir\0";
        mkdir(dir.as_ptr().cast(), 0o777).unwrap();

        assert_eq!(unlink(dir.as_ptr().cast()), Err(EISDIR));

        rmdir(dir.as_ptr().cast()).unwrap();
    }

    #[test]
    fn test_read_write() {
        let file = "/tmp/test_read_write\0";
        let fd_with_read_permission = creat(file.as_ptr().cast(), 0o644).unwrap();

        let fd_with_write_permission = open(file.as_ptr().cast(), O_WRONLY, 0).unwrap();

        let file_contents = "hello\0";
        assert_eq!(
            write(fd_with_write_permission, file_contents.as_ptr().cast(), 5),
            Ok(5)
        );

        let mut read_buf: [u8; 5] = [0; 5];
        assert_eq!(
            read(fd_with_read_permission, read_buf.as_mut_ptr().cast(), 5),
            Ok(5)
        );
        assert_eq!(&read_buf, b"hello");

        close(fd_with_read_permission).unwrap();
        close(fd_with_write_permission).unwrap();
        unlink(file.as_ptr().cast()).unwrap();
    }

    #[test]
    fn test_link() {
        let file = "/tmp/test_link\0";
        let ln = "/tmp/test_link_ln\0";
        let fd = creat(file.as_ptr().cast(), 0o644).unwrap();
        close(fd).unwrap();

        link(file.as_ptr().cast(), ln.as_ptr().cast()).unwrap();

        unlink(file.as_ptr().cast()).unwrap();
        unlink(ln.as_ptr().cast()).unwrap();
    }

    #[test]
    fn test_mkdir() {
        let dir = "/tmp/test_mkdir\0";
        mkdir(dir.as_ptr().cast(), 0o777).unwrap();

        rmdir(dir.as_ptr().cast()).unwrap();
    }

    #[test]
    fn test_rmdir_not_a_directory() {
        let file = "/tmp/test_rmdir_not_a_directory\0";
        close(creat(file.as_ptr().cast(), 0o644).unwrap()).unwrap();

        assert_eq!(rmdir(file.as_ptr().cast()), Err(ENOTDIR));

        unlink(file.as_ptr().cast()).unwrap();
    }

    #[test]
    fn test_rename() {
        let old_path = "/tmp/test_rename_old_path\0";
        let new_path = "/tmp/test_rename_new_path\0";
        close(creat(old_path.as_ptr().cast(), 0o644).unwrap()).unwrap();

        rename(old_path.as_ptr().cast(), new_path.as_ptr().cast()).unwrap();

        assert_eq!(unlink(old_path.as_ptr().cast()), Err(ENOENT));

        unlink(new_path.as_ptr().cast()).unwrap();
    }

    #[test]
    fn test_symlink() {
        let file = "/tmp/test_symlink\0";
        let soft_link = "/tmp/test_symlink_soft_link\0";
        close(creat(file.as_ptr().cast(), 0o644).unwrap()).unwrap();

        symlink(file.as_ptr().cast(), soft_link.as_ptr().cast()).unwrap();

        unlink(soft_link.as_ptr().cast()).unwrap();
        unlink(file.as_ptr().cast()).unwrap();
    }

    #[test]
    fn test_stat() {
        let file = "/tmp/test_stat\0";
        close(creat(file.as_ptr().cast(), 0o644).unwrap()).unwrap();

        let mut stat_buf = Stat::default();
        stat(file.as_ptr().cast(), &mut stat_buf as *mut Stat).unwrap();

        assert_eq!(stat_buf.st_mode & S_IFMT, S_IFREG);
        unlink(file.as_ptr().cast()).unwrap();
    }

    #[test]
    fn test_fstat() {
        let file = "/tmp/test_fstat\0";
        let fd = creat(file.as_ptr().cast(), 0o644).unwrap();

        let mut stat_buf = Stat::default();
        fstat(fd, &mut stat_buf as *mut Stat).unwrap();

        assert_eq!(stat_buf.st_mode & S_IFMT, S_IFREG);
        close(fd).unwrap();
        unlink(file.as_ptr().cast()).unwrap();
    }

    #[test]
    fn test_lstat() {
        let file = "/tmp/test_lstat\0";
        let soft_link = "/tmp/test_lstat_link\0";
        close(creat(file.as_ptr().cast(), 0o644).unwrap()).unwrap();
        symlink(file.as_ptr().cast(), soft_link.as_ptr().cast()).unwrap();

        let mut stat_buf = Stat::default();
        lstat(soft_link.as_ptr().cast(), &mut stat_buf as *mut Stat).unwrap();

        assert_eq!(stat_buf.st_mode & S_IFMT, S_IFLNK);

        unlink(file.as_ptr().cast()).unwrap();
        unlink(soft_link.as_ptr().cast()).unwrap();
    }

    #[test]
    fn test_statx() {
        let file = "/tmp/test_statx\0";
        close(creat(file.as_ptr().cast(), 0o644).unwrap()).unwrap();

        let mut statx_buf = Statx::default();
        statx(
            0,
            file.as_ptr().cast(),
            0,
            STATX_ALL,
            &mut statx_buf as *mut Statx,
        )
        .unwrap();

        assert_eq!(statx_buf.stx_mode & S_IFMT as u16, S_IFREG as u16);
        unlink(file.as_ptr().cast()).unwrap();
    }

    #[test]
    fn test_getdents64() {
        let tmp_dir = "/tmp\0";
        let tmp_dir_fd = open(tmp_dir.as_ptr().cast(), O_RDONLY, 0).unwrap();
        let mut buf = [0_u8; 100];
        getdents64(tmp_dir_fd, (&mut buf as *mut u8).cast(), 100).unwrap();
    }

    #[test]
    fn test_getdents64_not_a_directory() {
        let file = "/tmp/test_getdents64_not_a_directory\0";
        let fd = creat(file.as_ptr().cast(), 0o644).unwrap();

        let mut buf = [0_u8; 100];
        assert_eq!(
            getdents64(fd, (&mut buf as *mut u8).cast(), 100),
            Err(ENOTDIR)
        );

        close(fd).unwrap();
        unlink(file.as_ptr().cast()).unwrap();
    }

    #[test]
    fn test_chroot() {
        assert_eq!(chroot(".\0".as_ptr().cast()), Err(libc::EPERM));
    }

    #[test]
    fn test_lseek64() {
        let file = "/tmp/test_lseek\0";
        let fd = creat(file.as_ptr().cast(), 0o644).unwrap();
        close(fd).unwrap();

        let fd = open(file.as_ptr().cast(), O_RDWR, 0).unwrap();

        write(fd, "hello\0".as_ptr().cast(), 5).unwrap();

        assert_eq!(lseek64(fd, 0, SEEK_SET).unwrap(), 0);

        close(fd).unwrap();
        unlink(file.as_ptr().cast()).unwrap();
    }

    #[test]
    fn test_pread() {
        let file = "/tmp/test_pread\0";
        let fd = creat(file.as_ptr().cast(), 0o644).unwrap();
        close(fd).unwrap();

        let fd = open(file.as_ptr().cast(), O_RDWR, 0).unwrap();
        write(fd, "hello world\0".as_ptr().cast(), 11).unwrap();

        let mut buf = [0_u8; 5];
        assert_eq!(pread(fd, buf.as_mut_ptr().cast(), 5, 6).unwrap(), 5);

        assert_eq!(&buf, b"world");

        close(fd).unwrap();
        unlink(file.as_ptr().cast()).unwrap();
    }

    #[test]
    fn test_pwrite() {
        let file = "/tmp/test_pwrite\0";
        let fd = creat(file.as_ptr().cast(), 0o644).unwrap();
        close(fd).unwrap();

        let fd = open(file.as_ptr().cast(), O_RDWR, 0).unwrap();
        write(fd, "hello world\0".as_ptr().cast(), 11).unwrap();

        assert_eq!(pwrite(fd, "steve\0".as_ptr().cast(), 5, 6).unwrap(), 5);

        let mut buf = [0_u8; 11];
        assert_eq!(pread(fd, buf.as_mut_ptr().cast(), 11, 0).unwrap(), 11);

        assert_eq!(&buf, b"hello steve");

        close(fd).unwrap();
        unlink(file.as_ptr().cast()).unwrap();
    }

    #[test]
    fn test_readlink() {
        let file = "/tmp/test_readlink\0";
        let soft_link = "/tmp/test_readlink_link\0";
        close(creat(file.as_ptr().cast(), 0o644).unwrap()).unwrap();
        symlink(file.as_ptr().cast(), soft_link.as_ptr().cast()).unwrap();

        let buf = [0; 19];
        let bytes_read = readlink(
            soft_link.as_ptr().cast(),
            buf.as_ptr().cast_mut().cast(),
            20,
        )
        .unwrap();

        assert_eq!(buf, file.as_bytes());

        unlink(file.as_ptr().cast()).unwrap();
        unlink(soft_link.as_ptr().cast()).unwrap();
    }

    #[test]
    fn test_fsync() {
        let file = "/tmp/test_fsync\0";
        let fd = creat(file.as_ptr().cast(), 0o644).unwrap();

        fsync(fd).unwrap();

        close(fd).unwrap();
        unlink(file.as_ptr().cast()).unwrap();
    }

    #[test]
    fn test_fdatasync() {
        let file = "/tmp/test_fdatasync\0";
        let fd = creat(file.as_ptr().cast(), 0o644).unwrap();

        fdatasync(fd).unwrap();

        close(fd).unwrap();
        unlink(file.as_ptr().cast()).unwrap();
    }

    #[test]
    fn test_ftruncate() {
        let file = "/tmp/test_ftruncate\0";

        let fd_with_write_permission = open(file.as_ptr().cast(), O_CREAT | O_RDWR, 0).unwrap();
        assert_eq!(
            5,
            write(fd_with_write_permission, "hello\0".as_ptr().cast(), 5).unwrap()
        );
        ftruncate(fd_with_write_permission, 3).unwrap();

        let mut stat_buf = Stat::default();
        fstat(fd_with_write_permission, &mut stat_buf).unwrap();
        assert_eq!(stat_buf.st_size, 3);

        close(fd_with_write_permission).unwrap();
        unlink(file.as_ptr().cast()).unwrap();
    }

    #[test]
    fn test_chmod() {
        let file = "/tmp/test_chmod_libc\0";
        close(creat(file.as_ptr().cast(), 0o644).unwrap()).unwrap();

        chmod(file.as_ptr().cast(), 0o000).unwrap();

        let mut statx_buf = Statx::default();
        statx(
            0,
            file.as_ptr().cast(),
            0,
            STATX_ALL,
            &mut statx_buf as *mut Statx,
        )
        .unwrap();

        assert_eq!(statx_buf.stx_mode & 0o777, 0o000);

        unlink(file.as_ptr().cast()).unwrap();
    }

    #[test]
    fn test_fchmod() {
        let file = "/tmp/test_fchmod_libc\0";
        let fd = creat(file.as_ptr().cast(), 0o644).unwrap();

        fchmod(fd, 0o000).unwrap();

        let mut statx_buf = Statx::default();
        statx(
            0,
            file.as_ptr().cast(),
            0,
            STATX_ALL,
            &mut statx_buf as *mut Statx,
        )
        .unwrap();

        assert_eq!(statx_buf.stx_mode & 0o777, 0o000);

        close(fd).unwrap();
        unlink(file.as_ptr().cast()).unwrap();
    }

    #[test]
    fn test_utimensat() {
        let file = "/tmp/test_utimensat_libc\0";
        close(creat(file.as_ptr().cast(), 0o644).unwrap()).unwrap();

        let mut times = [Timespec::default(); 2];
        times[0].tv_nsec = libc::UTIME_OMIT;
        times[1].tv_nsec = libc::UTIME_OMIT;
        utimensat(0, file.as_ptr().cast(), &times as *const Timespec, 0).unwrap();

        unlink(file.as_ptr().cast()).unwrap();
    }

    #[test]
    fn test_chown() {
        let file = "/tmp/test_chown_libc\0";
        close(creat(file.as_ptr().cast(), 0o644).unwrap()).unwrap();

        let mut statx_buf = Statx::default();
        statx(
            0,
            file.as_ptr().cast(),
            0,
            STATX_ALL,
            &mut statx_buf as *mut Statx,
        )
        .unwrap();
        let uid = statx_buf.stx_uid;
        let gid = statx_buf.stx_gid;

        // Only user with privilege can change owner of a file
        // we "update" its owner to its owner, which is NOT a change.
        chown(file.as_ptr().cast(), uid, gid).unwrap();
        chown(file.as_ptr().cast(), uid_t::MAX, gid_t::MAX).unwrap();
        chown(file.as_ptr().cast(), uid, gid_t::MAX).unwrap();
        chown(file.as_ptr().cast(), uid_t::MAX, gid_t::MAX).unwrap();
        unlink(file.as_ptr().cast()).unwrap();
        chown(file.as_ptr().cast(), uid, gid).unwrap_err();
    }

    #[test]
    fn test_fchown() {
        let file = "/tmp/test_fchown_libc\0";
        let fd = creat(file.as_ptr().cast(), 0o644).unwrap();

        let mut statx_buf = Statx::default();
        statx(
            0,
            file.as_ptr().cast(),
            0,
            STATX_ALL,
            &mut statx_buf as *mut Statx,
        )
        .unwrap();
        let uid = statx_buf.stx_uid;
        let gid = statx_buf.stx_gid;

        // Only user with privilege can change owner of a file
        // we "update" its owner to its owner, which is NOT a change.
        fchown(fd, uid, gid).unwrap();
        fchown(fd, uid_t::MAX, gid_t::MAX).unwrap();
        fchown(fd, uid, gid_t::MAX).unwrap();
        fchown(fd, uid_t::MAX, gid_t::MAX).unwrap();
        unlink(file.as_ptr().cast()).unwrap();
        fchown(99999, uid, gid).unwrap_err();
    }
    #[test]
    fn test_lchown() {
        let file = "/tmp/test_lchown_libc\0";
        let link = "/tmp/test_lchown_link_libc\0";
        close(creat(file.as_ptr().cast(), 0o644).unwrap()).unwrap();
        symlink(file.as_ptr().cast(), link.as_ptr().cast()).unwrap();

        let mut statx_buf = Statx::default();
        statx(
            0,
            link.as_ptr().cast(),
            libc::AT_SYMLINK_NOFOLLOW,
            STATX_ALL,
            &mut statx_buf as *mut Statx,
        )
        .unwrap();
        let uid = statx_buf.stx_uid;
        let gid = statx_buf.stx_gid;

        lchown(link.as_ptr().cast(), uid, gid).unwrap();
        lchown(link.as_ptr().cast(), uid_t::MAX, gid_t::MAX).unwrap();
        lchown(link.as_ptr().cast(), uid, gid_t::MAX).unwrap();
        lchown(link.as_ptr().cast(), uid_t::MAX, gid_t::MAX).unwrap();
        unlink(link.as_ptr().cast()).unwrap();
        lchown(link.as_ptr().cast(), uid, gid).unwrap_err();

        unlink(file.as_ptr().cast()).unwrap();
    }
}
