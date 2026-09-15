# google-indexing-api — Rust client for the Google Indexing API

[![Crates.io](https://img.shields.io/crates/v/google-indexing-api?style=flat-square)](https://crates.io/crates/google-indexing-api)
[![Documentation](https://img.shields.io/docsrs/google-indexing-api?style=flat-square)](https://docs.rs/google-indexing-api)
[![CI](https://img.shields.io/github/actions/workflow/status/uiuifree/rust-google-indexing-api/ci.yml?branch=main&style=flat-square&label=CI)](https://github.com/uiuifree/rust-google-indexing-api/actions/workflows/ci.yml)
[![MSRV](https://img.shields.io/badge/MSRV-1.88-blue?style=flat-square)](https://github.com/uiuifree/rust-google-indexing-api/blob/main/Cargo.toml)
[![License](https://img.shields.io/crates/l/google-indexing-api?style=flat-square)](https://github.com/uiuifree/rust-google-indexing-api/blob/main/LICENSE)
[![Downloads](https://img.shields.io/crates/d/google-indexing-api?style=flat-square)](https://crates.io/crates/google-indexing-api)

`google-indexing-api` is an async Rust library (crate) for the
[Google Indexing API v3](https://developers.google.com/search/apis/indexing-api/v3/quickstart).
It lets a Rust application tell Google that a page was **added, updated, or deleted**
so that Google crawls it sooner, which is the basis of SEO for job boards
(Google for Jobs / `JobPosting`) and livestream pages (`BroadcastEvent`).

It wraps the three Indexing API operations:

- **`urlNotifications:publish`** — notify Google about one URL (`URL_UPDATED` / `URL_DELETED`)
- **`batch`** — send up to 100 publish notifications in a single multipart HTTP request
- **`urlNotifications/metadata`** — read the last notification Google received for a URL

The crate is small and dependency-light (`reqwest`, `serde`, `serde_json`, `urlencoding`),
works with any Tokio-based application, and does not depend on the Google API client
generator. Authentication is left to you: pass an OAuth 2.0 access token from a service
account (for example via [`yup-oauth2`](https://crates.io/crates/yup-oauth2) or
[`gcp_auth`](https://crates.io/crates/gcp_auth)).

> **Note:** Google officially supports the Indexing API only for pages containing
> [`JobPosting`](https://developers.google.com/search/docs/appearance/structured-data/job-posting) or
> [`BroadcastEvent`](https://developers.google.com/search/docs/appearance/structured-data/video#broadcast-event)
> structured data. For other page types, use sitemaps and Google Search Console instead.

## Table of contents

- [Features](#features)
- [What this crate does and does not do](#what-this-crate-does-and-does-not-do)
- [API coverage](#api-coverage)
- [Prerequisites](#prerequisites)
- [Installation](#installation)
- [Quick start](#quick-start)
- [Usage](#usage)
  - [Single URL operations](#single-url-operations)
  - [Batch operations](#batch-operations)
  - [Error handling](#error-handling)
- [Quota and rate limits](#quota-and-rate-limits)
- [Examples](#examples)
- [Testing](#testing)
- [FAQ](#faq)
- [Alternatives](#alternatives)
- [Requirements](#requirements)
- [Contributing](#contributing)
- [License](#license)
- [Links](#links)
- [日本語での概要](#日本語での概要)

## Features

- **URL notifications**: notify Google about URL updates and deletions
- **Metadata retrieval**: fetch the latest notification Google recorded for a URL
- **Batch operations**: process up to 100 URLs in one HTTP request (multipart/mixed batch)
- **Async/await**: built on `reqwest` and runs on Tokio
- **Type-safe API**: notification types and responses are Rust enums and structs
- **Error handling**: one `GoogleApiError` enum that implements `std::error::Error`
- **Tested**: mock-server tests for every method, plus an opt-in live test against Google

## What this crate does and does not do

| Does | Does not |
|------|----------|
| Build and send Indexing API requests | Obtain or refresh OAuth 2.0 access tokens |
| Parse single and batch responses into Rust types | Retry, queue, or throttle requests |
| Validate batch size (1 to 100) before sending | Check whether a URL is actually indexed (use the Search Console URL Inspection API) |
| Return the HTTP status and body on API errors | Manage Search Console properties or sitemaps |

## API coverage

| Google Indexing API endpoint | Method in this crate | Returns |
|------------------------------|----------------------|---------|
| `POST /v3/urlNotifications:publish` | `UrlNotificationsApi::publish(token, url, UrlNotificationsType)` | `serde_json::Value` (raw response) |
| `GET /v3/urlNotifications/metadata?url=...` | `UrlNotificationsApi::get_metadata(token, url)` | `ResponseUrlNotificationMetadata` |
| `POST /batch` (multipart of `publish` calls) | `UrlNotificationsApi::batch(token, urls, UrlNotificationsType)` | `Vec<ResponseGoogleIndexingBatch>` |

`UrlNotificationsType` maps to the API's `type` field:

| Variant | Wire value | Meaning |
|---------|------------|---------|
| `UrlNotificationsType::UPDATED` | `URL_UPDATED` | The URL was added or its content changed |
| `UrlNotificationsType::DELETED` | `URL_DELETED` | The URL was removed |
| `UrlNotificationsType::UrlNotificationTypeUnspecified` | `URL_NOTIFICATION_TYPE_UNSPECIFIED` | Default value; only appears in responses |

## Prerequisites

Before using this library, you need to:

1. **Enable the Google Indexing API** in your [Google Cloud Console](https://console.cloud.google.com/)
2. **Create a service account** and download the JSON key file
3. **Add the service account as an owner** of your property in Google Search Console

For detailed setup instructions, see the [Google Indexing API prerequisites](https://developers.google.com/search/apis/indexing-api/v3/prereqs).

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
google-indexing-api = "1.1"
tokio = { version = "1", features = ["full"] }
yup-oauth2 = "12" # or any other way to obtain an OAuth 2.0 access token
```

## Quick start

Authenticate with a service account, then notify Google that a page was updated:

```rust,no_run
use google_indexing_api::{GoogleIndexingApi, UrlNotificationsType};
use yup_oauth2::ServiceAccountAuthenticator;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load service account credentials
    let secret = yup_oauth2::read_service_account_key("service-account-key.json").await?;

    // Create authenticator and get an access token for the Indexing API scope
    let auth = ServiceAccountAuthenticator::builder(secret).build().await?;
    let scopes = &["https://www.googleapis.com/auth/indexing"];
    let token = auth.token(scopes).await?;
    let token_str = token.token().ok_or("access token is missing")?;

    // Initialize the API client
    let api = GoogleIndexingApi::url_notifications();

    // Notify Google about a URL update
    let response = api
        .publish(token_str, "https://example.com/page1", UrlNotificationsType::UPDATED)
        .await?;

    println!("Google accepted the notification: {response}");
    Ok(())
}
```

## Usage

### Single URL operations

```rust,no_run
use google_indexing_api::{GoogleApiError, GoogleIndexingApi, UrlNotificationsType};

async fn single_url(token_str: &str) -> Result<(), GoogleApiError> {
    let api = GoogleIndexingApi::url_notifications();

    // Notify about an added or updated URL
    let response = api
        .publish(token_str, "https://example.com/new-article", UrlNotificationsType::UPDATED)
        .await?;
    println!("{response}");

    // Notify about a deleted URL
    let response = api
        .publish(token_str, "https://example.com/old-article", UrlNotificationsType::DELETED)
        .await?;
    println!("{response}");

    // Get metadata about the notifications previously sent for a URL
    let metadata = api.get_metadata(token_str, "https://example.com/article").await?;
    if let Some(update) = metadata.latest_update {
        println!("last URL_UPDATED notification: {}", update.notify_time);
    }
    if let Some(remove) = metadata.latest_remove {
        println!("last URL_DELETED notification: {}", remove.notify_time);
    }
    Ok(())
}
```

### Batch operations

Send up to 100 URLs in a single HTTP request. Each URL still counts toward
your daily quota, but batching cuts the number of round trips:

```rust,no_run
use google_indexing_api::{GoogleApiError, GoogleIndexingApi, UrlNotificationsType};

async fn batch(token_str: &str) -> Result<(), GoogleApiError> {
    let api = GoogleIndexingApi::url_notifications();

    let urls = vec![
        "https://example.com/page1".to_string(),
        "https://example.com/page2".to_string(),
        "https://example.com/page3".to_string(),
    ];

    let batch_response = api.batch(token_str, urls, UrlNotificationsType::UPDATED).await?;

    // One entry per URL, in the same order as the input
    for result in batch_response {
        println!("URL: {}", result.url());
        println!("Status code: {}", result.status_code());
        // json() returns a serde_json::Value (Value::Null if the body is not JSON)
        println!("Response: {:?}", result.json());
    }
    Ok(())
}
```

### Error handling

All API calls return `GoogleApiError`. It implements `std::error::Error`, so it works
with `?` and `Box<dyn std::error::Error>`:

```rust,no_run
use google_indexing_api::{GoogleApiError, GoogleIndexingApi, UrlNotificationsType};

async fn handle_errors(token_str: &str, url: &str) {
    let api = GoogleIndexingApi::url_notifications();

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
}
```

Typical HTTP errors from Google and what they mean:

| Status | Usual cause |
|--------|-------------|
| `403 Forbidden` | The service account is not an owner of the Search Console property, or the API is not enabled |
| `429 Too Many Requests` | Daily publish quota exhausted |
| `400 Bad Request` | Malformed URL or unsupported notification type |

## Quota and rate limits

The Google Indexing API has the following default quotas per Google Cloud project:

- **200 publish requests per day** (`URL_UPDATED` and `URL_DELETED` combined)
- **180 metadata requests per minute**

For batch operations, each URL in the batch counts toward the daily publish quota.
You can request a quota increase from the Google Cloud Console.

## Examples

Runnable examples live in the
[`examples/`](https://github.com/uiuifree/rust-google-indexing-api/tree/main/examples)
directory. Both take the path to a service account JSON key and use `yup-oauth2`
to obtain a token:

```bash
# Notify Google about one URL and then read its notification metadata
cargo run --example publish -- service-account-key.json https://example.com/jobs/1

# Notify Google about several URLs in one batch request
cargo run --example batch -- service-account-key.json https://example.com/jobs/1 https://example.com/jobs/2
```

## Testing

```bash
cargo test            # unit and mock-server tests; no credentials needed
cargo test -- --ignored   # live test against Google; needs ./test.json (service account key)
```

Mock-server tests covering success and error paths for all three methods live in
`src/http/mod.rs`. The live integration test in
[`tests/`](https://github.com/uiuifree/rust-google-indexing-api/tree/main/tests)
calls the real Google API and is therefore marked `#[ignore]`.

## FAQ

**Does this crate obtain the OAuth 2.0 access token for me?**
No. It only sends requests. Get a token from a service account with
`yup-oauth2`, `gcp_auth`, or the Google Cloud SDK, and pass it as `&str`.
The required scope is `https://www.googleapis.com/auth/indexing`.

**Can I use the Indexing API to index a blog or e-commerce site faster?**
Google documents the API only for `JobPosting` and `BroadcastEvent` pages.
Requests for other pages may be accepted, but Google does not guarantee they
have any effect. Use sitemaps and Search Console for general pages.

**Does `get_metadata` tell me whether a URL is indexed?**
No. It returns the last `URL_UPDATED` / `URL_DELETED` notification that Google
received for the URL through the Indexing API. Use the Search Console
URL Inspection API to check index status.

**How many URLs can I send at once?**
Up to 100 per `batch` call, which is Google's limit. Larger or empty inputs are
rejected locally with `GoogleApiError::InvalidArgument`.

**Which TLS backend is used?**
`reqwest` 0.13 with its default `rustls` backend. No OpenSSL is required.

**Is the crate `Send`/`Sync` and usable from a web server?**
Yes. The client holds no state; you can create it per request or share it freely.

## Alternatives

- [`google-indexing3`](https://crates.io/crates/google-indexing3) — generated bindings from
  the `google-apis-rs` project, covering the same API with a heavier, generic client.
- Calling the REST API directly with `reqwest` — this crate mainly saves you the
  multipart batch request and response parsing.

## Requirements

- Rust 1.88 or later (checked in CI)
- Tokio runtime for async operations

## Contributing

Contributions are welcome. See
[CONTRIBUTING.md](https://github.com/uiuifree/rust-google-indexing-api/blob/main/CONTRIBUTING.md)
for how to run the checks locally.

## License

This project is licensed under the MIT License. See the
[LICENSE](https://github.com/uiuifree/rust-google-indexing-api/blob/main/LICENSE) file for details.

## Links

- [crates.io](https://crates.io/crates/google-indexing-api)
- [API documentation on docs.rs](https://docs.rs/google-indexing-api)
- [Changelog](https://github.com/uiuifree/rust-google-indexing-api/blob/main/CHANGELOG.md)
- [Repository](https://github.com/uiuifree/rust-google-indexing-api)
- [Google Indexing API documentation](https://developers.google.com/search/apis/indexing-api/v3/quickstart)
- [Google Indexing API reference (urlNotifications)](https://developers.google.com/search/apis/indexing-api/v3/reference/indexing/rest/v3/urlNotifications)

## 日本語での概要

`google-indexing-api` は、Google Indexing API を Rust から呼び出すための非同期クライアントです。
求人ページ（`JobPosting`、Google しごと検索）やライブ配信ページ（`BroadcastEvent`）の
追加・更新・削除を Google に通知し、クロールを早めるために使います。

- `publish` — URL を 1 件ずつ通知（`URL_UPDATED` / `URL_DELETED`）
- `batch` — 最大 100 件を 1 回の HTTP リクエストでまとめて通知
- `get_metadata` — その URL について Google が最後に受け取った通知を取得

アクセストークンの取得はこのクレートの範囲外です。`yup-oauth2` などでサービスアカウントから
取得したトークンを `&str` で渡してください。使い方は上の英語セクションのコード例と、
`examples/` ディレクトリを参照してください。
