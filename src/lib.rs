mod error;
mod http;


use serde_derive::{Deserialize, Serialize};
use serde_json::{json, Value};
use urlencoding::encode;
use crate::{GoogleApiError};
pub use error::*;
use crate::http::HttpClient;


pub struct GoogleIndexingApi {}

impl GoogleIndexingApi {
    pub fn url_notifications() -> UrlNotificationsApi {
        UrlNotificationsApi::default()
    }
}

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
            UrlNotificationsType::UPDATED => { "URL_UPDATED".to_string() }
            UrlNotificationsType::DELETED => { "URL_DELETED".to_string() }
            UrlNotificationsType::UrlNotificationTypeUnspecified => { "URL_NOTIFICATION_TYPE_UNSPECIFIED".to_string() }
        }
    }
}

/// https://developers.google.com/webmaster-tools/v1/searchanalytics
#[derive(Default)]
pub struct UrlNotificationsApi {}


impl UrlNotificationsApi {
    pub async fn publish(&self, token: &str, url: &str, url_type: UrlNotificationsType) -> Result<Value, GoogleApiError> {
        Ok(HttpClient::post(
            token,
            format!(r#"https://indexing.googleapis.com/v3/urlNotifications:publish"#, ).as_str(),
            json!({
                "url": url,
                "type": url_type.to_string(),
            }),
        ).await?)
    }
    pub async fn get_metadata(&self, token: &str, url: &str) -> Result<ResponseUrlNotificationMetadata, GoogleApiError> {
        Ok(HttpClient::get(
            token,
            format!(r#"https://indexing.googleapis.com/v3/urlNotifications/metadata?url={}"#, encode(url), ).as_str(),
        ).await?)
    }
    pub async fn batch(&self, token: &str, urls: Vec<String>, url_type: UrlNotificationsType) -> Result<Vec<GoogleIndexingBatch>, GoogleApiError> {
        Ok(HttpClient::execute(
            token,
            urls,
            url_type,
        ).await?)
    }
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct ResponseUrlNotificationMetadata {
    pub url: String,
    #[serde(rename = "latestUpdate")]
    pub latest_update: Option<ResponseUrlNotification>,
    #[serde(rename = "latestRemove")]
    pub latest_remove: Option<ResponseUrlNotification>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ResponseUrlNotification {
    pub url: String,
    #[serde(rename = "type")]
    pub url_type: UrlNotificationsType,
    #[serde(rename = "notifyTime")]
    pub notify_time: String,
}


#[derive(Debug, Default)]
pub struct GoogleIndexingBatch {
    url: String,
    status_code: u16,
    value: String,
}

impl GoogleIndexingBatch {
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