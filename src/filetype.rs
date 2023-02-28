use crate::backend::encapsulation;
use std::os::unix::fs::FileTypeExt;

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct FileType(pub(crate) encapsulation::FileType);

impl FileType {
    #[inline]
    pub fn is_dir(&self) -> bool {
        self.0 == encapsulation::FileType::Directory
    }

    #[inline]
    pub fn is_file(&self) -> bool {
        self.0 == encapsulation::FileType::RegularFile
    }

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
