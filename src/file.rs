use std::os::unix::io::OwnedFd;

#[derive(Debug)]
pub struct File {
    pub(crate) fd: OwnedFd,
}

impl File {
}
