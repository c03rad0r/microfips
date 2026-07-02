//! Minimal ESP32-D0WD FIPS leaf node using the `microfips-esp32-component`
//! single-dependency wrapper.
//!
//! This is the UART/serial-bridge transport. Flash with:
//!
//! ```sh
//! cargo +esp run --release              # via espflash
//! # or
//! cargo run --release --features esp32  # if feature-gated
//! ```
//!
//! Wiring: GPIO1 (TX) / GPIO3 (RX) to a USB-serial adapter; the host runs
//! `serial_udp_bridge.py` to forward framed traffic to the FIPS VPS.
//! GPIO2 drives the status LED.

#![no_std]
#![no_main]

// App descriptor + panic LED-blink, same as the in-tree firmware bins.
esp_bootloader_esp_idf::esp_app_desc!();
microfips_esp32_component::microfips_esp_transport::panic_blink!();

/// The single import. Everything else (chip run fn, protocol stack, transport)
/// comes through this one dependency.
use microfips_esp32_component as fips;

#[esp_rtos::main]
async fn main(_spawner: embassy_executor::Spawner) {
    let peripherals = esp_hal::init(esp_hal::Config::default());

    // Start the embassy runtime the same way the in-tree bins do.
    let sw_ints =
        esp_hal::interrupt::software::SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    let timg0 = esp_hal::timer::timg::TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(timg0.timer0, sw_ints.software_interrupt0);

    // One call — the wrapper selected the ESP32 chip + UART transport.
    fips::run_uart_node(
        peripherals.GPIO2,
        peripherals.UART0,
        peripherals.GPIO1,
        peripherals.GPIO3,
        peripherals.RNG,
        peripherals.ADC1,
    )
    .await;
}
