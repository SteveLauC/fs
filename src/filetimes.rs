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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn set_accessed() {
        let mut file_times = FileTimes::new();
        file_times = file_times.set_accessed(SystemTime::new(1, 2));
        assert_eq!(file_times.0[0].sec, 1);
        assert_eq!(file_times.0[0].nsec, 2);
    }

    #[test]
    fn set_modified() {
        let mut file_times = FileTimes::new();
        file_times = file_times.set_modified(SystemTime::new(1, 2));
        assert_eq!(file_times.0[1].sec, 1);
        assert_eq!(file_times.0[1].nsec, 2);
    }
}
