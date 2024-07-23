use std::fmt::Display;

#[derive(Debug)]
pub enum HttpError {
    ParseMethodError,
    ParseProtocolError,
    InvalidRequestFormat,
    IOError(std::io::Error),
    UTF8Error(std::str::Utf8Error),
}

impl Display for HttpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HttpError::ParseMethodError => write!(f, "Failed to parse method."),
            HttpError::ParseProtocolError => write!(f, "Failed to parse HTTP protocol"),
            HttpError::InvalidRequestFormat => write!(f, "Invalid request format"),
            HttpError::IOError(e) => write!(f, "Error: {}", e),
            HttpError::UTF8Error(e) => write!(f, "Error: {}", e),
        }
    }
}

impl From<std::str::Utf8Error> for HttpError {
    fn from(value: std::str::Utf8Error) -> Self {
        HttpError::UTF8Error(value)
    }
}

impl From<std::io::Error> for HttpError {
    fn from(value: std::io::Error) -> Self {
        HttpError::IOError(value)
    }
}
