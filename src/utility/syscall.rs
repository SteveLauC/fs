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
    blkcnt64_t, blksize_t, c_char, c_int, c_void, dev_t, gid_t, ino64_t,
    mode_t, nlink_t, off_t, size_t, time_t, uid_t, O_CREAT, O_RDONLY, O_TRUNC,
};
use sc::{
    nr::{
        CLOSE, FSTAT, GETDENTS64, LINK, LSTAT, MKDIR, OPEN, RENAME, RMDIR,
        STAT, SYMLINK, UNLINK, WRITE,
    },
    syscall,
};
use std::os::unix::io::RawFd;

/// A helper function to handle the return value of a raw syscall
#[inline]
fn syscall_result(ret_val: usize) -> Result<isize, c_int> {
    match ret_val as isize {
        minus_errno if (-4095..=-1).contains(&minus_errno) => {
            Err(-minus_errno as c_int)
        }
        success_ret_value => Ok(success_ret_value),
    }
}

#[inline]
pub(crate) fn open(
    pathname: *const c_char,
    flags: c_int,
) -> Result<RawFd, c_int> {
    let res = unsafe { syscall!(OPEN, pathname as usize, flags as usize) };

    syscall_result(res).map(|fd| fd as RawFd)
}

#[inline]
pub(crate) fn creat(
    pathname: *const c_char,
    mode: mode_t,
) -> Result<RawFd, c_int> {
    let res = unsafe {
        syscall!(
            OPEN,
            pathname as usize,
            (O_RDONLY | O_CREAT | O_TRUNC) as usize,
            mode as usize
        )
    };

    syscall_result(res).map(|fd| fd as RawFd)
}

// Only used in test.
#[inline]
fn close(fd: c_int) -> Result<(), c_int> {
    let res = unsafe { syscall!(CLOSE, fd as usize) };

    syscall_result(res).map(drop)
}

#[inline]
pub(crate) fn read(
    fd: c_int,
    buf: *mut c_void,
    count: size_t,
) -> Result<usize, c_int> {
    let res =
        unsafe { syscall!(READ, fd as usize, buf as usize, count as usize) };

    syscall_result(res).map(|num_read| num_read as usize)
}

#[inline]
pub(crate) fn write(
    fd: c_int,
    buf: *const c_void,
    count: size_t,
) -> Result<usize, c_int> {
    let res =
        unsafe { syscall!(WRITE, fd as usize, buf as usize, count as usize) };

    syscall_result(res).map(|num_read| num_read as usize)
}

#[inline]
pub(crate) fn link(
    oldpath: *const c_char,
    newpath: *const c_char,
) -> Result<(), c_int> {
    let res = unsafe { syscall!(LINK, oldpath as usize, newpath as usize) };

    syscall_result(res).map(drop)
}

#[inline]
pub(crate) fn unlink(pathname: *const c_char) -> Result<(), c_int> {
    let res = unsafe { syscall!(UNLINK, pathname as usize) };

    syscall_result(res).map(drop)
}

#[inline]
pub(crate) fn symlink(
    target: *const c_char,
    linkpath: *const c_char,
) -> Result<(), c_int> {
    let res = unsafe { syscall!(SYMLINK, target as usize, linkpath as usize) };

    syscall_result(res).map(drop)
}

#[inline]
pub(crate) fn mkdir(
    pathname: *const c_char,
    mode: mode_t,
) -> Result<(), c_int> {
    let res = unsafe { syscall!(MKDIR, pathname as usize, mode as usize) };

    syscall_result(res).map(drop)
}

#[inline]
pub(crate) fn rmdir(pathname: *const c_char) -> Result<(), c_int> {
    let res = unsafe { syscall!(RMDIR, pathname as usize) };

    syscall_result(res).map(drop)
}

