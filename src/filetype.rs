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
