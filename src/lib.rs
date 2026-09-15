#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

mod error;
mod http;

use crate::http::HttpClient;
pub use error::GoogleApiError;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use urlencoding::encode;

/// Entry point of the crate. Use [`GoogleIndexingApi::url_notifications`] to get a
/// [`UrlNotificationsApi`] client for the `urlNotifications` resource of the
/// Google Indexing API.
///
/// # Example
///
/// ```no_run
/// use google_indexing_api::{GoogleIndexingApi, UrlNotificationsType};
///
/// # async fn run(token: &str) -> Result<(), google_indexing_api::GoogleApiError> {
/// let api = GoogleIndexingApi::url_notifications();
/// api.publish(token, "https://example.com/jobs/1", UrlNotificationsType::UPDATED).await?;
/// # Ok(())
/// # }
/// ```
pub struct GoogleIndexingApi {}

impl GoogleIndexingApi {
    /// Returns a client for the `urlNotifications` resource
    /// (`publish`, `getMetadata`, and batch requests).
    ///
    /// The client holds no state and no credentials; every method takes the
    /// OAuth 2.0 access token as an argument.
    pub fn url_notifications() -> UrlNotificationsApi {
        UrlNotificationsApi::default()
    }
}

/// The `type` field of a URL notification: whether the URL was updated or deleted.
///
/// Serializes to the wire values used by the Indexing API
/// (`URL_UPDATED`, `URL_DELETED`, `URL_NOTIFICATION_TYPE_UNSPECIFIED`).
/// `Display` gives the same wire value.
///
/// # Example
///
/// ```
/// use google_indexing_api::UrlNotificationsType;
///
/// assert_eq!(UrlNotificationsType::UPDATED.to_string(), "URL_UPDATED");
/// assert_eq!(UrlNotificationsType::DELETED.to_string(), "URL_DELETED");
/// assert_eq!(
///     serde_json::to_string(&UrlNotificationsType::DELETED).unwrap(),
///     "\"URL_DELETED\""
/// );
/// ```
#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub enum UrlNotificationsType {
    /// `URL_NOTIFICATION_TYPE_UNSPECIFIED`: the default value. It only appears in
    /// responses; do not send it.
    #[serde(rename = "URL_NOTIFICATION_TYPE_UNSPECIFIED")]
    #[default]
    UrlNotificationTypeUnspecified,
    /// `URL_UPDATED`: the URL was added or its content changed.
    #[serde(rename = "URL_UPDATED")]
    UPDATED,
    /// `URL_DELETED`: the URL was removed.
    #[serde(rename = "URL_DELETED")]
    DELETED,
}

impl std::fmt::Display for UrlNotificationsType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            UrlNotificationsType::UPDATED => "URL_UPDATED",
            UrlNotificationsType::DELETED => "URL_DELETED",
            UrlNotificationsType::UrlNotificationTypeUnspecified => {
                "URL_NOTIFICATION_TYPE_UNSPECIFIED"
            }
        };
        f.write_str(value)
    }
}

/// Client for the `urlNotifications` resource of the Google Indexing API.
///
/// Obtain it with [`GoogleIndexingApi::url_notifications`]. It holds no state, so
/// it can be created per request or shared between tasks.
///
/// Reference: <https://developers.google.com/search/apis/indexing-api/v3/reference/indexing/rest/v3/urlNotifications>
#[derive(Default)]
pub struct UrlNotificationsApi {}

