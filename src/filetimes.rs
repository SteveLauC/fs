use std::time::SystemTime;

pub struct FileTimes;

impl FileTimes {
    pub fn new() -> Self {
        FileTimes
    }

    /// Set the last access time of a file.
    pub fn set_accessed(mut self, t: SystemTime) -> Self {
        unimplemented!()
    }

    /// Set the last modified time of a file.
    pub fn set_modified(mut self, t: SystemTime) -> Self {
        unimplemented!()
    }
}
