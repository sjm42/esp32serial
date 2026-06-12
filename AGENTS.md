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
Use the Rust toolchain from `rust-toolchain.toml` and an ESP-IDF environment. The default ESP32-C3 target uses `nightly`; the ESP-WROOM-32/Xtensa target uses the `esp` toolchain.

- `cargo build -r`: build firmware for the default target/feature (`esp32-c3`).
- `./flash_c3`: build, flash, and monitor the default ESP32-C3 target via `cargo run -r`.
- `./flash_wroom32`: build, flash, and monitor the ESP-WROOM-32 target via `cargo +esp run`.
- `./make_ota_image_c3`: build and export `firmware-c3.bin` with `espflash`.
- `./make_ota_image_wroom32`: build and export `firmware-wroom32.bin` with `espflash`.
- `cargo check`: fast compile check during development.
- `cargo test --no-run`: compile test artifacts without invoking the ESP flashing runner.
- `cargo fmt`: format Rust code.
- `cargo clippy --all-targets`: lint the default ESP32-C3 feature set.
- `MCU=esp32 cargo +esp clippy --target xtensa-esp32-espidf --no-default-features --features esp-wroom-32`: lint the ESP-WROOM-32 feature set.
- `cargo update --dry-run`: check whether compatible lockfile updates are available.
- `cargo outdated --root-deps-only`: check direct dependencies for newer incompatible releases.

Example alternate target build:
`MCU=esp32 cargo +esp build -r --target xtensa-esp32-espidf --no-default-features --features esp-wroom-32`

`Cargo.lock` is committed for reproducible firmware builds. Apply compatible updates with `cargo update`; treat semver-incompatible direct dependency upgrades as code changes that need a normal build.

## Coding Style & Naming Conventions
- Follow standard Rust style (4-space indentation, `snake_case` for functions/modules, `CamelCase` for types, `SCREAMING_SNAKE_CASE` for constants).
- Keep modules focused by subsystem (`wifi.rs`, `serial.rs`, etc.).
- Prefer small, explicit async tasks and clear error propagation with `anyhow`.
- Run `cargo fmt` and `cargo clippy` before opening a PR.

## Testing Guidelines
There is no dedicated `tests/` directory currently. Add unit tests near the code they cover (for example, config parsing/serialization in `src/config.rs`).

- There are currently no unit tests in `src/`.
- Name tests by behavior, e.g. `loads_default_config`, `rejects_bad_crc`.
- Run `cargo test --no-run` locally before submitting changes. Plain `cargo test` may invoke `espflash` and require an interactive terminal plus attached hardware.

## Commit & Pull Request Guidelines
Recent history uses short, imperative commit subjects (for example, `cargo update`, `Update readme & stuff`). Keep commits focused and messages concise.

- Commit format: imperative subject line, <= 72 chars when possible.
- PRs should include: what changed, why, test/build commands run, and any hardware-specific validation performed.
- For UI/config changes, include screenshots of the web page when relevant.
