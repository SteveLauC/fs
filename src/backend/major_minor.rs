pub(crate) fn major(dev: libc::dev_t) -> libc::c_uint {
    let mut major = 0;
    major |= (dev & 0x00000000000fff00) >> 8;
    major |= (dev & 0xfffff00000000000) >> 32;
    major as libc::c_uint
}

pub(crate) fn minor(dev: libc::dev_t) -> libc::c_uint {
    let mut minor = 0;
    minor |= dev & 0x00000000000000ff;
    minor |= (dev & 0x00000ffffff00000) >> 12;
    minor as libc::c_uint
}

pub(crate) fn makedev(major: libc::c_uint, minor: libc::c_uint) -> libc::dev_t {
    let major = major as libc::dev_t;
    let minor = minor as libc::dev_t;
    let mut dev = 0;
    dev |= (major & 0x00000fff) << 8;
    dev |= (major & 0xfffff000) << 32;
    dev |= minor & 0x000000ff;
    dev |= (minor & 0xffffff00) << 12;
    dev
}
