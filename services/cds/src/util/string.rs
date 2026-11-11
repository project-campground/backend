pub trait CdsStringExtension {
    fn truncate_clone(self, length: usize) -> Self;
}

impl CdsStringExtension for String {
    fn truncate_clone(self, length: usize) -> Self {
        let mut new_str = self.clone();
        new_str.truncate(length);
        new_str
    }
}
