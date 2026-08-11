pub trait AppviewStringExtension {
    fn truncate_clone(self, length: usize) -> Self;
}

impl AppviewStringExtension for String {
    fn truncate_clone(self, length: usize) -> Self {
        let mut new_str = self.clone();
        new_str.truncate(length);
        new_str
    }
}
