use crate::backend::encapsulation::{self, Mode};
use std::os::unix::fs::PermissionsExt;

/// Representation of the various permissions on a file.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Permissions(pub(crate) encapsulation::Mode);

impl Permissions {
    pub fn readonly(&self) -> bool {
        // check if any class (owner, group, others) has write permission
        self.0.bits() & 0o222 == 0
    }

    pub fn set_readonly(&mut self, readonly: bool) {
        if readonly {
            // remove write permission for all classes; equivalent to `chmod a-w <file>`
            self.0 &= !unsafe { Mode::from_bits_unchecked(0o222) };
        } else {
            // add write permission for all classes; equivalent to `chmod a+w <file>`
            self.0 |= unsafe { Mode::from_bits_unchecked(0o222) };
        }
    }
}

impl PermissionsExt for Permissions {
    #[inline]
    fn mode(&self) -> u32 {
        self.0.bits()
    }

    #[inline]
    fn set_mode(&mut self, mode: u32) {
        self.0 = unsafe { Mode::from_bits_unchecked(mode) };
    }

    #[inline]
    fn from_mode(mode: u32) -> Self {
        Permissions(unsafe { Mode::from_bits_unchecked(mode) })
    }
}
