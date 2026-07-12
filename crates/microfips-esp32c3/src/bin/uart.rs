//! ESP32-C3 UART binary entry point.

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use esp_hal::{
    gpio::{Input, Output},
    peripherals::{Peripherals, UART0, GPIO2, GPIO20, GPIO21, RNG, ADC1},
    uart,
};
use esp_hal::clock::CpuClock;
use static_cell::StaticCell;

use microfips_esp32c3::*;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM;
    let clocks = esp_hal::clock::ClockControl::configure(system, CpuClock::max(), esp_hal::clock::ApbClock::max())
        .freeze();

    let init = esp_hal::init::Init {
        clocks,
        ..Default::default()
    };
    esp_hal::init::init(init);

    // UART on GPIO20 (TX) and GPIO21 (RX) for ESP32-C3
    let gpio20 = peripherals.GPIO20;
    let gpio21 = peripherals.GPIO21;
    let gpio2 = peripherals.GPIO2;  // LED
    let uart0 = peripherals.UART0;
    let rng_periph = peripherals.RNG;
    let adc1 = peripherals.ADC1;

    run_uart_node(gpio2, uart0, gpio20, gpio21, rng_periph, adc1).await;
}
