### Funny stuff encoutered while implementing this toy file system stdlib

> Categorized by modules

##### file

1. How to implement `Debug` for `File`

   On UNIX platforms, `File` is basically a wrapper for a file descriptor.
   Obviously, something like the following stuff is not sufficient for debugging
   purposes:

   ```rust
   File { fd: OwnedFd { fd: 4 } }
   ```

   The `Debug` implementation for `std::fs::File` also prints the file path and 
   access mode flags (read & write):

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
   and [retrieve the file access flags on a fd (through the `fcntl(2)` syscall)]
   (https://man7.org/linux/man-pages/man2/fcntl.2.html):

   In the implementation of stdlib, Rust has two helper functions to retrieve
   file path and flag.
   
   To get the file path from a file descriptor, we exploit the symlinks under 
   `/proc/self/fd`:

   ```rust
   #[cfg(any(target_os = "linux", target_os = "netbsd"))]
   fn get_path(fd: c_int) -> Option<PathBuf> {
       let mut p = PathBuf::from("/proc/self/fd");
       p.push(&fd.to_string());
       readlink(&p).ok()
   }
   ```
   
   Call `fcntl(2)` to retrieve file access flag:

   ```rust
   #[cfg(any(target_os = "linux", target_os = "macos", target_os = "vxworks"))]
   fn get_mode(fd: c_int) -> Option<(bool, bool)> {
       let mode = unsafe { libc::fcntl(fd, libc::F_GETFL) };
       if mode == -1 {
           // Error
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

##### open_options

1. What's the usage of 
   [`get_access_mode()`](https://github.com/rust-lang/rust/blob/bddad597feb997a4e5d2cd174a76c3b07a84e4d6/library/std/src/sys/unix/fs.rs#L918-L927)
   and
   [`get_creation_mode()`](https://github.com/rust-lang/rust/blob/bddad597feb997a4e5d2cd174a76c3b07a84e4d6/library/std/src/sys/unix/fs.rs#L929-L951)

   `get_access_mode()` and `get_creation_mode()` are two helper function used 
   to convert the value of an `OpenOptions` struct to the `flags` argument of `open(2)`
   syscall.

   ```rust
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
   ```

   In the implementation of `OpenOptions::open()`, we use `get_access_mode()` and
   `get_creation_mode()` to compose the final `flags` argument:

   ```rust
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

	       // What do these two lines of code mean?
               (false, _, true) => Ok(libc::O_WRONLY | libc::O_APPEND),
               (true, _, true) => Ok(libc::O_RDWR | libc::O_APPEND),

               (false, false, false) => Err(Error::from_raw_os_error(libc::EINVAL)),
           }
        }
   }
   ```

   There are actually 8 (2^3) cases for the value of `(read: bool, write: bool, append: bool)`,
   but when `append` is set to true, `write` is also set to true. This is documented [here]
   (https://doc.rust-lang.org/std/fs/struct.OpenOptions.html#method.append).

   Thus, the value of `write` can be ignored ("_") when `append` is true.

3. Checking of invalid `OpenOptions` value

   Rust restricts the values we can set in an `OpenOptions` struct, this is done
   in function `get_creation_mode()`:

   ```rust
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
   ```

##### functions

1. The implementation of
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

