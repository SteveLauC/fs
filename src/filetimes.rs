use crate::non_fs::SystemTime;

#[derive(Copy, Clone, Debug, Default)]
pub struct FileTimes(pub(crate) [SystemTime; 2]);

impl FileTimes {
    pub fn new() -> Self {
        FileTimes::default()
    }

    /// Set the last access time of a file.
    pub fn set_accessed(mut self, t: SystemTime) -> Self {
        self.0[0] = t;
        self
    }

    /// Set the last modified time of a file.
    pub fn set_modified(mut self, t: SystemTime) -> Self {
        self.0[1] = t;
        self
    }
}
