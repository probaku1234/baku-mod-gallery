#[derive(Debug)]
pub struct SetupError(pub String);
impl std::error::Error for SetupError {}
impl std::fmt::Display for SetupError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error: {}", self.0)
    }
}

impl From<anyhow::Error> for SetupError {
    fn from(_: anyhow::Error) -> Self {
        Self { 0: "".to_string() }
    }
}
