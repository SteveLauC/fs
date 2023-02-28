use crate::backend::encapsulation::{mkdir, Mode};
use std::{
    io::{Error, ErrorKind, Result},
    os::unix::fs::DirBuilderExt,
    path::Path,
};

#[derive(Debug)]
pub struct DirBuilder {
    mode: Mode,
    recursive: bool,
}

impl DirBuilder {
    /// Creates a new set of options with default mode/security settings for all
    /// platforms and also non-recursive.
    pub fn new() -> Self {
        Self {
            mode: Mode::from_bits(0o777).unwrap(),
            recursive: false,
        }
    }

    /// Indicates that directories should be created recursively, creating all
    /// parent directories. Parents that do not exist are created with the same
    /// security and permissions settings.
    pub fn recursive(&mut self, recursive: bool) -> &mut Self {
        self.recursive = recursive;
        self
    }

    /// Creates the specified directory with the options configured in this
    /// builder.
    pub fn create<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        if self.recursive {
            self.create_dir_all(path.as_ref())
        } else {
            mkdir(path.as_ref(), self.mode)
        }
    }

    // To create `/some/dir` (assume `/some` does not exist), this impl will:
    // 1. call `mkdir("/some/dir"), which returns `ErrorKind::NotFound`
    // 2. Then call `create_dir_all(parent("/some/dir"))` =>
    //              `create_dir_all("/some") =>
    //              `Ok(()`
    // 3. `/some` has been created, then create `/some/dir`
    fn create_dir_all(&self, path: &Path) -> Result<()> {
        if path == Path::new("") {
            return Ok(());
        }

        match mkdir(path, self.mode) {
            Ok(()) => return Ok(()),
            // There is at least one parent directory that should be created first,
            // execute the next block.
            Err(ref e) if e.kind() == ErrorKind::NotFound => {}
            Err(_) if path.is_dir() => return Ok(()),
            Err(e) => return Err(e),
        }
        match path.parent() {
            // recursive implementation
            Some(p) => self.create_dir_all(p)?,
            None => {
                return Err(Error::new(
                    ErrorKind::Uncategorized,
                    "failed to create whole tree",
                ));
            }
        }
        match mkdir(path, self.mode) {
            Ok(()) => Ok(()),
            Err(_) if path.is_dir() => Ok(()),
            Err(e) => Err(e),
        }
    }
}

impl DirBuilderExt for DirBuilder {
    fn mode(&mut self, mode: u32) -> &mut Self {
        self.mode = Mode::from_bits_truncate(mode);
        self
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::os::{linux::fs::MetadataExt, unix::fs::DirBuilderExt};

    /// Get the umask used by the underlying file system.
    #[inline]
    fn get_umask() -> u32 {
        let umask = nix::sys::stat::umask(nix::sys::stat::Mode::from_bits(0o22).unwrap());
        nix::sys::stat::umask(umask);
        umask.bits()
    }

    #[test]
    fn mode() {
        let mode = 0o666_u32;
        DirBuilder::new().mode(mode).create("dir").unwrap();
        let umask = get_umask();
        let actual_mode = mode & !umask;

        let dir_metadata = crate::functions::metadata("dir").unwrap();
        assert!(dir_metadata.is_dir());
        assert_eq!(dir_metadata.st_mode() & (0o777), actual_mode);

        crate::functions::remove_dir("dir").unwrap();
    }

    #[test]
    fn recursive() {
        DirBuilder::new()
            .recursive(true)
            .create("dir1/dir2/dir3")
            .unwrap();

        crate::functions::remove_dir("dir1/dir2/dir3").unwrap();
        crate::functions::remove_dir("dir1/dir2").unwrap();
        crate::functions::remove_dir("dir1").unwrap();
    }
}
