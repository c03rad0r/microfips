//! Single-dependency ESP32 import for the microfips FIPS leaf node.
//!
//! `microfips-esp32-component` collapses the per-chip crates
//! ([`microfips_esp32`] for the ESP32-D0WD, [`microfips_esp32s3`] for the
//! ESP32-S3) plus the shared transport/protocol stack behind **one** cargo
//! dependency. Pick exactly one chip with a cargo feature:
//!
//! ```toml
//! [dependencies]
//! microfips-esp32-component = { version = "0.1", features = ["esp32"] }
//! # and, optionally, a transport:
//! #   features = ["esp32", "wifi"]
//! ```
//!
//! Then call the run function for your transport:
//!
//! ```no_run
//! # // host doctest only — real use is under an Xtensa/RISC-V ESP target.
//! # #[cfg(any(target_arch = "xtensa", target_arch = "riscv32"))]
//! use microfips_esp32_component as fips;
//! # #[cfg(any(target_arch = "xtensa", target_arch = "riscv32"))]
//! # #[esp_rtos::main]
//! # async fn main(_s: embassy_executor::Spawner) {
//! #   let p = esp_hal::init(esp_hal::Config::default());
//! #   fips::run_uart_node(p.GPIO2, p.UART0, p.GPIO1, p.GPIO3, p.RNG, p.ADC1).await;
//! # }
//! ```
//!
//! UART is always available. BLE GATT, BLE L2CAP, and WiFi are gated behind the
//! `ble`, `l2cap`, and `wifi` features respectively (see [`run`] for the full
//! matrix).
//!
//! # no_std
//! This crate is `#![no_std]` and is intended to be compiled for an ESP target
//! (`xtensa-esp32-none-elf`, `xtensa-esp32s3-none-elf`, or the RISC-V ESP32-C
//! family). It will not compile on a host target with a chip feature enabled.

#![no_std]

/// Exactly one chip feature must be enabled.
#[cfg(not(any(feature = "esp32", feature = "esp32s3")))]
compile_error!("microfips-esp32-component: enable exactly one chip feature — `esp32` or `esp32s3`");

/// The two chip features are mutually exclusive (they select conflicting
/// esp-hal / esp-radio / esp-rt1 features and would produce duplicate symbols).
#[cfg(all(feature = "esp32", feature = "esp32s3"))]
compile_error!(
    "microfips-esp32-component: features `esp32` and `esp32s3` are mutually exclusive — pick one"
);

// ---------------------------------------------------------------------------
// Chip-specific re-exports
// ---------------------------------------------------------------------------

/// Chip-specific run functions and binary entry points.
///
/// Re-exports the active per-chip crate's public surface: the `run_*_node`
/// async entry points, the transport helpers (`handler`, `node_info`, `led`,
/// `rng`, `stats`, `uart_transport`, and the BLE/L2CAP/WiFi transports behind
/// their respective features).
#[cfg(feature = "esp32")]
pub use microfips_esp32 as chip;

#[cfg(feature = "esp32s3")]
pub use microfips_esp32s3 as chip;

// Convenience flat re-exports of the most-used entry points so callers can write
// `microfips_esp32_component::run_uart_node` instead of
// `microfips_esp32_component::chip::run::run_uart_node`. Both chip crates
// re-export the transport run functions into their `run` module.
// Gated on a chip feature so the (already compile_error!'d) no-chip case doesn't
// also emit "unresolved name `chip`".
#[cfg(any(feature = "esp32", feature = "esp32s3"))]
pub use chip::run::run_uart_node;

/// ESP32-S3 only: USB-Serial-JTAG transport entry point.
#[cfg(feature = "esp32s3")]
pub use chip::run::run_usb_node;

#[cfg(all(any(feature = "esp32", feature = "esp32s3"), feature = "ble"))]
pub use chip::run::run_ble_node;

#[cfg(all(any(feature = "esp32", feature = "esp32s3"), feature = "l2cap"))]
pub use chip::run::run_l2cap_node;

#[cfg(all(any(feature = "esp32", feature = "esp32s3"), feature = "wifi"))]
pub use chip::run::run_wifi_node;

// ---------------------------------------------------------------------------
// Shared transport stack (chip-agnostic surface)
// ---------------------------------------------------------------------------

/// Shared ESP transport implementations and the run helper that drives the
/// [`microfips_protocol::node::Node`] from a [`microfips_protocol::Transport`].
///
/// Exposed so advanced users can build a custom transport and still reuse the
/// heap init, TRNG, LED, backoff, and demo-FSP wiring.
pub use microfips_esp_transport;

#[cfg(any(feature = "esp32", feature = "esp32s3"))]
pub use microfips_esp_transport::{handler, node_info, runner};

// ---------------------------------------------------------------------------
// Protocol surface (so consumers don't need a separate microfips-protocol dep)
// ---------------------------------------------------------------------------

/// The no_std FIPS protocol state machine: [`Node`] runtime, length-prefixed
/// [`Transport`] / [`FrameWriter`] / [`FrameReader`], peer policy, and MMP.
///
/// [`Node`]: microfips_protocol::node::Node
/// [`Transport`]: microfips_protocol::Transport
/// [`FrameWriter`]: microfips_protocol::FrameWriter
/// [`FrameReader`]: microfips_protocol::FrameReader
pub use microfips_protocol;

/// Core types: Noise primitives, FMP wire format, FSP session, identity.
pub use microfips_core;

#[cfg(feature = "service")]
/// Optional transport-neutral request/response service layer for app authors.
pub use microfips_service;
