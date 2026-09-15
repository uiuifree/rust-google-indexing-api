//! Notify Google about several URLs in one batch request (up to 100 URLs).
//!
//! Usage:
//!   cargo run --example batch -- <service-account-key.json> <url>...

use google_indexing_api::{GoogleIndexingApi, UrlNotificationsType};
use yup_oauth2::ServiceAccountAuthenticator;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let Some(key_path) = args.next() else {
        eprintln!("usage: cargo run --example batch -- <service-account-key.json> <url>...");
        std::process::exit(2);
    };
    let urls: Vec<String> = args.collect();

    let secret = yup_oauth2::read_service_account_key(&key_path).await?;
    let auth = ServiceAccountAuthenticator::builder(secret).build().await?;
    let token = auth
        .token(&["https://www.googleapis.com/auth/indexing"])
        .await?;
    let token_str = token.token().ok_or("access token is missing")?;

    let results = GoogleIndexingApi::url_notifications()
        .batch(token_str, urls, UrlNotificationsType::UPDATED)
        .await?;

    for result in results {
        println!(
            "{} -> {} {}",
            result.url(),
            result.status_code(),
            result.json()
        );
    }
    Ok(())
}
