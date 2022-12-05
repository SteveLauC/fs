pub(crate) fn major(dev: libc::dev_t) -> libc::c_uint {
    let mut major = 0;
    major |= (dev & 0x00000000000fff00) >> 8;
    major |= (dev & 0xfffff00000000000) >> 32;
    major as libc::c_uint
}

pub(crate) fn minor(dev: libc::dev_t) -> libc::c_uint {
    let mut minor = 0;
    minor |= (dev & 0x00000000000000ff) >> 0;
    minor |= (dev & 0x00000ffffff00000) >> 12;
    minor as libc::c_uint
}
