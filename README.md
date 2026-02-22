# esp32serial

A serial port TCP server for ESP32, written in Rust. It bridges a UART serial port to TCP/IP, allowing multiple remote clients to connect over the network and communicate with a serial device attached to the ESP32.

## Features

- **TCP-to-UART bridge** — Raw TCP connections on a configurable port (default 23) are bridged to UART1. Multiple clients can connect simultaneously; serial data is broadcast to all.
- **Bidirectional** — Clients can both read and write serial data (write can be disabled in config).
- **Web configuration UI** — Built-in HTTP server with a browser-based settings page for WiFi, IP, and serial parameters (Askama template + embedded JS/CSS assets).
- **Persistent configuration** — Settings are stored in NVS (non-volatile storage) and survive reboots.
- **OTA firmware updates** — Upload new firmware via the web UI by providing a URL.
- **WPA2-Enterprise support** — Connects to both WPA2-Personal and WPA2-Enterprise (PEAP) networks.
- **Static IP or DHCP** — Configurable IPv4 networking with custom DNS.
- **Factory reset** — Hold the reset button for ~5 seconds to restore default settings (pin depends on target hardware).
- **Network health monitoring** — Pings the gateway every 5 minutes and reboots on failure.

## Hardware

Supports two target variants selected via Cargo features:

- **`esp32-c3`** (default, RISC-V)
- **`esp-wroom-32`** (Xtensa ESP32 / ESP-WROOM-32)

Backward-compatible aliases are still accepted:

- `esp32c3` -> `esp32-c3`
- `esp32s` -> `esp-wroom-32`

### GPIO pinout (`esp32-c3`, default)

| GPIO | Function     |
|------|--------------|
| 0    | UART1 TX     |
| 1    | UART1 RX     |
| 8    | Status LED   |
| 9    | Reset button |

### GPIO pinout (`esp-wroom-32`)

| GPIO | Function     |
|------|--------------|
| 17   | UART1 TX     |
| 16   | UART1 RX     |
| 2    | Status LED   |
| 0    | Reset button |

## Building and flashing

Requires Rust nightly (`rust-toolchain.toml`) and the [ESP-IDF](https://github.com/espressif/esp-idf) build environment. Install [espflash](https://github.com/esp-rs/espflash) for flashing/image creation.

```bash
# Build release firmware (default target: esp32-c3)
cargo build --release

# Build, flash, and monitor (ESP32-C3)
./flash_c3

# Build, flash, and monitor (ESP-WROOM-32 / Xtensa)
./flash_wroom32

# Create OTA firmware images
./make_ota_image_c3
./make_ota_image_wroom32

# Manual WROOM-32 build (equivalent feature/target flags)
MCU=esp32 cargo +esp build -r --target xtensa-esp32-espidf --no-default-features --features esp-wroom-32
```

Default WiFi credentials can be overridden at build time via `WIFI_SSID` and `WIFI_PASS`.

## Configuration

Runtime settings are configurable through the web UI served at `http://<device-ip>/` on port **80** (fixed in code).

| Setting             | Default       | Description                         |
|---------------------|---------------|-------------------------------------|
| WiFi SSID           | internet      | Wireless network name               |
| WiFi password       | password      | Wireless network password           |
| WPA2-Enterprise     | off           | Enable EAP authentication           |
| WiFi username       | (empty)       | WPA2-Enterprise username/identity   |
| DHCP                | on            | Use DHCP for IPv4 addressing        |
| Baud rate           | 9600          | UART serial speed                   |
| Serial TCP port     | 23            | TCP port for serial connections     |
| Serial write        | on            | Allow TCP clients to write to UART  |

Configuration is persisted to NVS using [postcard](https://github.com/jamesmunns/postcard) binary serialization with CRC32 integrity validation.

### REST API

| Endpoint      | Method | Description                                      |
|---------------|--------|--------------------------------------------------|
| `/`           | GET    | Web configuration UI                             |
| `/form.js`    | GET    | Embedded UI JavaScript                           |
| `/index.css`  | GET    | Embedded UI stylesheet                           |
| `/favicon.ico`| GET    | Embedded favicon                                 |
| `/conf`       | GET    | Current configuration as JSON                    |
| `/conf`       | POST   | Update configuration (JSON body)                 |
| `/reset_conf` | GET    | Reset to factory defaults                        |
| `/fw`         | POST   | OTA firmware update (HTML form field `url`)      |

## Architecture

The application runs on a single-threaded [Tokio](https://tokio.rs/) async runtime with five concurrent tasks managed by `tokio::select!`:

1. **Serial bridge** (`serial.rs`) — Opens UART1 with the configured baud rate. Reads incoming serial data and broadcasts it to all connected TCP clients via a `tokio::sync::broadcast` channel. Client-to-serial writes flow through an `mpsc` channel. Each TCP client is handled by a spawned async task. The status LED toggles on serial activity.

2. **WiFi manager** (`wifi.rs`) — Configures and maintains the WiFi connection with automatic reconnection. Supports WPA2-Personal, WPA2-Enterprise (via raw esp-idf-sys EAP calls), and open networks. Sets the device hostname to `esp32serial-<MAC>`.

3. **API server** (`apiserver.rs`) — An [Axum](https://github.com/tokio-rs/axum) HTTP server bound to port `80` (`DEFAULT_API_PORT`) that serves the configuration web UI (rendered with [Askama](https://github.com/djc/askama) templates from `templates/index.html.ask`). Static assets (`form.js`, `index.css`, `favicon.ico`) are embedded in the binary via `include_bytes!`. Configuration changes trigger a device reboot.

4. **Reset button monitor** (`bin/esp32serial.rs`) — Polls the target-specific reset pin (`GPIO9` on `esp32-c3`, `GPIO0` on `esp-wroom-32`) every 2 seconds. When held down, counts down from 9 in 500ms intervals; reaching zero triggers a factory reset.

5. **Gateway pinger** (`bin/esp32serial.rs`) — Every 5 minutes, pings the default gateway using `esp_idf_svc::ping`. Reboots the device if the ping fails, providing automatic recovery from network issues.

### Shared state

Application state (`state.rs`) is wrapped in `Arc<Pin<Box<MyState>>>` and shared across all tasks. Mutable fields (WiFi status, IP address, NVS handle, restart flag) use `tokio::sync::RwLock`. The API request counter uses `AtomicU32`.

### Flash partition layout

Dual OTA slots (~2MB each) enable safe firmware updates with rollback. Defined in `partitions.csv`:

| Partition | Size   | Purpose                    |
|-----------|--------|----------------------------|
| nvs       | 16 KB  | Configuration storage      |
| otadata   | 8 KB   | OTA boot selection         |
| ota_0     | 1984 KB| Firmware slot A            |
| ota_1     | 1984 KB| Firmware slot B            |

## License

MIT
