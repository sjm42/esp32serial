// bin/esp32serial.rs

#![warn(clippy::large_futures)]

use esp_idf_hal::{
    delay::FreeRtos,
    gpio::{AnyInputPin, Input, InputPin, OutputPin, PinDriver},
    prelude::Peripherals,
};
use esp_idf_svc::{
    eventloop::EspSystemEventLoop, nvs, ota::EspOta, ping, timer::EspTaskTimerService,
    wifi::WifiDriver,
};
use esp_idf_sys::esp;

use esp32serial::*;

#[cfg(all(feature = "esp32-c3", feature = "esp-wroom-32"))]
compile_error!("Select only one hardware feature: `esp32-c3` or `esp-wroom-32`");
#[cfg(not(any(feature = "esp32-c3", feature = "esp-wroom-32")))]
compile_error!("Select a hardware feature: `esp32-c3` or `esp-wroom-32`");

// DANGER! DO NOT USE THIS until esp-idf-svc supports newer versions of ESP-IDF
// - until then, only up to esp-idf 5.3.2 is supported with esp_app_desc!()
// Without the macro usage up to esp-idf v5.4 is supported.
// ESP-IDF version 5.5 requires updated esp-idf-svc crate to be released.

// use esp_idf_sys::esp_app_desc;
// esp_app_desc!();

const CONFIG_RESET_COUNT: i32 = 9;

fn main() -> anyhow::Result<()> {
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    // eventfd is needed by our mio poll implementation.  Note you should set max_fds
    // higher if you have other code that may need eventfd.

    #[allow(clippy::needless_update)]
    let config = esp_idf_sys::esp_vfs_eventfd_config_t {
        max_fds: 1,
        ..Default::default()
    };
    esp! { unsafe { esp_idf_sys::esp_vfs_eventfd_register(&config) } }?;

    // comment or uncomment these, if you encounter this boot error:
    // E (439) esp_image: invalid segment length 0xXXXX
    // this means that the code size is not 32bit aligned
    // and any small change to the code will likely fix it.
    info!("Hello.");
    info!("Starting up, firmware version {}", FW_VERSION);
    let ota_slot = {
        let mut ota = EspOta::new()?;
        let running_slot = ota.get_running_slot()?;
        ota.mark_running_slot_valid()?;
        let ota_slot = format!("{} ({:?})", &running_slot.label, running_slot.state);
        info!("OTA slot: {ota_slot}");
        ota_slot
    };

    let sysloop = EspSystemEventLoop::take()?;
    let timer = EspTaskTimerService::new()?;
    let nvs_default_partition = nvs::EspDefaultNvsPartition::take()?;

    let ns = env!("CARGO_BIN_NAME");
    let mut nvs = match nvs::EspNvs::new(nvs_default_partition.clone(), ns, true) {
        Ok(nvs) => {
            info!("Got namespace {ns:?} from default partition");
            nvs
        }
        Err(e) => panic!("Could not get namespace {ns}: {e:?}"),
    };

    let config = match MyConfig::from_nvs(&mut nvs) {
        None => {
            error!("Could not read nvs config, using defaults");
            let c = MyConfig::default();
            c.to_nvs(&mut nvs)?;
            info!("Successfully saved default config to nvs.");
            c
        }

        // using settings saved on nvs if we could find them
        Some(c) => c,
    };
    info!("My config:\n{config:#?}");

    let peripherals = Peripherals::take().unwrap();
    let pins = peripherals.pins;

    #[cfg(feature = "esp32-c3")]
    let (tx, rx, led, button) = (
        pins.gpio0.downgrade_output(),
        pins.gpio1.downgrade_input(),
        pins.gpio8.downgrade_output(),
        PinDriver::input(pins.gpio9.downgrade_input())?,
    );

    #[cfg(feature = "esp-wroom-32")]
    let (tx, rx, led, button) = (
        pins.gpio17.downgrade_output(),
        pins.gpio16.downgrade_input(),
        pins.gpio2.downgrade_output(),
        PinDriver::input(pins.gpio0.downgrade_input())?,
    );

    let uart = peripherals.uart1;

    let wifi_driver = WifiDriver::new(
        peripherals.modem,
        sysloop.clone(),
        Some(nvs_default_partition),
    )?;

    let state = Box::pin(MyState::new(
        config,
        ota_slot,
        nvs,
        MySerial { uart, tx, rx, led },
    ));
    let shared_state = Arc::new(state);

    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(Box::pin(async move {
            let wifi_loop = WifiLoop {
                state: shared_state.clone(),
                wifi: None,
            };

            info!("Entering main loop...");
            tokio::select! {
                _ = Box::pin(poll_reset(shared_state.clone(), button)) => { error!("poll_reset() ended."); }
                _ = Box::pin(run_api_server(shared_state.clone())) => { error!("run_api_server() ended."); }
                _ = Box::pin(run_serial(shared_state.clone())) => { error!("run_serial() ended."); }
                _ = Box::pin(wifi_loop.run(wifi_driver, sysloop, timer)) => { error!("wifi_loop() ended."); }
                _ = Box::pin(pinger(shared_state.clone())) => { error!("pinger() ended."); }

            };
        }));

    // not actually returning from main() but we reboot instead!
    info!("main() finished, reboot.");
    FreeRtos::delay_ms(3000);
    esp_idf_hal::reset::restart();
}

