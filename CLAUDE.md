# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

ESP32-C3 serial port TCP server written in Rust. Bridges UART serial communication with TCP/IP, allowing remote clients to connect via raw TCP (default port 23) to read/write a serial device. Includes a web UI for configuration and OTA firmware updates.

## Build & Flash Commands

```bash
# Build release firmware
cargo build --release

# Build and flash to device with serial monitor
./flash                # runs: cargo run -r -- --baud 921600

# Generate firmware binary without flashing
./makeimage            # creates firmware.bin via espflash

# Clippy
cargo clippy
```

**Target:** `riscv32imc-esp-espidf` (RISC-V, configured in `.cargo/config.toml`)

**Toolchain:** Rust nightly (specified in `rust-toolchain.toml`), requires ESP-IDF v5.4.3 build environment.

There are no unit tests in this project.

## Architecture

Five concurrent async tasks run in a `tokio::select!` loop (see `src/bin/esp32serial.rs`):

1. **Serial bridge** (`src/serial.rs`) — UART1 ↔ TCP bridge. Reads UART and broadcasts to all connected TCP clients via `tokio::sync::broadcast`. Client writes go to UART via `mpsc` channel. Toggles LED on activity.

2. **WiFi manager** (`src/wifi.rs`) — Handles WPA2-Personal/Enterprise, DHCP/static IP, automatic reconnection, hostname from MAC address.

3. **API server** (`src/apiserver.rs`) — Axum HTTP server serving web config UI (Askama template in `templates/index.html.ask`). Endpoints: `GET/POST /conf` (config), `POST /fw` (OTA update), `GET /` (web UI). `src/form.js` converts HTML form data to JSON for POST.

4. **Reset button** (`src/bin/esp32serial.rs:poll_reset`) — Monitors GPIO9; long press triggers factory reset.

5. **Pinger** (`src/bin/esp32serial.rs:pinger`) — Pings gateway every 5 minutes; reboots on failure.

## Key Modules

- **`src/config.rs`** — `MyConfig` struct with all settings (WiFi, IP, serial, API port). Persisted to NVS using `postcard` binary serialization with CRC32 validation.
- **`src/state.rs`** — `MyState` shared state (Arc + Pin). Contains config, OTA info, WiFi status, hardware pins (`MySerial`: UART1, GPIO0/TX, GPIO1/RX, GPIO8/LED).

## GPIO Pinout

| GPIO | Function |
|------|----------|
| 0    | UART1 TX |
| 1    | UART1 RX |
| 8    | Status LED |
| 9    | Reset button |

## Flash Partition Layout

Dual OTA slots (ota_0/ota_1, ~2MB each) with NVS for config storage. Defined in `partitions.csv`.

## Features

- `esp32c3` (default) — ESP32-C3 target
- `esp32s` — Alternative ESP32 variant
