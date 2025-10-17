# og-euler-anunoby

A Rust/Dioxus app for interactive quantitative analysis and streaming demos. It exposes single-asset, multi-asset, and portfolio analysis views with charting and optional server-backed operations.

## Preview

![UI preview](assets/ui-preview.png)

Place the provided screenshot at `assets/ui-preview.png` to render on GitHub.

## Features
- Single Asset, Multi-Asset, Portfolio analysis views
- Charting via `plotters` (candlesticks, lines, etc.)
- Router-based navigation with Dioxus 0.6
- Optional Polygon News fetching (provide API key in UI)
- Optional server mode (Axum + Tokio) with optional data/ML backends

## Tech Stack
- Rust 1.74+ (edition 2021)
- Dioxus 0.6 (web/fullstack/router)
- Plotters, Rayon, Serde, Polars (optional)
- Axum + Tokio (optional, for `server` feature)

## Project Layout
- `assets/` – static assets bundled for web
- `src/`
  - `main.rs` – launches web or server (feature-gated)
  - `lib.rs` – routes and app entry
  - `css_files/` – styles
  - `charts/` – plotting helpers
  - `data_structures/` – shared types
  - `model_request/` – request builder + executor (web)
  - `news/` – Polygon models and requests
  - `ops/` – utility ops (datetime, etc.)
  - `prompting/`, `surr_queries/`, `tables/`, `views/` – UI modules
- `Cargo.toml` – features and dependencies
- `Dioxus.toml` – Dioxus web config

## Quick Start (Web, default)
Prereqs:
- Rust toolchain installed
- Dioxus CLI (recommended): `cargo install dioxus-cli`

Run dev server:
```
dx serve
```

Build for web:
```
dx build --release
```
Artifacts are written to `dist/`.

## Server Mode (optional)
Enable the `server` feature to run with Axum/Tokio and optional backends:
```
cargo run --features server
```
Notes:
- `server` enables optional deps: `surrealdb`, `polars`, `ml_backend` (path = `../bento_queries`), `axum`, `tokio`, `dioxus-cli-config`.
- If `../bento_queries` is not present, add it as a sibling repo or avoid `--features server`.

## News (Polygon)
The `news` module can call Polygon’s News API. Provide your Polygon API key in the UI where prompted.

## Development
- Format: `cargo fmt`
- Lint: `cargo clippy`
- Build (web default): `cargo build`
- Run (web default): `cargo run`

## Notes
- `Cargo.lock` is tracked (binary/project). Do not ignore it unless publishing a library.
- Optional crates (e.g., `polars`, `surrealdb`) compile only under `--features server`.
