use std::os::unix::io::OwnedFd;

pub struct File {
    pub(crate) fd: OwnedFd,
}
