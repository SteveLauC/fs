use std::{
    collections::VecDeque,
    env::current_dir,
    ffi::OsString,
    io::Result,
    path::{Path, PathBuf},
};

#[derive(Debug)]
struct Paths {
    pub parsed: PathBuf,
    pub remained: VecDeque<OsString>,
}

impl Paths {
    /// Construct a new [`Paths`] struct
    fn new<P>(parsed: Option<PathBuf>, remained: Option<P>) -> Self
    where
        P: AsRef<Path>,
    {
        let parsed = match parsed {
            Some(p) => p,
            None => PathBuf::new(),
        };
        let remained = match remained {
            Some(r) => r
                .as_ref()
                .components()
                .map(|com| com.as_os_str().to_owned())
                .collect(),
            None => VecDeque::new(),
        };

        Self { parsed, remained }
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
    fn remained_next_entry(&mut self) -> Option<OsString> {
        self.remained.pop_front()
    }

    #[inline]
    fn remained_push_front<P: AsRef<Path>>(&mut self, entry: P) {
        entry
            .as_ref()
            .components()
            .rev()
            .map(|com| com.as_os_str().to_owned())
            .for_each(|item| {
                self.remained.push_front(item);
            });
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
    let mut paths = Paths::new(
        if path.as_ref().is_absolute() {
            None
        } else {
            Some(cwd)
        },
        Some(path),
    );

    while let Some(entry) = paths.remained_next_entry() {
        if is_dot(entry.as_os_str()) {
            continue;
        } else if is_a_pair_of_dots(entry.as_os_str()) {
            paths.parsed_cd_to_parent();
        } else {
            paths.parsed_push_back(entry);
        }

        if paths.parsed.is_symlink() {
            let link_content = paths.parsed.read_link().expect("can not follow symlink");
            if link_content.is_absolute() {
                let clean_link: PathBuf = link_content.components().collect();
                paths.replace_parsed_with(clean_link);
            } else {
                paths.parsed_cd_to_parent();
                paths.remained_push_front(link_content);
            }
        }
    }

    Ok(paths.parsed.clone())
}

#[cfg(test)]
mod test {
    use super::realpath;
    use std::{env::current_dir, path::Path};

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
        let cwd = current_dir().expect("can not get cwd");
        let res3 = realpath("test/..");
        assert_eq!(res3.unwrap(), cwd);
    }

    #[test]
    fn test4() {
        let mut cwd = current_dir().expect("can not get cwd");
        let res4 = realpath("test/path/..");
        cwd.push("test");

        assert_eq!(res4.unwrap(), cwd);
    }

    #[test]
    fn test5() {
        let mut cwd = current_dir().expect("can not get cwd");
        let res5 = realpath("test/../path");
        cwd.push("path");

        assert_eq!(res5.unwrap(), cwd);
    }

    #[test]
    fn test6() {
        let mut cwd = current_dir().expect("can not get cwd");
        let res6 = realpath("test/../path");
        cwd.push("path");

        assert_eq!(res6.unwrap(), cwd);
    }
}
