//! # GoogleIndexingAPI
//!
//! This is a general-purpose library for using the GoogleIndex API.
//!
//! Since it defines the structures needed for the API, it can simplify API access.
//!
//! ※ Token generation is not supported in this API.
//!
//! ## Batch API
//! With the batch API, you can make multiple publish requests in bulk. When processing a large number of job listings consecutively, batch requests are processed faster.
//! ```rust
//! use google_indexing_api::{GoogleIndexingApi, ResponseGoogleIndexingBatch, UrlNotificationsType};
//! async fn example_batch(token: &str)   {
//!     GoogleIndexingApi::url_notifications()
//!         .batch(
//!             token,
//!             vec![
//!                 "http://!example.com/widgets/1".to_string(),
//!                 "http://!example.com/widgets/2".to_string(),
//!             ],
//!             UrlNotificationsType::UPDATED,
//!         )
//!         .await;
//! }
//! ```
//!
//! ## Metadata API
//! The metadata API allows you to check the status of a URL notification.
//! ```rust
//! use google_indexing_api::{GoogleIndexingApi, ResponseUrlNotificationMetadata};
//!  async fn example_metadata(token: &str) {
//!     GoogleIndexingApi::url_notifications()
//!         .get_metadata(
//!             token,
//!             "http://!example.com/widgets/1",
//!         )
//!         .await;
//! }
//! ```
mod error;
mod http;

use crate::http::HttpClient;
use crate::GoogleApiError;
pub use error::*;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use urlencoding::encode;

/// API Access Endpoint
pub struct GoogleIndexingApi {}

impl GoogleIndexingApi {
    pub fn url_notifications() -> UrlNotificationsApi {
        UrlNotificationsApi::default()
    }
}

/// URL Notification Type
/// UPDATE or DELETE
#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub enum UrlNotificationsType {
    #[serde(rename = "URL_NOTIFICATION_TYPE_UNSPECIFIED")]
    #[default]
    UrlNotificationTypeUnspecified,
    #[serde(rename = "URL_UPDATED")]
    UPDATED,
    #[serde(rename = "URL_DELETED")]
    DELETED,
}

impl ToString for UrlNotificationsType {
    fn to_string(&self) -> String {
        match self {
            UrlNotificationsType::UPDATED => "URL_UPDATED".to_string(),
            UrlNotificationsType::DELETED => "URL_DELETED".to_string(),
            UrlNotificationsType::UrlNotificationTypeUnspecified => {
                "URL_NOTIFICATION_TYPE_UNSPECIFIED".to_string()
            }
        }
    }
}

// https://developers.go
// ogle.com/search/apis/indexing-api/v3/reference/indexing/rest/v3/urlNotifications?hl=ja
/// urlNotifications API
#[derive(Default)]
pub struct UrlNotificationsApi {}

impl UrlNotificationsApi {
    pub async fn publish(
        &self,
        token: &str,
        url: &str,
        url_type: UrlNotificationsType,
    ) -> Result<Value, GoogleApiError> {
        Ok(HttpClient::post(
            token,
            format!(r#"https://indexing.googleapis.com/v3/urlNotifications:publish"#,).as_str(),
            json!({
                "url": url,
                "type": url_type.to_string(),
            }),
        )
        .await?)
    }
    pub async fn get_metadata(
        &self,
        token: &str,
        url: &str,
    ) -> Result<ResponseUrlNotificationMetadata, GoogleApiError> {
        Ok(HttpClient::get(
            token,
            format!(
                r#"https://indexing.googleapis.com/v3/urlNotifications/metadata?url={}"#,
                encode(url),
            )
            .as_str(),
        )
        .await?)
    }
    pub async fn batch(
        &self,
        token: &str,
        urls: Vec<String>,
        url_type: UrlNotificationsType,
    ) -> Result<Vec<ResponseGoogleIndexingBatch>, GoogleApiError> {
        Ok(HttpClient::execute(token, urls, url_type).await?)
    }
}

/// Response Url Notification Metadata
#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct ResponseUrlNotificationMetadata {
    pub url: String,
    #[serde(rename = "latestUpdate")]
    pub latest_update: Option<ResponseUrlNotification>,
    #[serde(rename = "latestRemove")]
    pub latest_remove: Option<ResponseUrlNotification>,
}

/// Response Url Notification
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ResponseUrlNotification {
    pub url: String,
    #[serde(rename = "type")]
    pub url_type: UrlNotificationsType,
    #[serde(rename = "notifyTime")]
    pub notify_time: String,
}

/// Google Index Batch Response
#[derive(Debug, Default)]
pub struct ResponseGoogleIndexingBatch {
    url: String,
    status_code: u16,
    value: String,
}

impl ResponseGoogleIndexingBatch {
    pub fn url(&self) -> &str {
        self.url.as_str()
    }
    pub fn status_code(&self) -> u16 {
        self.status_code
    }
    pub fn value(&self) -> &str {
        self.value.as_str()
    }
    pub fn json(&self) -> Value {
        let v = serde_json::from_str(self.value.as_str());
        if v.is_ok() {
            return v.unwrap();
        }
        Value::default()
    }
}
