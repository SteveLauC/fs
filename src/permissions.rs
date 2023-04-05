use crate::backend::encapsulation::{self, Mode};
use std::os::unix::fs::PermissionsExt;

/// Representation of the various permissions on a file.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Permissions(pub(crate) encapsulation::Mode);

impl Permissions {
    /// Returns true if these permissions describe a readonly (unwritable) file.
    ///
    /// # Note
    /// This function does not take Access Control Lists (ACLs) or Unix group membership into account.
    pub fn readonly(&self) -> bool {
        // check if any class (owner, group, others) has write permission
        self.0.bits() & 0o222 == 0
    }

    /// Modifies the readonly flag for this set of permissions. If the readonly
    /// argument is true, using the resulting Permission will update file permissions
    /// to forbid writing. Conversely, if it’s false, using the resulting Permission
    /// will update file permissions to allow writing.
    ///
    /// This operation does not modify the files attributes. This only changes the
    /// in-memory value of these attributes for this Permissions instance. To
    /// modify the files attributes use the `set_permissions` function which commits
    /// these attribute changes to the file.
    ///
    /// # Note
    /// `set_readonly(false)` makes the file world-writable on Unix. You can use the
    /// PermissionsExt trait on Unix to avoid this issue.
    ///
    /// It also does not take Access Control Lists (ACLs) or Unix group membership into account.
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
