# microfips-esp32-component

A **single-dependency** wrapper that makes the microfips FIPS leaf node importable
on ESP32 / ESP32-S3 without juggling the six underlying crates.

```toml
[dependencies]
microfips-esp32-component = { version = "0.1", features = ["esp32"] }
# add a transport only if you need BLE / L2CAP / WiFi (UART is always available):
# features = ["esp32", "wifi"]
```

One import, one call:

```rust
use microfips_esp32_component as fips;

#[esp_rtos::main]
async fn main(spawner: embassy_executor::Spawner) {
    let p = esp_hal::init(esp_hal::Config::default());
    /* ...start esp-rtos... */
    fips::run_uart_node(p.GPIO2, p.UART0, p.GPIO1, p.GPIO3, p.RNG, p.ADC1).await;
}
```

It re-exports the chip-specific crates ([`microfips-esp32`] for the D0WD,
[`microfips-esp32s3`] for the S3), the shared transport stack
([`microfips-esp-transport`]: UART / USB / BLE GATT / BLE L2CAP / WiFi), and the
no_std protocol surface ([`microfips-protocol`] `Node` + [`microfips-core`]
Noise/FMP/FSP) — so app authors never depend on those names directly.

> ## ⛔ Do not publish yet — release blocker
> Current upstream `main` is mid-migration from Noise **IK** (FMP v0) to Noise
> **XX** (FMP v1). `microfips-core`/`microfips-protocol` wire constants already
> declare XX sizes (`MSG1=41`, `MSG2=118`, `MSG3=85`), but
> `microfips-protocol/src/node.rs` still drives the **2-message IK** flow
> (`NoiseIkInitiator`/`NoiseIkResponder`, 114-byte msg1). Two unit tests fail.
> A firmware built today advertises `FMP_VERSION=1` while handshaking as IK, so
> it interoperates with **neither** an IK nor an XX peer. See
> [`docs/PROTOCOL_AUDIT.md`](docs/PROTOCOL_AUDIT.md). Phase 6 (publish) is
> blocked until `node.rs` is migrated and `cargo test -p microfips-protocol
> --features std` is green.

---

## Feature matrix

Pick **exactly one** chip feature (enforced by `compile_error!`), then optionally
one transport. UART needs no feature.

| Chip feature | Target triple | Crate re-exported |
|---|---|---|
| `esp32` | `xtensa-esp32-none-elf` | `microfips-esp32` (D0WD) |
| `esp32s3` | `xtensa-esp32s3-none-elf` | `microfips-esp32s3` |

| Transport feature | Entry point | Notes |
|---|---|---|
| _(none)_ — UART | `run_uart_node` | always available, serial-bridge path |
| `ble` | `run_ble_node` | BLE GATT via host bridge |
| `l2cap` | `run_l2cap_node` | direct BLE L2CAP CoC to a local FIPS daemon (PSM 133) |
| `wifi` | `run_wifi_node` | direct UDP to FIPS VPS |
| `esp32s3` only | `run_usb_node` | USB-Serial-JTAG |

## Getting started (pure cargo — recommended)

1. Install the ESP Rust toolchain:
   ```sh
   rustup component add rust-src
   cargo install espflash
   # Xtensa needs the esp toolchain (espup), or use the prebuilt image.
   ```
2. New project:
   ```sh
   cargo new uart-leaf && cd uart-leaf
   cargo add microfips-esp32-component --features esp32
   cargo add esp-hal@1.1.0 --features esp32,unstable
   cargo add esp-rtos@0.3.0 --features esp32,embassy,esp-alloc,esp-radio
   cargo add esp-bootloader-esp-idf@0.5 --features esp32
   cargo add embassy-executor@0.10
   ```
3. Write `main.rs` (copy [`examples/uart-leaf/src/main.rs`](examples/uart-leaf/src/main.rs)).
4. Build & flash:
   ```sh
   cargo +esp run --release            # builds + flashes via espflash
   ```

A complete working example lives in [`examples/uart-leaf/`](examples/uart-leaf/).

## PlatformIO

The [`platformio/`](platformio/) directory is a self-contained PlatformIO
library. esp-hal firmware is a complete no_std **binary**, so this wrapper does
not pretend to be a C component — `platformio/scripts/build_rust.py` runs
`cargo +esp build` to produce the firmware ELF for the active board, and
PlatformIO's upload step flashes it.

```ini
; platformio.ini
[env:esp32dev]
platform = espressif32
board = esp32dev
framework = espidf
lib_deps =
  microfips-esp32
build_flags =
  -DMICROFIPS_TRANSPORT=wifi   ; or ble / l2cap / uart (default)
```

`pio run` builds the Rust firmware; `pio run -t upload` flashes it. Requires the
`esp` toolchain + Xtensa GCC on PATH (set `MICROFIPS_CARGO` /
`MICROFIPS_RUST_TOOLCHAIN` to override).

## ESP-IDF

The [`esp-idf/`](esp-idf/) directory is an ESP-IDF component providing a CMake
function that builds the Rust firmware:

```cmake
# your project's CMakeLists.txt
idf_component_register(SRCS "main.c" INCLUDE_DIRS ".")
microfips_esp32_firmware(TRANSPORT wifi)
```

Install locally (pre-publish):
```sh
idf.py add-dependency "amperstrand/microfips-esp32^0.1.0"
```

## Layout

```
microfips-esp32-component/
├── Cargo.toml              # the wrapper crate (Phase 2)
├── src/lib.rs              # chip re-exports + compile_error guards
├── examples/uart-leaf/     # minimal end-to-end example (Phase 5)
├── platformio/             # PlatformIO library wrapper (Phase 3)
│   ├── library.json
│   ├── library.properties
│   └── scripts/build_rust.py
├── esp-idf/                # ESP-IDF component (Phase 4)
│   ├── idf_component.yml
│   └── CMakeLists.txt
└── docs/PROTOCOL_AUDIT.md  # Phase 1 wire-protocol audit
```

## Integrating into the microfips repo

This wrapper is designed to live at `crates/microfips-esp32-component/` inside
the microfips workspace. To adopt it:

1. Copy this directory to `crates/microfips-esp32-component/`.
2. Add `"crates/microfips-esp32-component"` to the `members` list in the root
   `Cargo.toml`.
3. The path deps (`../microfips-core`, …) then resolve against the workspace.
   Verified: `cargo metadata` succeeds and `cargo build -p
   microfips-esp32-component --features esp32` resolves the full graph (it only
   stops at `xtensa_lx` when built on a host without the Xtensa target — same as
   the in-tree `microfips-esp32` crate).
4. At publish time, run `cargo release` — it rewrites the path deps to version
   requirements so the published crate is a true single-dependency pull from
   crates.io. Delete the `[patch.crates-io]` block in
   `examples/uart-leaf/Cargo.toml` once the microfips crates are published.

## What is *not* here (honest scope)

- **C-ABI interop** — esp-hal firmware is a no_std binary with its own async
  `main`; there is no C-callable library API today. The PlatformIO/ESP-IDF
  wrappers build and flash the Rust **binary**. A C-ABI shim is a documented
  future extension, not a current capability.
- **Cross-compilation was not exercised in the packaging environment** (no
  Xtensa target installed). Resolution + feature unification are verified via
  `cargo metadata`/`cargo build` up to the architecture boundary; full firmware
  builds must be run in a properly tooled environment.
- **The Noise XX migration of `node.rs`** is out of scope for packaging and is
  tracked as a separate task (see top-of-file blocker).

## License

MIT OR Apache-2.0, matching the microfips workspace.
