1. How to implement `Debug` for `File`

   On UNIX platforms, `File` is basically a wrapper for a file descriptor.
   Obviously, something like the following stuff is not sufficient for debugging
   purposes:

   ```rust
   File { fd: OwnedFd { fd: 4 } }
   ```

   The `Debug` for `std::fs::File` implementation also prints the Path and Mode
   (read & write):

   ```rust
   fn main() {
       let cwd = std::fs::File::open(".").unwrap();
   
       println!("{:?}", cwd);
   }
   ```
   ```shell
   $ cargo r -q
   File { fd: 3, path: "/home/steve/Documents/workspace/rust", read: true, write: false }
   ```

   So the questions become how to
   [get the PATH from a fd](https://stackoverflow.com/q/1188757/14092446)
   and how to
   [retrieve the file access flags on a fd]():

   ```rust
   // Impl of `Debug` for `std::fs::File`

   let fd = self.as_raw_fd();
   let mut b = f.debug_struct("File");
   b.field("fd", &fd);
   if let Some(path) = get_path(fd) {
       b.field("path", &path);
   }
   if let Some((read, write)) = get_mode(fd) {
       b.field("read", &read).field("write", &write);
   }
   b.finish()
   ```

   ```rust
   // Two helper functions employed by the Debug impl

   #[cfg(any(target_os = "linux", target_os = "netbsd"))]
   fn get_path(fd: c_int) -> Option<PathBuf> {
       let mut p = PathBuf::from("/proc/self/fd");
       p.push(&fd.to_string());
       readlink(&p).ok()
   }
   
   #[cfg(any(target_os = "linux", target_os = "macos", target_os = "vxworks"))]
   fn get_mode(fd: c_int) -> Option<(bool, bool)> {
       let mode = unsafe { libc::fcntl(fd, libc::F_GETFL) };
       if mode == -1 {
	   return None;
       }
       match mode & libc::O_ACCMODE {
	   libc::O_RDONLY => Some((true, false)),
	   libc::O_RDWR => Some((true, true)),
	   libc::O_WRONLY => Some((false, true)),
	   _ => None,
       }
   }
   ```

2. What does `(false, _, true) => Ok(libc::O_WRONLY | libc::O_APPEND)`
   and `(true, _, true) => Ok(libc::O_RDWR | libc::O_APPEND)` mean in
   [`get_access_mode()`](https://github.com/rust-lang/rust/blob/bddad597feb997a4e5d2cd174a76c3b07a84e4d6/library/std/src/sys/unix/fs.rs#L918-L927)
   ?

   ```rust
   impl OpenOptions {
 	fn get_access_mode(&self) -> io::Result<c_int> {
            match (self.read, self.write, self.append) {
               (true, false, false) => Ok(libc::O_RDONLY),
               (false, true, false) => Ok(libc::O_WRONLY),
               (true, true, false) => Ok(libc::O_RDWR),
               (false, _, true) => Ok(libc::O_WRONLY | libc::O_APPEND),
               (true, _, true) => Ok(libc::O_RDWR | libc::O_APPEND),
               (false, false, false) => Err(Error::from_raw_os_error(libc::EINVAL)),
           }
        }
   }
   ```

   There are actually 8 (2^3) cases of the value of `(read, write, append)`, but
   when `append` is set to true, `write` is also set to true as
   [`.append(true)` is equivalent to `.write(true).append(true)`](https://doc.rust-lang.org/std/fs/struct.OpenOptions.html#method.append)
   .
   Thus, the value of `write` can be ignored when `append` is true.

3. The implementation of
   [`create_dir_all()`](https://github.com/rust-lang/rust/blob/b9341bfdb1dec09b49b1e7d01d7c4db0e2436737/library/std/src/fs.rs#L2455)

   ```rust
   fn create_dir_all(&self, path: &Path) -> io::Result<()> {
        if path == Path::new("") {
            return Ok(());
        }

        match self.inner.mkdir(path) {
            Ok(()) => return Ok(()),
            Err(ref e) if e.kind() == io::ErrorKind::NotFound => {}
            Err(_) if path.is_dir() => return Ok(()),
            Err(e) => return Err(e),
        }
        match path.parent() {
            Some(p) => self.create_dir_all(p)?,
            None => {
                return Err(io::const_io_error!(
                    io::ErrorKind::Uncategorized,
                    "failed to create whole tree",
                ));
            }
        }
        match self.inner.mkdir(path) {
            Ok(()) => Ok(()),
            Err(_) if path.is_dir() => Ok(()),
            Err(e) => Err(e),
        }
   }
   ```
   
   The

