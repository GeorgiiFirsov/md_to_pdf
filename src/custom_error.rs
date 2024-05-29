use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct CustomError {
    message: String,
    cause: Option<Box<dyn std::error::Error>>,
}

impl CustomError {
    pub fn with_cause(message: &str, cause: Box<dyn std::error::Error>) -> CustomError {
        CustomError {
            message: message.to_string(),
            cause: Some(cause),
        }
    }
}

impl fmt::Display for CustomError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CustomError: {}", self.message)?;
        if let Some(ref cause) = self.cause {
            write!(f, "; caused by: {}", cause)?;
        }
        Ok(())
    }
}

impl Error for CustomError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.cause.as_ref().map(|e| e.as_ref())
    }
}

pub fn custom_err_with_cause(str: &str, cause: Box<dyn std::error::Error>) -> Box<dyn std::error::Error> {
    Box::new(CustomError::with_cause(str, cause))
}
