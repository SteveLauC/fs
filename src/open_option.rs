use crate::{
    backend::encapsulation::{open, Flags, Mode},
    file::File,
};
use libc::mode_t;
use std::{io::Result, ops::BitOr, os::unix::fs::OpenOptionsExt, path::Path};

#[derive(Default, Debug)]
pub struct OpenOptions {
    // generic
    read: bool,
    write: bool,
    append: bool,
    truncate: bool,
    create: bool,
    create_new: bool,
    // system-specific
    custom_flags: i32,
    mode: mode_t,
}

impl OpenOptions {
    pub fn new() -> Self {
        OpenOptions {
            // generic
            read: false,
            write: false,
            append: false,
            truncate: false,
            create: false,
            create_new: false,
            // system-specific
            custom_flags: 0,
            mode: 0o666,
        }
    }

    pub fn read(&mut self, read: bool) -> &mut Self {
        self.read = read;
        self
    }
    pub fn write(&mut self, write: bool) -> &mut Self {
        self.write = write;
        self
    }
    pub fn append(&mut self, append: bool) -> &mut Self {
        self.append = append;
        self
    }
    pub fn truncate(&mut self, truncate: bool) -> &mut Self {
        self.truncate = truncate;
        self
    }
    pub fn create(&mut self, create: bool) -> &mut Self {
        self.create = create;
        self
    }
    pub fn create_new(&mut self, create_new: bool) -> &mut Self {
        self.create_new = create_new;
        self
    }

    pub fn custom_flags(&mut self, flags: i32) -> &mut Self {
        self.custom_flags = flags;
        self
    }
    pub fn mode(&mut self, mode: u32) -> &mut Self {
        self.mode = mode as mode_t;
        self
    }

    pub fn open<P: AsRef<Path>>(&self, path: P) -> Result<File> {
        let mut flag = Flags::empty();

        if self.read && self.write {
            flag.insert(Flags::O_RDWR);
        } else if self.read {
            flag.insert(Flags::O_RDONLY);
        } else if self.write {
            flag.insert(Flags::O_WRONLY);
        }

        if self.append {
            flag.insert(Flags::O_APPEND);
        }

        if self.truncate {
            flag.insert(Flags::O_TRUNC);
        }

        if self.create {
            flag.insert(Flags::O_CREAT);
        }

        if self.create_new {
            flag.insert(Flags::O_CREAT);
            flag.insert(Flags::O_EXCL);
        }

        flag = flag.bitor(Flags::from_bits_truncate(self.custom_flags));

        let fd = open(path, flag, Mode::from_bits_truncate(self.mode))?;

        Ok(File { fd })
    }
}

impl OpenOptionsExt for OpenOptions {
    fn mode(&mut self, mode: u32) -> &mut Self {
        self.mode = mode;
        self
    }
    fn custom_flags(&mut self, flags: i32) -> &mut Self {
        self.custom_flags = flags;
        self
    }
}
