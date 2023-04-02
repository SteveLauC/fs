use crate::backend::encapsulation;
use std::os::unix::fs::FileTypeExt;

/// A structure representing a type of file with accessors for each file type.
/// It is returned by `Metadata::file_type` method.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct FileType(pub(crate) encapsulation::FileType);

impl FileType {
    /// Tests whether this file type represents a directory. The result is 
    /// mutually exclusive to the results of `is_file` and `is_symlink`; only 
    /// zero or one of these tests may pass.
    #[inline]
    pub fn is_dir(&self) -> bool {
        self.0 == encapsulation::FileType::Directory
    }

    /// Tests whether this file type represents a regular file. The result is 
    /// mutually exclusive to the results of `is_dir` and `is_symlink`; only zero 
    /// or one of these tests may pass.
    ///
    /// When the goal is simply to read from (or write to) the source, the most 
    /// reliable way to test the source can be read (or written to) is to open it.
    /// Only using `is_file` can break workflows like `diff <( prog_a )` on a 
    /// Unix-like system for example. See `File::open` or `OpenOptions::open` 
    /// for more information.
    #[inline]
    pub fn is_file(&self) -> bool {
        self.0 == encapsulation::FileType::RegularFile
    }

    /// Tests whether this file type represents a symbolic link. The result is 
    /// mutually exclusive to the results of `is_dir` and `is_file`; only zero or 
    /// one of these tests may pass.
    ///
    /// The underlying Metadata struct needs to be retrieved with the 
    /// `fs::symlink_metadata` function and not the `fs::metadata` function. 
    /// The `fs::metadata` function follows symbolic links, so `is_symlink` 
    /// would always return false for the target file.
    #[inline]
    pub fn is_symlink(&self) -> bool {
        self.0 == encapsulation::FileType::Symlink
    }
}

impl FileTypeExt for FileType {
    #[inline]
    fn is_block_device(&self) -> bool {
        self.0 == encapsulation::FileType::BlkDev
    }

    #[inline]
    fn is_char_device(&self) -> bool {
        self.0 == encapsulation::FileType::CharDev
    }

    #[inline]
    fn is_fifo(&self) -> bool {
        self.0 == encapsulation::FileType::Fifo
    }

    #[inline]
    fn is_socket(&self) -> bool {
        self.0 == encapsulation::FileType::Socket
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::file::File;

    #[test]
    fn is_dir() {
        let cwd = File::open(".").unwrap();
        assert!(cwd.metadata().unwrap().file_type().is_dir());
    }

    #[test]
    fn is_file() {
        let cargo_toml = File::open("Cargo.toml").unwrap();
        assert!(cargo_toml.metadata().unwrap().file_type().is_file());
    }

    #[test]
    fn is_symlink() {
        let name = "link";
        crate::functions::symlink("Cargo.toml", "link").unwrap();
        let metadata = crate::functions::symlink_metadata(name).unwrap();
        assert!(metadata.file_type().is_symlink());

        crate::functions::remove_file(name).unwrap();
    }

    #[test]
    fn is_block_device() {
        let dev = crate::functions::read_dir("/dev").unwrap();
        let mut opt_device_file = None;
        for res_entry in dev {
            let entry = res_entry.unwrap();
            if entry.metadata().unwrap().file_type().is_block_device() {
                opt_device_file = Some(entry);
                break;
            }
        }

        if let Some(device_file) = opt_device_file {
            let metadata = crate::functions::metadata(device_file.path()).unwrap();
            assert!(metadata.file_type().is_block_device());
        }
    }

    #[test]
    fn is_char_device() {
        let dev = crate::functions::read_dir("/dev").unwrap();
        let mut opt_device_file = None;
        for res_entry in dev {
            let entry = res_entry.unwrap();
            if entry.metadata().unwrap().file_type().is_char_device() {
                opt_device_file = Some(entry);
                break;
            }
        }

        if let Some(device_file) = opt_device_file {
            let metadata = crate::functions::metadata(device_file.path()).unwrap();
            assert!(metadata.file_type().is_char_device());
        }
    }
}