impl UrlNotificationsApi {
    /// Notifies Google that a single URL was updated or deleted
    /// (`POST https://indexing.googleapis.com/v3/urlNotifications:publish`).
    ///
    /// * `token` — OAuth 2.0 access token with the
    ///   `https://www.googleapis.com/auth/indexing` scope.
    /// * `url` — the fully qualified URL of the page.
    /// * `url_type` — [`UrlNotificationsType::UPDATED`] or [`UrlNotificationsType::DELETED`].
    ///
    /// Returns the raw JSON response from Google (an `urlNotificationMetadata` object).
    /// Each call counts toward the daily publish quota.
    ///
    /// # Errors
    ///
    /// Returns [`GoogleApiError::HttpStatus`] when Google answers with a non-success
    /// status (for example `403` when the service account does not own the
    /// Search Console property, or `429` when the quota is exhausted),
    /// [`GoogleApiError::Connection`] when the request cannot be sent, and
    /// [`GoogleApiError::JsonParse`] when the response body is not JSON.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use google_indexing_api::{GoogleIndexingApi, UrlNotificationsType};
    ///
    /// # async fn run(token: &str) -> Result<(), google_indexing_api::GoogleApiError> {
    /// let api = GoogleIndexingApi::url_notifications();
    ///
    /// // A page was added or changed
    /// let response = api
    ///     .publish(token, "https://example.com/jobs/1", UrlNotificationsType::UPDATED)
    ///     .await?;
    /// println!("{response}");
    ///
    /// // A page was removed
    /// api.publish(token, "https://example.com/jobs/2", UrlNotificationsType::DELETED)
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn publish(
        &self,
        token: &str,
        url: &str,
        url_type: UrlNotificationsType,
    ) -> Result<Value, GoogleApiError> {
        HttpClient::post(
            token,
            "https://indexing.googleapis.com/v3/urlNotifications:publish",
            json!({
                "url": url,
                "type": url_type.to_string(),
            }),
        )
        .await
    }

    /// Fetches the latest notifications Google received for a URL through the
    /// Indexing API (`GET https://indexing.googleapis.com/v3/urlNotifications/metadata`).
    ///
    /// This does **not** report whether the URL is indexed by Google; use the
    /// Search Console URL Inspection API for that.
    ///
    /// # Errors
    ///
    /// Same as [`UrlNotificationsApi::publish`]. Google answers `404` (returned as
    /// [`GoogleApiError::HttpStatus`]) when no notification was ever sent for the URL.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use google_indexing_api::GoogleIndexingApi;
    ///
    /// # async fn run(token: &str) -> Result<(), google_indexing_api::GoogleApiError> {
    /// let metadata = GoogleIndexingApi::url_notifications()
    ///     .get_metadata(token, "https://example.com/jobs/1")
    ///     .await?;
    ///
    /// if let Some(update) = metadata.latest_update {
    ///     println!("last URL_UPDATED notification: {}", update.notify_time);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_metadata(
        &self,
        token: &str,
        url: &str,
    ) -> Result<ResponseUrlNotificationMetadata, GoogleApiError> {
        HttpClient::get(
            token,
            format!(
                r#"https://indexing.googleapis.com/v3/urlNotifications/metadata?url={}"#,
                encode(url),
            )
            .as_str(),
        )
        .await
    }

    /// Sends one `publish` notification per URL in a single multipart batch request
    /// (`POST https://indexing.googleapis.com/batch`).
    ///
    /// Accepts 1 to 100 URLs, which is Google's limit per batch. Every URL still
    /// counts toward the daily publish quota; batching only reduces round trips.
    ///
    /// The result contains one [`ResponseGoogleIndexingBatch`] per input URL, in
    /// input order, each carrying the HTTP status and body of its own sub-response.
    /// A batch call succeeds as a whole even if some sub-responses report an error,
    /// so check [`ResponseGoogleIndexingBatch::status_code`] for each entry.
    ///
    /// # Errors
    ///
    /// Returns [`GoogleApiError::InvalidArgument`] when `urls` is empty or has more
    /// than 100 entries, [`GoogleApiError::JsonParse`] when the multipart response
    /// cannot be parsed, and otherwise the same errors as
    /// [`UrlNotificationsApi::publish`].
    ///
    /// # Example
    ///
    /// ```no_run
    /// use google_indexing_api::{GoogleIndexingApi, UrlNotificationsType};
    ///
    /// # async fn run(token: &str) -> Result<(), google_indexing_api::GoogleApiError> {
    /// let urls = vec![
    ///     "https://example.com/jobs/1".to_string(),
    ///     "https://example.com/jobs/2".to_string(),
    /// ];
    /// let results = GoogleIndexingApi::url_notifications()
    ///     .batch(token, urls, UrlNotificationsType::UPDATED)
    ///     .await?;
    ///
    /// for result in results {
    ///     if result.status_code() != 200 {
    ///         eprintln!("{} failed: {}", result.url(), result.value());
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn batch(
        &self,
        token: &str,
        urls: Vec<String>,
        url_type: UrlNotificationsType,
    ) -> Result<Vec<ResponseGoogleIndexingBatch>, GoogleApiError> {
        HttpClient::execute(token, urls, url_type).await
    }
}

