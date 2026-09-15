# Contributing

Bug reports and pull requests are welcome on
[GitHub](https://github.com/uiuifree/rust-google-indexing-api).

## Before opening a pull request

Run the same checks as CI:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets
cargo test --doc
```

The minimum supported Rust version is declared in `Cargo.toml` (`rust-version`) and
checked in CI; please do not use newer language features without bumping it and
noting the change in `CHANGELOG.md`.

## Live test

`tests/test_inspection.rs` calls the real Google Indexing API and is marked `#[ignore]`.
To run it, put a service account key at `./test.json` (it is git-ignored) and run:

```bash
cargo test -- --ignored
```

## Documentation

The crate-level documentation on docs.rs is generated from `README.md`, so keep the
code examples there compiling (they run as doc tests) and add a line to
`CHANGELOG.md` under `Unreleased` for user-visible changes.
