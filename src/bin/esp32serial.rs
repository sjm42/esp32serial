// bin/esp32serial.rs
#![warn(clippy::large_futures)]

use esp_idf_hal::{
    delay::FreeRtos,
    gpio::{InputPin, OutputPin},
    prelude::Peripherals,
};
use esp_idf_svc::{
    eventloop::EspSystemEventLoop, nvs, timer::EspTaskTimerService, wifi::WifiDriver,
};
use esp_idf_sys::{self as _, esp, esp_app_desc};
use log::*;
use std::{net, sync::Arc};
use tokio::sync::RwLock;

use esp32serial::*;

esp_app_desc!();

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
    info!("Starting up.");

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

    #[cfg(feature = "reset_settings")]
    let config = {
        let c = MyConfig::default();
        c.to_nvs(&mut nvs)?;
        c
    };

    #[cfg(not(feature = "reset_settings"))]
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
    let led = pins.gpio8.downgrade_output();
    let tx = pins.gpio0.downgrade_output();
    let rx = pins.gpio1.downgrade_input();
    let uart = peripherals.uart1;

    let wifidriver = WifiDriver::new(
        peripherals.modem,
        sysloop.clone(),
        Some(nvs_default_partition),
    )?;

    let state = Box::pin(MyState {
        config: RwLock::new(config),
        cnt: RwLock::new(0),
        wifi_up: RwLock::new(false),
        ip_addr: RwLock::new(net::Ipv4Addr::new(0, 0, 0, 0)),
        myid: RwLock::new("esp32temp".into()),
        nvs: RwLock::new(nvs),
        reset: RwLock::new(false),
        serial: RwLock::new(Some(MySerial { uart, tx, rx, led })),
    });
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
                _ = Box::pin(run_api_server(shared_state.clone())) => {}
                _ = Box::pin(run_serial(shared_state.clone())) => {}
                _ = Box::pin(wifi_loop.run(wifidriver, sysloop, timer)) => {}
            };
        }));

    // not actually returing from main() but we reboot instead
    info!("main() finished, reboot.");
    FreeRtos::delay_ms(3000);
    esp_idf_hal::reset::restart();

    Ok(())
}

// EOF