/// Response of [`UrlNotificationsApi::get_metadata`]: the latest `URL_UPDATED` and
/// `URL_DELETED` notifications Google received for a URL.
///
/// # Example
///
/// The JSON Google returns, and how it maps onto this struct:
///
/// ```
/// use google_indexing_api::{ResponseUrlNotificationMetadata, UrlNotificationsType};
///
/// let body = r#"{
///   "url": "https://example.com/jobs/1",
///   "latestUpdate": {
///     "url": "https://example.com/jobs/1",
///     "type": "URL_UPDATED",
///     "notifyTime": "2024-01-31T12:34:56.789Z"
///   }
/// }"#;
///
/// let metadata: ResponseUrlNotificationMetadata = serde_json::from_str(body).unwrap();
/// let update = metadata.latest_update.unwrap();
/// assert!(matches!(update.url_type, UrlNotificationsType::UPDATED));
/// assert_eq!(update.notify_time, "2024-01-31T12:34:56.789Z");
/// assert!(metadata.latest_remove.is_none());
/// ```
#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct ResponseUrlNotificationMetadata {
    /// The URL the metadata refers to.
    pub url: String,
    /// The latest `URL_UPDATED` notification, if any was ever sent.
    #[serde(rename = "latestUpdate")]
    pub latest_update: Option<ResponseUrlNotification>,
    /// The latest `URL_DELETED` notification, if any was ever sent.
    #[serde(rename = "latestRemove")]
    pub latest_remove: Option<ResponseUrlNotification>,
}

/// A single notification recorded by Google (an `UrlNotification` object).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ResponseUrlNotification {
    /// The notified URL.
    pub url: String,
    /// Whether the notification was an update or a deletion.
    #[serde(rename = "type")]
    pub url_type: UrlNotificationsType,
    /// When Google received the notification, as an RFC 3339 timestamp
    /// (for example `2024-01-31T12:34:56.789Z`).
    #[serde(rename = "notifyTime")]
    pub notify_time: String,
}

/// One entry of a batch response: the sub-response for a single URL.
///
/// Returned by [`UrlNotificationsApi::batch`], one per input URL and in input order.
/// Instances are only created by this crate; read them through the accessor methods.
///
/// # Example
///
/// ```no_run
/// use google_indexing_api::{GoogleIndexingApi, UrlNotificationsType};
///
/// # async fn run(token: &str) -> Result<(), google_indexing_api::GoogleApiError> {
/// let results = GoogleIndexingApi::url_notifications()
///     .batch(token, vec!["https://example.com/jobs/1".to_string()], UrlNotificationsType::UPDATED)
///     .await?;
///
/// let first = &results[0];
/// assert_eq!(first.url(), "https://example.com/jobs/1");
/// if first.status_code() == 200 {
///     // The body of a successful sub-response is an urlNotificationMetadata object
///     println!("{}", first.json()["urlNotificationMetadata"]["url"]);
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Default)]
pub struct ResponseGoogleIndexingBatch {
    url: String,
    status_code: u16,
    value: String,
}

impl ResponseGoogleIndexingBatch {
    /// The URL this entry belongs to.
    pub fn url(&self) -> &str {
        self.url.as_str()
    }
    /// HTTP status code of the sub-response (`200` on success).
    pub fn status_code(&self) -> u16 {
        self.status_code
    }
    /// Raw body of the sub-response as text.
    pub fn value(&self) -> &str {
        self.value.as_str()
    }
    /// Body of the sub-response parsed as JSON, or `Value::Null` if it is not JSON.
    pub fn json(&self) -> Value {
        serde_json::from_str(self.value.as_str()).unwrap_or_default()
    }
}
