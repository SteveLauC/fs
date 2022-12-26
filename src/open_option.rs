// Progress:
//
// * Implementation: Done
// * Tests:
//   To finish tests, you need to implement `Read/Write` on `File` first.

use crate::{
    backend::encapsulation::{open, Flags, Mode},
    file::File,
};
use std::{
    io::{Error, Result},
    os::unix::fs::OpenOptionsExt,
    path::Path,
};

#[derive(Debug, Clone)]
pub struct OpenOptions {
    // generic
    // access flags
    read: bool,  // O_RDONLY
    write: bool, // O_WRONLY
    // though `O_APPEND` technically belongs to `file status flag`
    append: bool, // O_APPEND

    // file creation flags
    truncate: bool,   // O_TRUNCATE
    create: bool,     // O_CREAT
    create_new: bool, // O_CREAT & O_EXCL

    // system-specific
    custom_flags: i32,
    mode: libc::mode_t,
}

impl OpenOptions {
    /// `get_access_mode()` and `get_creation_mode()` are helper functions used to map
    /// `OpenOptions` to the `flag` argument (of type `c_int`) of `open(2)`
    ///
    /// `get_access_mode()` will generates an int that possibly contains `O_RDONLY`,
    /// `O_WRONLY`, `O_RDWR` and `O_APPEND`
    ///
    /// #### Mappings
    ///
    /// There are actually 2^3 (8) cases, but when `append` is true, the value of
    /// `write` will also be set to true, which means that `true, true, true` and
    /// `true, false, true` can be abbreviated as `true, _, true`. Similarly,
    /// `false, true, true` and `false, false, true` can be aggregated as `false, _, true`
    ///
    /// 5: `(true, true, true)` => `O_RDWR | O_APPEND`
    /// 3: `(true, true, false)` => `O_RDWR`
    /// 5: `(true, false, true)` => `(true, true, true)` => `O_RDWR | O_APPEND`
    /// 1: `(true, false, false)` => `O_RDONLY`
    /// 4: `(false, true, true)` => `O_WRONLY | O_APPEND`
    /// 2: `(false, true, false)` => `O_WRONLY`
    /// 4: `(false, false, true)` => `(false, true, true)` => `O_WRONLY | O_APPEND`
    /// 6: `(false, false, false)` => `O_EINVAL` (One of `O_RDONLY/O_WRONLY/O_RDWR` must be included)
    fn get_access_mode(&self) -> Result<libc::c_int> {
        match (self.read, self.write, self.append) {
            (true, false, false) => Ok(libc::O_RDONLY), // case 1
            (false, true, false) => Ok(libc::O_WRONLY), // case 2
            (true, true, false) => Ok(libc::O_RDWR),    // case 3
            // When `append` is true, the value of `write` does not matter as
            // `.append()` is equivalent to `.write(true).append(true)`
            (false, _, true) => Ok(libc::O_WRONLY | libc::O_APPEND), // case 4
            (true, _, true) => Ok(libc::O_RDWR | libc::O_APPEND),    // case 5
            (false, false, false) => {
                // case 6
                Err(Error::from_raw_os_error(libc::EINVAL))
            }
        }
    }

    fn get_creation_mode(&self) -> Result<libc::c_int> {
        // Invalid value check.
        //
        // 1. To `truncate/create/create_new` a file, `write` must be set.
        // 2. You can not `truncate` and `append` a file at the same time.
        //    When `create_new` is set, `truncate` will be ignored.
        match (self.write, self.append) {
            (true, false) => {}
            (false, false) => {
                if self.truncate || self.create || self.create_new {
                    return Err(Error::from_raw_os_error(libc::EINVAL));
                }
            }
            (_, true) => {
                if self.truncate && !self.create_new {
                    return Err(Error::from_raw_os_error(libc::EINVAL));
                }
            }
        }

        Ok(match (self.create, self.truncate, self.create_new) {
            (false, false, false) => 0,
            (true, false, false) => libc::O_CREAT,
            (false, true, false) => libc::O_TRUNC,
            (true, true, false) => libc::O_CREAT | libc::O_TRUNC,
            // If `.create_new(true)` is set, `.create()` and `.truncate()` are ignored.
            (_, _, true) => libc::O_CREAT | libc::O_EXCL,
        })
    }

    pub fn new() -> Self {
        OpenOptions {
            // generic
            read: false,
            write: false,
            append: false,
            truncate: false,
            create: false,
            create_new: false,
            // system-specific
            custom_flags: 0,
            mode: 0o666,
        }
    }

    pub fn read(&mut self, read: bool) -> &mut Self {
        self.read = read;
        self
    }
    pub fn write(&mut self, write: bool) -> &mut Self {
        self.write = write;
        self
    }
    pub fn append(&mut self, append: bool) -> &mut Self {
        self.append = append;
        self
    }
    pub fn truncate(&mut self, truncate: bool) -> &mut Self {
        self.truncate = truncate;
        self
    }
    pub fn create(&mut self, create: bool) -> &mut Self {
        self.create = create;
        self
    }
    pub fn create_new(&mut self, create_new: bool) -> &mut Self {
        self.create_new = create_new;
        self
    }

    pub fn custom_flags(&mut self, flags: i32) -> &mut Self {
        self.custom_flags = flags;
        self
    }
    pub fn mode(&mut self, mode: u32) -> &mut Self {
        self.mode = mode as libc::mode_t;
        self
    }

    pub fn open<P: AsRef<Path>>(&self, path: P) -> Result<File> {
        let mut flag = Flags::from_bits(libc::O_CLOEXEC).unwrap();
        flag |= Flags::from_bits(self.get_access_mode()?).unwrap();
        flag |= Flags::from_bits(self.get_creation_mode()?).unwrap();
        flag |= Flags::from_bits(
            self.custom_flags as libc::c_int & !libc::O_ACCMODE,
        )
        .unwrap();

        let fd = open(path, flag, Mode::from_bits_truncate(self.mode))?;
        Ok(File { fd })
    }
}

impl OpenOptionsExt for OpenOptions {
    fn mode(&mut self, mode: u32) -> &mut Self {
        self.mode = mode;
        self
    }
    fn custom_flags(&mut self, flags: i32) -> &mut Self {
        self.custom_flags = flags;
        self
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn file_not_found() {
        let std_error = std::fs::OpenOptions::new()
            .read(true)
            .open("dOeSnOtExIst")
            .unwrap_err()
            .kind();

        let my_fs_error = OpenOptions::new()
            .read(true)
            .open("dOeSnOtExIst")
            .unwrap_err()
            .kind();

        assert_eq!(std_error, my_fs_error);
    }

    #[test]
    fn try_to_write_to_home_dir() {
        let std_error = std::fs::OpenOptions::new()
            .write(true)
            .open("/home")
            .unwrap_err()
            .kind();

        let my_fs_error = OpenOptions::new()
            .write(true)
            .open("/home")
            .unwrap_err()
            .kind();

        assert_eq!(std_error, my_fs_error);
    }
}
