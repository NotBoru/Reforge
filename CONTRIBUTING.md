# Contributing

## Setup

Install Rust (`x86_64-pc-windows-msvc`) and Visual Studio C++ Build Tools, then run:

```powershell
cargo test
cargo run
```

## Adding apps to the catalog

Add entries in `src/core/catalog.rs` with a valid `winget_id`, short English/Portuguese descriptions, and `detect_keywords`. Run `cargo test` to verify there are no duplicate IDs.

## Pull requests

Make sure `cargo test --locked` and `cargo build --release --locked` succeed without warnings before opening a PR.
