//! ESP32-C3 firmware: chip-specific run functions and binary entry points.

#![no_std]

pub mod run;
pub use microfips_esp_transport::{handler, node_info};

pub use microfips_esp_transport::{led, rng, stats, uart_transport};

#[cfg(any(feature = "ble", feature = "l2cap", feature = "wifi", feature = "esp-now"))]
pub use microfips_esp_transport::control;
#[cfg(any(feature = "ble", feature = "l2cap", feature = "wifi", feature = "esp-now"))]
pub use microfips_esp_transport::logger;

#[cfg(feature = "ble")]
pub use microfips_esp_transport::ble_transport;

#[cfg(feature = "l2cap")]
pub use microfips_esp_transport::l2cap_transport;

#[cfg(feature = "esp-now")]
pub use microfips_esp_transport::esp_now_transport;
