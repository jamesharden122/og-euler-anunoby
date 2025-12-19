# Repository Guidelines

## Project Structure & Module Organization
- `src/main.rs`: entry point; launches Dioxus for web by default, or an Axum server when built with the `server` feature.
- `src/lib.rs` and `src/**`: app modules:
  - `views/` (UI screens), `charts/` (Plotters-based charts), `news/` (Polygon API models), `model_request/` (HTTP/execution helpers), `ops/` and `tables/` (data ops/UI tables), `css_files/` (component styles).
- `assets/`: static assets served in web builds.
- `Cargo.toml`: feature flags (`web` default, `server`, `desktop`, `mobile`).
- `Dioxus.toml`: web app metadata and dev/build settings.
- `.env.example`: server IP/PORT and optional API keys.

## Build, Test, and Development Commands
- Web dev (hot reload): `dx serve` (install with `cargo install dioxus-cli`).
- Web release build: `dx build --release` → emits `dist/`.
- Server/fullstack dev: `IP=127.0.0.1 PORT=8080 cargo run --features server` (requires optional deps in `Cargo.toml`).
- Desktop (optional): `cargo run --features desktop`.
- Lint/format: `cargo clippy -- -D warnings` and `cargo fmt --all`.

## Coding Style & Naming Conventions
- Rust 2021; 4-space indentation; keep functions small and pure where possible.
- Naming: modules/files `snake_case`; types/enums `CamelCase`; functions/vars `snake_case`; constants `SCREAMING_SNAKE_CASE`.
- Organize UI under `src/views/`; chart code in `src/charts/`; HTTP/data helpers in `src/model_request/`, `src/ops/`.
- CSS files use kebab-case (e.g., `home_style.css`, `model_button.css`) and live in `src/css_files/` or `assets/styling/`.

## Testing Guidelines
- Framework: built-in Rust tests. Run all with `cargo test`.
- Unit tests live alongside modules using `#[cfg(test)]`; integration tests go under `tests/`.
- Name tests by behavior (e.g., `build_url_includes_api_key`).
- Prefer fast, deterministic tests; mock network calls instead of hitting Polygon in CI.

## Commit & Pull Request Guidelines
- Commits: imperative mood with optional scope (e.g., `feat(views): add portfolio table`). Keep subject ≤ 72 chars; include rationale in body and link issues.
- PRs: clear description, linked issues, screenshots/GIFs for UI changes, reproduction steps, and notes on feature flags used.
- Before opening: run `cargo fmt`, `cargo clippy`, and a local build (`dx build` or `cargo run --features server`).

## Security & Configuration Tips
- Copy `.env.example` to `.env`. Set `POLYGON_API_KEY` if using news features. Never commit secrets.
- In server mode, bind via `IP`/`PORT`. Use `127.0.0.1` for local development.

## References
- Dioxus docs: https://dioxuslabs.com/learn/0.7/
- Dioxus crates: https://docs.rs/dioxus/0.7.2/dioxus/index.html
- DuckDB docs: https://duckdb.org/docs/stable/
- DuckDB crate docs: https://docs.rs/duckdb/latest/duckdb/

