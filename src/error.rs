use std::fmt::Formatter;

#[derive(Debug)]
pub struct InvalidS3PathComponent {
    pub component: String,
    pub reason: String,
}

impl std::fmt::Display for InvalidS3PathComponent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Invalid S3 path component '{}': {}", self.component, self.reason)
    }
}

impl std::error::Error for InvalidS3PathComponent {}
