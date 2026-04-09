#[derive(Debug, thiserror::Error)]
pub enum ApplicationError {
    #[error("Execution error: {0}")]
    Execution(String),
    #[error("Verification error: {0}")]
    Verification(String),
    #[error("State error: {0}")]
    State(String),
}

#[cfg(test)]
mod tests {
    use super::ApplicationError;

    #[test]
    fn test_application_error_display() {
        let err = ApplicationError::Verification("fail".to_string());
        assert!(err.to_string().contains("fail"));
    }
}
