use crate::backend::encapsulation;

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