#[inline]
pub(crate) fn rename(
    oldpath: *const c_char,
    newpath: *const c_char,
) -> Result<(), c_int> {
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
pub(crate) fn stat(
    pathname: *const c_char,
    statbuf: *mut Stat,
) -> Result<(), c_int> {
    let res = unsafe { syscall!(STAT, pathname as usize, statbuf as usize) };

    syscall_result(res).map(drop)
}

#[inline]
pub(crate) fn fstat(fd: c_int, statbuf: *mut Stat) -> Result<(), c_int> {
    let res = unsafe { syscall!(FSTAT, fd as usize, statbuf as usize) };

    syscall_result(res).map(drop)
}

#[inline]
pub(crate) fn lstat(
    pathname: *const c_char,
    statbuf: *mut Stat,
) -> Result<(), c_int> {
    let res = unsafe { syscall!(LSTAT, pathname as usize, statbuf as usize) };

    syscall_result(res).map(drop)
}

#[inline]
pub(crate) fn getdents64(
    fd: c_int,
    dirp: *mut c_void,
    count: size_t,
) -> Result<usize, c_int> {
    let res = unsafe {
        syscall!(GETDENTS64, fd as usize, dirp as usize, count as usize)
    };

    syscall_result(res).map(|num_read| num_read as usize)
}

#[cfg(test)]
mod test {
    use super::*;
    use libc::{EISDIR, ENOENT, ENOTDIR, O_WRONLY, S_IFLNK, S_IFMT, S_IFREG};

    #[test]
    fn test_open_close() {
        let fd = open(
            "/proc/self/mounts\0".as_ptr() as *const c_char,
            libc::O_RDONLY,
        )
        .unwrap();

        close(fd).unwrap();
    }

    #[test]
    fn test_creat_unlink() {
        let file = "/tmp/test_creat_unlink\0";
        let fd = creat(file.as_ptr() as *const c_char, 0o644).unwrap();
        close(fd).unwrap();
        unlink(file.as_ptr() as *const c_char).unwrap();
    }

    #[test]
    fn test_unlink_is_a_dir() {
        let dir = "/tmp/test_unlink_is_a_dir\0";
        mkdir(dir.as_ptr() as *const c_char, 0o777).unwrap();

        assert_eq!(unlink(dir.as_ptr() as *const c_char), Err(EISDIR));

        rmdir(dir.as_ptr() as *const c_char).unwrap();
    }

    #[test]
    fn test_read_write() {
        let file = "/tmp/test_read_write\0";
        let fd_with_read_permission =
            creat(file.as_ptr() as *const c_char, 0o644).unwrap();

        let fd_with_write_permission =
            open(file.as_ptr() as *const c_char, O_WRONLY).unwrap();

        let file_contents = "hello\0";
        assert_eq!(
            write(
                fd_with_write_permission,
                file_contents.as_ptr() as *const c_void,
                5
            ),
            Ok(5)
        );

        let read_buf = [0; 5];
        assert_eq!(
            read(fd_with_read_permission, read_buf.as_ptr() as *mut c_void, 5),
            Ok(5)
        );
        assert_eq!(&read_buf, b"hello");

        close(fd_with_read_permission).unwrap();
        close(fd_with_write_permission).unwrap();
        unlink(file.as_ptr() as *const c_char).unwrap();
    }

    #[test]
    fn test_link() {
        let file = "/tmp/test_link\0";
        let ln = "/tmp/test_link_ln\0";
        let fd = creat(file.as_ptr() as *const c_char, 0o644).unwrap();
        close(fd).unwrap();

        link(file.as_ptr() as *const c_char, ln.as_ptr() as *const c_char)
            .unwrap();

        unlink(file.as_ptr() as *const c_char).unwrap();
        unlink(ln.as_ptr() as *const c_char).unwrap();
    }

    #[test]
    fn test_mkdir() {
        let dir = "/tmp/test_mkdir\0";
        mkdir(dir.as_ptr() as *const c_char, 0o777).unwrap();

        rmdir(dir.as_ptr() as *const c_char).unwrap();
    }

    #[test]
    fn test_rmdir_not_a_directory() {
        let file = "/tmp/test_rmdir_not_a_directory\0";
        close(creat(file.as_ptr() as *const c_char, 0o644).unwrap()).unwrap();

        assert_eq!(rmdir(file.as_ptr() as *const c_char), Err(ENOTDIR));

        unlink(file.as_ptr() as *const c_char).unwrap();
    }

    #[test]
    fn test_rename() {
        let old_path = "/tmp/test_rename_old_path\0";
        let new_path = "/tmp/test_rename_new_path\0";
        close(creat(old_path.as_ptr() as *const c_char, 0o644).unwrap())
            .unwrap();

        rename(
            old_path.as_ptr() as *const c_char,
            new_path.as_ptr() as *const c_char,
        )
        .unwrap();

        assert_eq!(unlink(old_path.as_ptr() as *const c_char), Err(ENOENT));

        unlink(new_path.as_ptr() as *const c_char).unwrap();
    }

    #[test]
    fn test_symlink() {
        let file = "/tmp/test_symlink\0";
        let soft_link = "/tmp/test_symlink_soft_link\0";
        close(creat(file.as_ptr() as *const c_char, 0o644).unwrap()).unwrap();

        symlink(
            file.as_ptr() as *const c_char,
            soft_link.as_ptr() as *const c_char,
        )
        .unwrap();

        unlink(soft_link.as_ptr() as *const c_char).unwrap();
        unlink(file.as_ptr() as *const c_char).unwrap();
    }

    #[test]
    fn test_stat() {
        let file = "/tmp/test_stat\0";
        close(creat(file.as_ptr() as *const c_char, 0o644).unwrap()).unwrap();

        let mut stat_buf = Stat::default();
        stat(file.as_ptr() as *const c_char, &mut stat_buf as *mut Stat)
            .unwrap();

        assert_eq!(stat_buf.st_mode & S_IFMT, S_IFREG);
        unlink(file.as_ptr() as *const c_char).unwrap();
    }

    #[test]
    fn test_fstat() {
        let file = "/tmp/test_fstat\0";
        let fd = creat(file.as_ptr() as *const c_char, 0o644).unwrap();

        let mut stat_buf = Stat::default();
        fstat(fd, &mut stat_buf as *mut Stat).unwrap();

        assert_eq!(stat_buf.st_mode & S_IFMT, S_IFREG);
        close(fd).unwrap();
        unlink(file.as_ptr() as *const c_char).unwrap();
    }

    #[test]
    fn test_lstat() {
        let file = "/tmp/test_lstat\0";
        let soft_link = "/tmp/test_lstat_link\0";
        close(creat(file.as_ptr() as *const c_char, 0o644).unwrap()).unwrap();
        symlink(
            file.as_ptr() as *const c_char,
            soft_link.as_ptr() as *const c_char,
        )
        .unwrap();

        let mut stat_buf = Stat::default();
        lstat(
            soft_link.as_ptr() as *const c_char,
            &mut stat_buf as *mut Stat,
        )
        .unwrap();

        assert_eq!(stat_buf.st_mode & S_IFMT, S_IFLNK);

        unlink(file.as_ptr() as *const c_char).unwrap();
        unlink(soft_link.as_ptr() as *const c_char).unwrap();
    }

    #[test]
    fn test_getdents64() {
        let tmp_dir = "/tmp\0";
        let tmp_dir_fd =
            open(tmp_dir.as_ptr() as *const c_char, O_RDONLY).unwrap();
        let mut buf = [0_u8; 100];
        let num_read =
            getdents64(tmp_dir_fd, &mut buf as *mut u8 as *mut c_void, 100)
                .unwrap();
    }

    #[test]
    fn test_getdents64_not_a_directory() {
        let file = "/tmp/test_getdents64_not_a_directory\0";
        let fd = creat(file.as_ptr() as *const c_char, 0o644).unwrap();

        let mut buf = [0_u8; 100];
        assert_eq!(
            getdents64(fd, &mut buf as *mut u8 as *mut c_void, 100),
            Err(ENOTDIR)
        );

        close(fd).unwrap();
        unlink(file.as_ptr() as *const c_char).unwrap();
    }
}
