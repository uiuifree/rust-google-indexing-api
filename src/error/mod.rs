use std::fmt::{Debug, Display, Formatter};

/// Error type returned by all API calls.
///
/// It implements [`std::error::Error`] and [`Display`], so it works with `?` and
/// `Box<dyn std::error::Error>`. The variants tell you where the call failed:
/// before sending ([`InvalidArgument`](Self::InvalidArgument)), while sending
/// ([`Connection`](Self::Connection)), in Google's answer
/// ([`HttpStatus`](Self::HttpStatus)), or while reading the answer
/// ([`JsonParse`](Self::JsonParse)).
///
/// # Example
///
/// ```
/// use google_indexing_api::GoogleApiError;
///
/// let error = GoogleApiError::HttpStatus(403, r#"{"error":{"status":"PERMISSION_DENIED"}}"#.into());
/// assert_eq!(
///     error.to_string(),
///     r#"http status 403: {"error":{"status":"PERMISSION_DENIED"}}"#
/// );
///
/// // Usable as a boxed error
/// let boxed: Box<dyn std::error::Error> = Box::new(error);
/// assert!(boxed.to_string().starts_with("http status 403"));
/// ```
///
/// Matching on the variants:
///
/// ```no_run
/// use google_indexing_api::{GoogleApiError, GoogleIndexingApi, UrlNotificationsType};
///
/// # async fn run(token: &str) {
/// let result = GoogleIndexingApi::url_notifications()
///     .publish(token, "https://example.com/jobs/1", UrlNotificationsType::UPDATED)
///     .await;
///
/// match result {
///     Ok(response) => println!("accepted: {response}"),
///     Err(GoogleApiError::HttpStatus(429, _)) => eprintln!("daily quota exhausted"),
///     Err(GoogleApiError::HttpStatus(status, body)) => eprintln!("{status}: {body}"),
///     Err(GoogleApiError::Connection(e)) => eprintln!("could not reach Google: {e}"),
///     Err(GoogleApiError::JsonParse(e)) => eprintln!("unexpected response: {e}"),
///     Err(GoogleApiError::InvalidArgument(e)) => eprintln!("bad input: {e}"),
/// }
/// # }
/// ```
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
