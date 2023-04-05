use std::{
    collections::VecDeque,
    env::current_dir,
    ffi::OsString,
    io::{Error, ErrorKind, Result},
    path::{Path, PathBuf},
};

/// `realpath(3)` parser.
#[derive(Debug)]
struct RealpathParser {
    parsed: PathBuf,
    remaining: VecDeque<OsString>,
}

impl RealpathParser {
    /// Construct a new [`Paths`] struct
    fn new<P>(parsed: Option<PathBuf>, remaining: Option<P>) -> Self
    where
        P: AsRef<Path>,
    {
        let parsed = match parsed {
            Some(p) => p,
            None => PathBuf::new(),
        };
        let remaining = match remaining {
            Some(r) => r
                .as_ref()
                .components() // this will normailize it
                .map(|com| com.as_os_str().to_owned())
                .collect(),
            None => VecDeque::new(),
        };

        Self { parsed, remaining }
    }

    #[inline]
    /// Replaces `self.parsed` with `new_parsed`
    fn replace_parsed_with(&mut self, new_parsed: PathBuf) {
        self.parsed = new_parsed;
    }

    #[inline]
    fn parsed_push_back<P: AsRef<Path>>(&mut self, entry: P) {
        self.parsed.push(entry);
    }

    #[inline]
    fn parsed_cd_to_parent(&mut self) {
        if let Some(parent) = self.parsed.parent() {
            let parent_len = parent.as_os_str().len();

            // Note that to avoid allocating memory, we directly alter the `length`
            // field of `self.parsed (PathBuf)`.
            assert!(parent_len <= self.parsed.capacity());
            let mut p_to_heap_memory = &mut self.parsed as *mut PathBuf as *mut usize;
            unsafe {
                p_to_heap_memory = p_to_heap_memory.add(2);
                *p_to_heap_memory = parent_len;
            }
            assert!(self.parsed.as_os_str().len() == parent_len);
        }
    }

    #[inline]
    fn remaining_next_entry(&mut self) -> Option<OsString> {
        self.remaining.pop_front()
    }
}

#[inline]
fn is_dot<P: AsRef<Path>>(path: P) -> bool {
    path.as_ref() == Path::new(".")
}

#[inline]
fn is_a_pair_of_dots<P: AsRef<Path>>(path: P) -> bool {
    path.as_ref() == Path::new("..")
}

/// return the canonicalized absolute pathname
pub(crate) fn realpath<P: AsRef<Path>>(path: P) -> Result<PathBuf> {
    let cwd = current_dir().expect("can not get cwd");
    let mut parser = RealpathParser::new(
        if path.as_ref().is_absolute() {
            Some(PathBuf::from("/"))
        } else {
            Some(cwd)
        },
        Some(path),
    );

    while let Some(entry) = parser.remaining_next_entry() {
        // Check the `parsed` part exists before we proceed
        if parser.parsed.try_exists()? == false {
            return Err(Error::new(ErrorKind::NotFound, "No such file or directory"));
        }

        if is_dot(entry.as_os_str()) {
            continue;
        } else if is_a_pair_of_dots(entry.as_os_str()) {
            parser.parsed_cd_to_parent();
        } else {
            parser.parsed_push_back(entry);
        }

        if parser.parsed.is_symlink() {
            let mut link_content = parser.parsed.read_link().expect("can not follow symlink");
            if link_content.is_relative() {
                // A relative symlink is relative to the parent directory of that link.
                link_content = parser
                    .parsed
                    .parent()
                    .expect("must have a parent")
                    .join(link_content);
            }
            // A symlink can be literally anything, should also be parsed.
            parser.replace_parsed_with(realpath(link_content)?);
        }
    }

    Ok(parser.parsed.clone())
}

#[cfg(test)]
mod test {
    use super::realpath;
    use std::{
        env::current_dir,
        fs::{create_dir, create_dir_all, remove_dir, remove_dir_all, remove_file, File},
        os::unix::fs::symlink,
        path::Path,
    };

    #[test]
    fn test1() {
        let res1 = realpath("/..");
        assert_eq!(res1.unwrap(), Path::new("/"));
    }

    #[test]
    fn test2() {
        let res2 = realpath("/../test");
        assert_eq!(res2.unwrap(), Path::new("/test"));
    }

    #[test]
    fn test3() {
        create_dir("test3").unwrap();
        let cwd = current_dir().expect("can not get cwd");
        let res3 = realpath("test3/..");
        assert_eq!(res3.unwrap(), cwd);

        remove_dir("test3").unwrap();
    }

    #[test]
    fn test4() {
        create_dir_all("test4/path").unwrap();
        let mut cwd = current_dir().expect("can not get cwd");
        let res4 = realpath("test4/path/..");
        cwd.push("test4");

        assert_eq!(res4.unwrap(), cwd);
        remove_dir_all("test4").unwrap();
    }

    #[test]
    fn test5() {
        create_dir("test5").unwrap();
        create_dir("path").unwrap();
        let mut cwd = current_dir().expect("can not get cwd");
        let res5 = realpath("test5/../path");
        cwd.push("path");

        assert_eq!(res5.unwrap(), cwd);

        remove_dir("test5").unwrap();
        remove_dir("path").unwrap();
    }

    #[test]
    fn test_symlink() {
        File::create("source").unwrap();
        symlink("source", "link").unwrap();

        let mut cwd = current_dir().expect("can not get cwd");
        cwd.push("source");
        let parsed = realpath("link").unwrap();

        assert_eq!(parsed, cwd);

        remove_file("source").unwrap();
        remove_file("link").unwrap();
    }
}
