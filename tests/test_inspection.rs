use std::io::Write;
use reqwest::{Client, multipart};
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderValue};
use reqwest::multipart::{Form, Part};
use yup_oauth2::{AccessToken};
use google_indexing_api::{GoogleIndexingApi, UrlNotificationsType};


async fn test_token() -> AccessToken {
    // 認証
    let secret = yup_oauth2::read_service_account_key("./test.json")
        .await
        .expect("test.json");
    let auth = yup_oauth2::ServiceAccountAuthenticator::builder(secret).build().await.unwrap();
    let scopes = &["https://www.googleapis.com/auth/indexing"];

    let token = auth.token(scopes).await;
    assert!(token.is_ok(), "{}", token.err().unwrap().to_string());
    token.unwrap()
}


#[tokio::test]
async fn test_sitemaps() {
    let token = test_token().await;
    let test_url = "https://example.com/".to_string();
    let res = GoogleIndexingApi::url_notifications().publish(
        token.as_str(), test_url.as_str(), UrlNotificationsType::DELETED).await;
    assert!(res.is_ok());
    let res = GoogleIndexingApi::url_notifications().get_metadata(
        token.as_str(), test_url.as_str()).await.unwrap();
    assert!(res.is_ok());
}
