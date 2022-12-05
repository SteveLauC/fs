//! Stuff that does not belong to `std::fs` or `std::os::unix::fs` but has to be
//! used in our implementation.

/// A struct similar to the [`std::time::SystemTime`]
///
/// [`std::time::SystemTime`]: https://doc.rust-lang.org/std/time/struct.SystemTime.html
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct SystemTime {
    sec: i64,
    nsec: i64,
}

impl SystemTime {
    pub fn new(sec: i64, nsec: i64) -> Self {
        Self { sec, nsec }
    }
}
