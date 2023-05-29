use std::collections::HashMap;
use std::io::Write;
use hyper::{Body, Client, Method, Request, Response};
use hyper::body::HttpBody;
use hyper::client::HttpConnector;
use hyper::header::CONTENT_LENGTH;
use hyper_tls::HttpsConnector;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderValue};
use serde_json::json;
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
    let a = GoogleIndexingApi::url_notifications().batch(
        token.as_str(),
        vec![
            "http://example.com/widgets/1".to_string(),
            "http://example.com/widgets/2".to_string(),
        ],
        UrlNotificationsType::UPDATED
    ).await;
    assert!(a.is_ok(), "{}", a.err().unwrap().to_string());

    for value in a.unwrap(){
        println!("{} {:?}",value.url(),value.json());
    }
}

