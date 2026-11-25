# Contributing to Reforge

Thank you for your interest in contributing to **Reforge**!

## Development Setup

1. Install [Rust](https://rustup.rs/) (`x86_64-pc-windows-msvc`) and Visual Studio C++ Build Tools.
2. Clone the repo and run:
   ```powershell
   cargo test
   cargo run
   ```

## Adding a Curated App to the Installer

Edit `src/core/catalog.rs` and add a `CatalogApp` entry:
- Always provide an exact, verified `winget_id` (used as fallback & updater link).
- Prefer official vendor evergreen download URLs (`DirectSource::StaticUrl`) or official `DirectSource::GitHubRelease` entries with verified silent install flags (`/S`, `/VERYSILENT /NORESTART`, `/qn /norestart`).
- Add accurate lowercase `detect_keywords` so Reforge detects existing installations cleanly.
- Provide both `desc_en` and `desc_pt` concise descriptions.
- Run `cargo test` to verify catalog uniqueness and integrity.

## Safety Standards

- Never add tweaks that permanently break Windows Update, Windows Defender, or Windows Store without explicit user opt-in and a clean `revert` path.
- Never target browser profiles, cookies, credentials, or user documents in `src/core/cleaner.rs`.
- Keep all external processes hidden (`hidden_std_cmd` / `hidden_tokio_cmd`) and non-blocking.

## Pull Requests

- Ensure `cargo test` passes with zero warnings.
- Ensure `cargo build --release` succeeds cleanly.
- Keep commits focused and descriptive.
