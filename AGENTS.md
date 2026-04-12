# Repository Guidelines

## Project Structure & Module Organization
This repository is a single Rust crate for ESP32 serial-to-TCP firmware.

- `src/bin/esp32serial.rs`: firmware entry point and top-level task orchestration.
- `src/*.rs`: core modules (`serial`, `wifi`, `apiserver`, `config`, `state`, `lib`).
- `src/form.js`, `src/index.css`, `src/favicon.ico`: embedded web UI assets.
- `templates/index.html.ask`: Askama template for the configuration page.
- `build.rs`: build-time setup for ESP-IDF integration.
- `partitions.csv`, `sdkconfig.defaults`: ESP-IDF/flash layout configuration.
- `flash_c3`, `flash_wroom32`: helper scripts for flashing the supported hardware targets.
- `make_ota_image_c3`, `make_ota_image_wroom32`: helper scripts for generating OTA images.

## Build, Test, and Development Commands
Use the Rust nightly toolchain (`rust-toolchain.toml`) and an ESP-IDF environment.

- `cargo build --release`: build firmware for the default target/feature (`esp32-c3`).
- `./flash_c3`: build, flash, and monitor the default ESP32-C3 target.
- `./flash_wroom32`: build, flash, and monitor the ESP-WROOM-32 target.
- `./make_ota_image_c3`: build and export `firmware-c3.bin` with `espflash`.
- `./make_ota_image_wroom32`: build and export `firmware-wroom32.bin` with `espflash`.
- `cargo check`: fast compile check during development.
- `cargo test`: run tests (mainly unit tests in `src/lib.rs` and module tests, if present).
- `cargo fmt`: format Rust code.
- `cargo clippy --all-targets --all-features`: lint code before submitting.

Example alternate target build:
`MCU=esp32 cargo +esp build -r --target xtensa-esp32-espidf --no-default-features --features esp-wroom-32`

## Coding Style & Naming Conventions
- Follow standard Rust style (4-space indentation, `snake_case` for functions/modules, `CamelCase` for types, `SCREAMING_SNAKE_CASE` for constants).
- Keep modules focused by subsystem (`wifi.rs`, `serial.rs`, etc.).
- Prefer small, explicit async tasks and clear error propagation with `anyhow`.
- Run `cargo fmt` and `cargo clippy` before opening a PR.

## Testing Guidelines
There is no dedicated `tests/` directory currently. Add unit tests near the code they cover (for example, config parsing/serialization in `src/config.rs`).

- There are currently no unit tests in `src/`.
- Name tests by behavior, e.g. `loads_default_config`, `rejects_bad_crc`.
- Run `cargo test` locally before submitting changes.

## Commit & Pull Request Guidelines
Recent history uses short, imperative commit subjects (for example, `cargo update`, `Update readme & stuff`). Keep commits focused and messages concise.

- Commit format: imperative subject line, <= 72 chars when possible.
- PRs should include: what changed, why, test/build commands run, and any hardware-specific validation performed.
- For UI/config changes, include screenshots of the web page when relevant.
