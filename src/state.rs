// state.rs

use std::net;

use esp_idf_hal::{gpio::*, uart::UART1};
use esp_idf_svc::nvs;
use std::net::Ipv4Addr;
use tokio::sync::RwLock;

use crate::*;

pub struct MySerial {
    pub uart: UART1,
    pub tx: AnyOutputPin,
    pub rx: AnyInputPin,
    pub led: AnyOutputPin,
}
unsafe impl Sync for MySerial {}

pub struct MyState {
    pub config: MyConfig,
    pub api_cnt: RwLock<u64>,
    pub nvs: RwLock<nvs::EspNvs<nvs::NvsDefault>>,
    pub wifi_up: RwLock<bool>,
    pub if_index: RwLock<u32>,
    pub ip_addr: RwLock<Ipv4Addr>,
    pub ping_ip: RwLock<Option<Ipv4Addr>>,
    pub myid: RwLock<String>,
    pub restart: RwLock<bool>,
    pub serial: RwLock<Option<MySerial>>,
}

impl MyState {
    pub fn new(config: MyConfig, nvs: nvs::EspNvs<nvs::NvsDefault>, serial: MySerial) -> Self {
        MyState {
            config,
            api_cnt: RwLock::new(0),
            nvs: RwLock::new(nvs),
            wifi_up: RwLock::new(false),
            if_index: RwLock::new(0),
            ip_addr: RwLock::new(net::Ipv4Addr::new(0, 0, 0, 0)),
            ping_ip: RwLock::new(None),
            myid: RwLock::new("esp32clock".into()),
            restart: RwLock::new(false),
            serial: RwLock::new(Some(serial)),
        }
    }
}

// EOF
