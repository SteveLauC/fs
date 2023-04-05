use crate::{backend::encapsulation, filetype::FileType, metadata::Metadata};
use std::{
    ffi::OsString,
    fmt,
    fmt::{Debug, Formatter},
    io,
    os::unix::fs::DirEntryExt,
    path::PathBuf,
};

/// Iterator over the entries in a directory.
///
/// This iterator is returned from the read_dir function of this module and will
/// yield instances of `io::Result<DirEntry>`. Through a DirEntry information like
/// the entry’s path and possibly other metadata can be learned.
///
/// The order in which this iterator returns entries is platform and filesystem
/// dependent.
pub struct ReadDir(pub(crate) encapsulation::Dir);

impl Debug for ReadDir {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_tuple("ReadDir").field(&self.0.root()).finish()
    }
}

impl Iterator for ReadDir {
    type Item = io::Result<DirEntry>;

    fn next(&mut self) -> Option<Self::Item> {
        let entry = self.0.readdir()?;

        Some(entry.map(|d_entry| DirEntry(d_entry)))
    }
}

/// Entries returned by the [`ReadDir`] iterator.
///
/// An instance of DirEntry represents an entry inside of a directory on the
/// filesystem. Each entry can be inspected via methods to learn about the
/// full path or possibly other metadata through per-platform extension traits.
pub struct DirEntry(pub(crate) encapsulation::Dirent);

impl DirEntry {
    /// Returns the full path to the file that this entry represents.
    ///
    /// The full path is created by joining the original path to `read_dir` with
    /// the filename of this entry.
    #[inline]
    pub fn path(&self) -> PathBuf {
        self.0.path.clone()
    }

    /// Returns the metadata for the file that this entry points at.
    ///
    /// This function will not traverse symlinks if this entry points at a symlink.
    /// To traverse symlinks use fs::metadata or fs::File::metadata.
    #[inline]
    pub fn metadata(&self) -> io::Result<Metadata> {
        let path = self.0.path.as_path();
        encapsulation::statx(path).map(|statx| Metadata(statx))
    }

    /// Returns the file type for the file that this entry points at.
    ///
    /// This function will not traverse symlinks if this entry points at a symlink.
    #[inline]
    pub fn file_type(&self) -> io::Result<FileType> {
        Ok(FileType(self.0.file_type))
    }

    /// Returns the bare file name of this directory entry without any other
    /// leading path component.
    #[inline]
    pub fn file_name(&self) -> OsString {
        self.0.name.clone()
    }
}

impl Debug for DirEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("DirEntry").field(&self.path()).finish()
    }
}

impl DirEntryExt for DirEntry {
    #[inline]
    fn ino(&self) -> u64 {
        self.0.ino
    }
}

#[cfg(test)]
mod test {
    use tempdir::TempDir;

    #[test]
    fn iterating_over_root() {
        let output = std::process::Command::new("ls")
            .args(["-al", "/"])
            .output()
            .unwrap();
        assert!(output.stderr.is_empty());
        let num_of_file = output.stdout.iter().filter(|&&b| b == b'\n').count() - 1;

        let dir = crate::read_dir("/").unwrap();
        let mut n_files = dir.into_iter().count();
        // add "." and ".."
        n_files += 2;
        assert_eq!(num_of_file, n_files);
    }

    #[test]
    fn empty_dir() {
        let temp_dir = TempDir::new("test_empty_dir").unwrap();
        let temp_dir_path = temp_dir.path();

        let mut dir = crate::functions::read_dir(temp_dir_path).unwrap();
        let none = dir.next();

        assert!(none.is_none());
    }
}
