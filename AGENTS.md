# Repository Guidelines

## Project Structure & Module Organization
This repository is a single Rust crate for ESP32 serial-to-TCP firmware.

- `src/bin/esp32serial.rs`: firmware entry point and top-level task orchestration.
- `src/*.rs`: core modules (`serial`, `wifi`, `apiserver`, `config`, `state`).
- `src/form.js`, `src/favicon.ico`: embedded web UI assets.
- `templates/index.html.ask`: Askama template for the configuration page.
- `build.rs`: build-time setup for ESP-IDF integration.
- `partitions.csv`, `sdkconfig.defaults`: ESP-IDF/flash layout configuration.
- `flash`, `makeimage`: helper scripts for flashing and image generation.

## Build, Test, and Development Commands
Use the Rust nightly toolchain (`rust-toolchain.toml`) and an ESP-IDF environment.

- `cargo build --release`: build firmware for the default target/feature (`esp32c3`).
- `./flash`: run the firmware build and flash flow (`cargo run -r -- --baud 921600`).
- `./makeimage`: build release firmware and export `firmware.bin` with `espflash`.
- `cargo check`: fast compile check during development.
- `cargo test`: run tests (mainly unit tests in `src/lib.rs` and module tests, if present).
- `cargo fmt`: format Rust code.
- `cargo clippy --all-targets --all-features`: lint code before submitting.

Example alternate target build:
`cargo build --release --no-default-features --features esp32s`

## Coding Style & Naming Conventions
- Follow standard Rust style (4-space indentation, `snake_case` for functions/modules, `CamelCase` for types, `SCREAMING_SNAKE_CASE` for constants).
- Keep modules focused by subsystem (`wifi.rs`, `serial.rs`, etc.).
- Prefer small, explicit async tasks and clear error propagation with `anyhow`.
- Run `cargo fmt` and `cargo clippy` before opening a PR.

## Testing Guidelines
There is no dedicated `tests/` directory currently. Add unit tests near the code they cover (for example, config parsing/serialization in `src/config.rs`).

- Name tests by behavior, e.g. `loads_default_config`, `rejects_bad_crc`.
- Run `cargo test` locally before submitting changes.

## Commit & Pull Request Guidelines
Recent history uses short, imperative commit subjects (for example, `cargo update`, `Update readme & stuff`). Keep commits focused and messages concise.

- Commit format: imperative subject line, <= 72 chars when possible.
- PRs should include: what changed, why, test/build commands run, and any hardware-specific validation performed.
- For UI/config changes, include screenshots of the web page when relevant.
