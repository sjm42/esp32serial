// state.rs

use crate::*;

use esp_idf_hal::{gpio::*, uart::UART1};
use esp_idf_svc::nvs;
use std::net::Ipv4Addr;
use tokio::sync::RwLock;

pub struct MySerial {
    pub uart: UART1,
    pub tx: AnyOutputPin,
    pub rx: AnyInputPin,
    pub led: AnyOutputPin,
}
unsafe impl Sync for MySerial {}

pub struct MyState {
    pub config: RwLock<MyConfig>,
    pub cnt: RwLock<u64>,
    pub wifi_up: RwLock<bool>,
    pub ip_addr: RwLock<Ipv4Addr>,
    pub myid: RwLock<String>,
    pub nvs: RwLock<nvs::EspNvs<nvs::NvsDefault>>,
    pub reset: RwLock<bool>,
    pub serial: RwLock<Option<MySerial>>,
}

// EOF
