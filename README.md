# Rust Google Indexing API

[![Crates.io](https://img.shields.io/crates/v/google-indexing-api?style=flat-square)](https://crates.io/crates/google-indexing-api)
[![Documentation](https://img.shields.io/docsrs/google-indexing-api?style=flat-square)](https://docs.rs/google-indexing-api)
[![License](https://img.shields.io/crates/l/google-indexing-api?style=flat-square)](https://github.com/uiuifree/rust-google-indexing-api/blob/main/LICENSE)
[![Downloads](https://img.shields.io/crates/d/google-indexing-api?style=flat-square)](https://crates.io/crates/google-indexing-api)

A Rust library for interfacing with the [Google Indexing API](https://developers.google.com/search/apis/indexing-api/v3/quickstart).
Notify Google when pages are added, updated, or deleted on your website for faster indexing in search results.

> **Note:** Google officially supports the Indexing API only for pages containing
> [`JobPosting`](https://developers.google.com/search/docs/appearance/structured-data/job-posting) or
> [`BroadcastEvent`](https://developers.google.com/search/docs/appearance/structured-data/video#broadcast-event)
> structured data. For other page types, use sitemaps and Google Search Console instead.

## Features

- **URL Notifications**: Notify Google about URL updates and deletions
- **Metadata Retrieval**: Fetch metadata about notifications previously sent to the Indexing API
- **Batch Operations**: Process multiple URLs efficiently in a single request (up to 100 URLs)
- **Async/Await Support**: Built with Tokio for modern async Rust applications
- **Type-Safe API**: Leverages Rust's type system for safer API interactions
- **Error Handling**: Comprehensive error types for robust applications

## Prerequisites

Before using this library, you need to:

1. **Enable the Google Indexing API** in your [Google Cloud Console](https://console.cloud.google.com/)
2. **Create a Service Account** and download the JSON key file
3. **Grant permissions** to the service account in Google Search Console for your property

For detailed setup instructions, see the [Google Indexing API documentation](https://developers.google.com/search/apis/indexing-api/v3/quickstart).

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
google-indexing-api = "1.1"
tokio = { version = "1", features = ["full"] }
yup-oauth2 = "12" # For authentication
```

## Quick Start

### Basic Example with Authentication

```rust
use google_indexing_api::{GoogleIndexingApi, UrlNotificationsType};
use yup_oauth2::ServiceAccountAuthenticator;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load service account credentials
    let secret = yup_oauth2::read_service_account_key("service-account-key.json")
        .await
        .expect("Failed to read service account key");

    // Create authenticator
    let auth = ServiceAccountAuthenticator::builder(secret)
        .build()
        .await?;

    // Get access token
    let scopes = &["https://www.googleapis.com/auth/indexing"];
    let token = auth.token(scopes).await?;
    let token_str = token.token().unwrap();

    // Initialize API client
    let api = GoogleIndexingApi::url_notifications();

    // Notify Google about a URL update
    let response = api.publish(
        token_str,
        "https://example.com/page1",
        UrlNotificationsType::UPDATED
    ).await?;

    println!("Successfully notified Google about the URL update");
    Ok(())
}
```

### Single URL Operations

```rust
use google_indexing_api::{GoogleIndexingApi, UrlNotificationsType};

let api = GoogleIndexingApi::url_notifications();

// Notify about an updated URL
let response = api.publish(
    token_str,
    "https://example.com/new-article",
    UrlNotificationsType::UPDATED
).await?;

// Notify about a deleted URL
let response = api.publish(
    token_str,
    "https://example.com/old-article",
    UrlNotificationsType::DELETED
).await?;

// Get metadata about notifications previously sent for a URL
let metadata = api.get_metadata(
    token_str,
    "https://example.com/article"
).await?;
```

### Batch Operations

Process up to 100 URLs in a single request for better performance:

```rust
use google_indexing_api::{GoogleIndexingApi, UrlNotificationsType};

let api = GoogleIndexingApi::url_notifications();

let urls = vec![
    "https://example.com/page1".to_string(),
    "https://example.com/page2".to_string(),
    "https://example.com/page3".to_string(),
];

let batch_response = api.batch(
    token_str,
    urls,
    UrlNotificationsType::UPDATED
).await?;

// Process batch results
for result in batch_response {
    println!("URL: {}", result.url());
    println!("Status Code: {}", result.status_code());
    // json() returns a serde_json::Value (Value::Null if the body is not JSON)
    println!("Response: {:?}", result.json());
}
```

## API Reference

### `GoogleIndexingApi::url_notifications()`

Creates a new API client for URL notifications.

### Methods

#### `publish(token: &str, url: &str, notification_type: UrlNotificationsType) -> Result<serde_json::Value, GoogleApiError>`

Notify Google about a single URL update or deletion. Returns the raw JSON response.

**Parameters:**
- `token`: OAuth2 access token
- `url`: The URL to notify Google about
- `notification_type`: Either `UrlNotificationsType::UPDATED` or `UrlNotificationsType::DELETED`

#### `get_metadata(token: &str, url: &str) -> Result<ResponseUrlNotificationMetadata, GoogleApiError>`

Retrieve metadata about notifications previously sent for a URL through the Indexing API
(`latest_update` / `latest_remove`). It does not tell you whether the URL is indexed
by Google — use the Search Console URL Inspection API for that.

**Parameters:**
- `token`: OAuth2 access token
- `url`: The URL to get notification metadata for

#### `batch(token: &str, urls: Vec<String>, notification_type: UrlNotificationsType) -> Result<Vec<ResponseGoogleIndexingBatch>, GoogleApiError>`

Notify Google about multiple URLs in a single batch request.

**Parameters:**
- `token`: OAuth2 access token
- `urls`: Vector of URLs (1 to 100 entries; other sizes are rejected with `GoogleApiError::InvalidArgument`)
- `notification_type`: Either `UrlNotificationsType::UPDATED` or `UrlNotificationsType::DELETED`

### URL Notification Types

```rust
pub enum UrlNotificationsType {
    UPDATED,  // Notify that a URL has been updated or added
    DELETED,  // Notify that a URL has been deleted
}
```

## Error Handling

All API calls return a `GoogleApiError`. It implements `std::error::Error`, so it works
with `?` and `Box<dyn std::error::Error>`:

```rust
use google_indexing_api::GoogleApiError;

match api.publish(token_str, url, UrlNotificationsType::UPDATED).await {
    Ok(response) => println!("Success: {:?}", response),
    // The request could not be sent (network problem, DNS, TLS, ...)
    Err(GoogleApiError::Connection(e)) => eprintln!("Connection error: {}", e),
    // The API answered with an error status; the status code and body are kept
    Err(GoogleApiError::HttpStatus(status, body)) => {
        eprintln!("API returned {}: {}", status, body)
    }
    // The response could not be parsed (invalid JSON or a malformed batch response)
    Err(GoogleApiError::JsonParse(e)) => eprintln!("Parse error: {}", e),
    // The input was rejected before sending (e.g. batch size out of range)
    Err(GoogleApiError::InvalidArgument(e)) => eprintln!("Invalid argument: {}", e),
}
```

## Rate Limits

The Google Indexing API has the following quotas:
- **200 requests per day** (default quota)
- Request quota increase through Google Cloud Console if needed

For batch operations, each URL in the batch counts toward your quota.

## Examples

See the [tests](tests/) directory for a live integration test with service account
authentication and batch processing. It calls the real Google API, so it is marked
`#[ignore]`: put your service account key at `./test.json` and run
`cargo test -- --ignored`. Mock-server tests covering success and error paths live
in `src/http/mod.rs` and run with a plain `cargo test`.

## Requirements

- Rust 1.85 or later
- Tokio runtime for async operations

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Links

- [Crates.io](https://crates.io/crates/google-indexing-api)
- [Documentation](https://docs.rs/google-indexing-api)
- [Changelog](CHANGELOG.md)
- [Repository](https://github.com/uiuifree/rust-google-indexing-api)
- [Google Indexing API Documentation](https://developers.google.com/search/apis/indexing-api/v3/quickstart)


