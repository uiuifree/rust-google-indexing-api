# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.1.0] - 2026-08-15

### Breaking

- Added the `GoogleApiError::HttpStatus(u16, String)` and
  `GoogleApiError::InvalidArgument(String)` variants. Code that matches
  `GoogleApiError` exhaustively (without a `_` arm) will need new match arms.
- `batch` now rejects inputs outside 1 to 100 URLs with
  `GoogleApiError::InvalidArgument` instead of sending them (the API limit is 100).
- The default TLS backend changed from native-tls (OpenSSL) to rustls, following
  the reqwest 0.13 update.
- The minimum supported Rust version is now 1.88.

### Added

- `GoogleApiError` now implements `std::error::Error` and `Display`, so it works
  with `?` and `Box<dyn std::error::Error>` as shown in the README.
- API error responses now keep the HTTP status code and response body via
  `GoogleApiError::HttpStatus` instead of being folded into `JsonParse`.
- Unit tests for the batch response parser and mock-server tests covering
  success and error paths for all three API methods. They run with a plain
  `cargo test` and need no credentials.
- GitHub Actions CI: `cargo fmt --check`, clippy with `-D warnings`,
  `cargo test --all-targets`, and doc tests run on every push and pull request.
- `LICENSE` file (MIT).

### Changed

- Replaced the direct hyper client in the batch API with reqwest, and removed
  the `hyper`, `hyper-tls`, `hyper-util`, and `http-body-util` dependencies.
- Updated reqwest to 0.13 and trimmed its features to `json` only.
- The live integration test is now marked `#[ignore]`; run it with
  `cargo test -- --ignored` and a service account key at `./test.json`.
- README fixes: code examples now compile as written (`status_code()`,
  `json()` returning `serde_json::Value`, error handling example), method
  signatures match the implementation, `get_metadata` is described as returning
  metadata about past notifications (not index status), and a note clarifies
  that Google supports the Indexing API only for `JobPosting` and
  `BroadcastEvent` pages.
- Improved crates.io metadata (description, categories, documentation,
  rust-version).

### Fixed

- The batch request body now ends with the proper multipart terminator
  (`--boundary--`); it previously ended with a plain `--boundary`.
- Malformed batch responses (no boundary in `Content-Type`, no closing boundary
  in the body, a part count that does not match the request, an unknown or
  duplicated `Content-ID`, or an unparsable HTTP status line in a part) now
  return a `JsonParse` error instead of silently succeeding with wrong results.
  Quoted boundaries (`boundary="..."`) are now parsed correctly.
- Reading a response body could panic instead of returning an error, and a
  non-text `Content-Type` header in the batch response could panic as well.
- The batch request no longer sends an invalid `Content-Length` header
  (the literal string `content_length`); the length is now set automatically.

## [1.0.0] - 2025-12-20

### Changed

- Updated dependencies: reqwest 0.11 → 0.12, hyper 0.14 → 1.0
  (with `hyper-util` and `http-body-util`), yup-oauth2 7 → 12.
- Rewrote the README with setup instructions, examples, and API reference.

## [0.1.x] - 2023

- Initial releases: `publish` and `get_metadata` for single URLs (0.1.0),
  batch requests for up to 100 URLs (0.1.1), and metadata/README updates
  (0.1.3, 0.1.4).