async fn poll_reset(
    mut state: Arc<Pin<Box<MyState>>>,
    button: PinDriver<'_, AnyInputPin, Input>,
) -> anyhow::Result<()> {
    loop {
        sleep(Duration::from_secs(2)).await;

        if *state.restart.read().await {
            esp_idf_hal::reset::restart();
        }

        if button.is_low() {
            Box::pin(reset_button(&mut state, &button)).await?;
        }
    }
}

async fn reset_button<'a>(
    state: &mut Arc<std::pin::Pin<Box<MyState>>>,
    button: &PinDriver<'a, AnyInputPin, Input>,
) -> anyhow::Result<()> {
    let mut reset_cnt = CONFIG_RESET_COUNT;

    while button.is_low() {
        // button is pressed and kept down, countdown and factory reset if reach zero
        let msg = format!("Reset? {reset_cnt}");
        error!("{msg}");

        if reset_cnt == 0 {
            // okay do factory reset now
            error!("Factory resetting...");

            let new_config = MyConfig::default();
            new_config.to_nvs(&mut *state.nvs.write().await)?;
            sleep(Duration::from_millis(2000)).await;
            esp_idf_hal::reset::restart();
        }

        reset_cnt -= 1;
        sleep(Duration::from_millis(500)).await;
        continue;
    }
    Ok(())
}

async fn pinger(state: Arc<std::pin::Pin<Box<MyState>>>) -> anyhow::Result<()> {
    loop {
        sleep(Duration::from_secs(300)).await;

        if let Some(ping_ip) = *state.ping_ip.read().await {
            let if_idx = *state.if_index.read().await;
            if if_idx > 0 {
                tracing::log::info!("Starting ping {ping_ip} (if_idx {if_idx})");
                let conf = ping::Configuration {
                    count: 2,
                    interval: Duration::from_millis(500),
                    timeout: Duration::from_millis(200),
                    data_size: 64,
                    tos: 0,
                };
                let mut ping = ping::EspPing::new(if_idx);
                let res = ping.ping(ping_ip, &conf)?;
                tracing::log::info!("Pinger result: {res:?}");
                if res.received == 0 {
                    tracing::log::error!("Ping failed, rebooting.");
                    sleep(Duration::from_millis(2000)).await;
                    esp_idf_hal::reset::restart();
                }
            } else {
                tracing::log::error!("No if_index. wat?");
            }
        }
    }
}

// EOF
