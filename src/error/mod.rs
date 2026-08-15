use std::fmt::{Debug, Display, Formatter};

/// Error type returned by all API calls.
pub enum GoogleApiError {
    /// Failed to connect or send the request.
    Connection(String),
    /// The response could not be parsed (invalid JSON or a malformed batch response).
    JsonParse(String),
    /// The API returned an error status. Holds the HTTP status code and the response body.
    HttpStatus(u16, String),
    /// The request input was rejected before sending (e.g. batch size out of range).
    InvalidArgument(String),
}

impl Display for GoogleApiError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            GoogleApiError::Connection(e) => write!(f, "connection error: {}", e),
            GoogleApiError::JsonParse(e) => write!(f, "json parse error: {}", e),
            GoogleApiError::HttpStatus(status, body) => {
                write!(f, "http status {}: {}", status, body)
            }
            GoogleApiError::InvalidArgument(e) => write!(f, "invalid argument: {}", e),
        }
    }
}

impl Debug for GoogleApiError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

impl std::error::Error for GoogleApiError {}
