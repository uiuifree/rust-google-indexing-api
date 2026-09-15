//! Notify Google about one URL, then read the notification metadata for it.
//!
//! Usage:
//!   cargo run --example publish -- <service-account-key.json> <url>

use google_indexing_api::{GoogleIndexingApi, UrlNotificationsType};
use yup_oauth2::ServiceAccountAuthenticator;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let (Some(key_path), Some(url)) = (args.next(), args.next()) else {
        eprintln!("usage: cargo run --example publish -- <service-account-key.json> <url>");
        std::process::exit(2);
    };

    let secret = yup_oauth2::read_service_account_key(&key_path).await?;
    let auth = ServiceAccountAuthenticator::builder(secret).build().await?;
    let token = auth
        .token(&["https://www.googleapis.com/auth/indexing"])
        .await?;
    let token_str = token.token().ok_or("access token is missing")?;

    let api = GoogleIndexingApi::url_notifications();

    let response = api
        .publish(token_str, &url, UrlNotificationsType::UPDATED)
        .await?;
    println!("publish response: {response}");

    let metadata = api.get_metadata(token_str, &url).await?;
    println!("metadata: {metadata:#?}");
    Ok(())
}
