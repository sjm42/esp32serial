# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

ESP32 serial port TCP server written in Rust. It bridges UART serial communication with TCP/IP, allowing remote clients to connect via raw TCP (default port 23) to read and optionally write a serial device. The firmware includes a web UI for configuration and OTA firmware updates.

## Build & Flash Commands

```bash
# Build release firmware (default target: esp32-c3)
cargo build --release

# Build and flash ESP32-C3
./flash_c3             # runs: cargo run -r

# Build and flash ESP-WROOM-32
./flash_wroom32        # runs xtensa target build with MCU=esp32

# Generate OTA firmware images
./make_ota_image_c3
./make_ota_image_wroom32

# Lint
cargo clippy --all-targets --all-features
```

**Default target:** `riscv32imc-esp-espidf` (`esp32-c3` feature)

**Alternate target:** `xtensa-esp32-espidf` with `--no-default-features --features esp-wroom-32`

**Features:** `esp32-c3` (default), `esp-wroom-32`, plus deprecated aliases `esp32c3` and `esp32s`

**Toolchain:** Rust nightly by default (`rust-toolchain.toml`). The Xtensa build scripts use `cargo +esp`.

There are no unit tests in this project.

## Architecture

Five concurrent async tasks run in a `tokio::select!` loop (see `src/bin/esp32serial.rs`):

1. **Serial bridge** (`src/serial.rs`) — UART1 ↔ TCP bridge. Reads UART and broadcasts to all connected TCP clients via `tokio::sync::broadcast`. Client writes go to UART via `mpsc` channel. Toggles LED on activity.

2. **WiFi manager** (`src/wifi.rs`) — Handles WPA2-Personal/Enterprise, DHCP or static IPv4, automatic reconnection, and hostname derived from the MAC address.

3. **API server** (`src/apiserver.rs`) — Axum HTTP server on port `80` serving the web config UI (Askama template in `templates/index.html.ask`). Endpoints include `GET /`, `GET /form.js`, `GET /index.css`, `GET /favicon.ico`, `GET/POST /conf`, `GET /reset_conf`, and `POST /fw`. `src/form.js` converts HTML form data to JSON for POST.

4. **Reset button** (`src/bin/esp32serial.rs:poll_reset`) — Monitors the target-specific reset pin (`GPIO9` on `esp32-c3`, `GPIO0` on `esp-wroom-32`); long press triggers factory reset.

5. **Pinger** (`src/bin/esp32serial.rs:pinger`) — Pings gateway every 5 minutes; reboots on failure.

## Key Modules

- **`src/config.rs`** — `MyConfig` struct with WiFi, IPv4, and serial settings. Persisted to NVS using `postcard` binary serialization with CRC32 validation.
- **`src/state.rs`** — `MyState` shared state (`Arc<Pin<Box<MyState>>>`). Contains config, OTA info, WiFi status, NVS access, restart state, and target-specific serial hardware pins.

## GPIO Pinout

### esp32-c3

| GPIO | Function |
|------|----------|
| 0    | UART1 TX |
| 1    | UART1 RX |
| 8    | Status LED |
| 9    | Reset button |

### esp-wroom-32

| GPIO | Function |
|------|----------|
| 17   | UART1 TX |
| 16   | UART1 RX |
| 2    | Status LED |
| 0    | Reset button |

## Flash Partition Layout

Dual OTA slots (ota_0/ota_1, ~2MB each) with NVS for config storage. Defined in `partitions.csv`.
