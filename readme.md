# Rust Google Indexing API
![Crates.io](https://img.shields.io/crates/v/google_indexing_api?style=flat-square)

This is a Rust library for interfacing with the Google Indexing API. It allows you to notify Google about pages on your
web app for indexing in search results.

## Features

* URL Notifications: Seamlessly notify Google about updates and deletions of URLs on your website.
* URL Metadata Retrieval: Obtain metadata about a URL from the Google Index.
* Batch Operations: Efficiently notify Google about multiple URLs in a single batch operation.

## Installation

To use the google-indexing-api in your Rust project, add it as a dependency in your Cargo.toml:

```toml
[dependencies]
google-indexing-api = "0.1"  
```

## Usage

```rust
async fn main() {
    use google_indexing_api::{GoogleIndexingApi, UrlNotificationsType};

    let api = GoogleIndexingApi::url_notifications();

// Notify Google about a URL update
    let response = api.publish("YOUR_GOOGLE_TOKEN", "https://yourwebsite.com/page1", UrlNotificationsType::UPDATED).await;
// Notify Google about a URL delete
    let response = api.publish("YOUR_GOOGLE_TOKEN", "https://yourwebsite.com/page1", UrlNotificationsType::DELETED).await;

// Retrieve metadata about a URL
    let metadata = api.get_metadata("YOUR_GOOGLE_TOKEN", "https://yourwebsite.com/page1").await;

// Batch notify Google about multiple URL updates
    let urls = vec!["https://yourwebsite.com/page1", "https://yourwebsite.com/page2"];
    let batch_response = api.batch("YOUR_GOOGLE_TOKEN", urls, UrlNotificationsType::UPDATED).await;
}

```


## Error Handling
The library provides structured error handling through the GoogleApiError enum. You can match against it to handle specific error scenarios in your application.