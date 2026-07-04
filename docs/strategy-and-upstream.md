# Project Strategy & Upstream Relationship

> Created: 2026-07-04 | Signal group: microFIPS-esp32

## Strategy

**Maintain microFIPS as a standalone, hardened implementation.** Do NOT attempt
fips-core extraction until upstream FIPS v2 protocols stabilize and the maintainer
(jmcorgan) provides guidance on integration.

### What the upstream maintainer said (2026-07-04)

After opening issue #122 (fips-core extraction proposal), jmcorgan responded:

1. "Start with what the problems are before suggesting the solution"
2. Protocol state machines are tightly integrated with the tokio runtime —
   you can't just import structs without rewriting state machines from scratch
3. "A very long timeframe methodical evolution, not a week-long changeover"
4. v2 protocols are in development — extraction must wait until those stabilize
5. He is writing **detailed v2 protocol specs** to enable independent interop
6. "Small pieces at a time that move things in the right direction"
7. Sympathetic to ESP32 port, but "a lot more work than appears on the surface"

### What we do now

- Maintain microFIPS as the MCU FIPS implementation
- Test interoperability with upstream FIPS at each release
- Watch for v2 protocol specs from jmcorgan
- When small runtime-agnostic opportunities arise upstream, contribute them
- Revisit extraction after v2 stabilizes
- Next in-person discussion: Madeira meetup

### Issue #122

Closed 2026-07-02 with explanation referencing the maintainer conversation.
Title: "Proposal: Extract protocol layer into a standalone fips-core crate
(no_std-compatible)"

---

## Current Status

- All core milestones (M0–M9, M11): COMPLETE
- Noise XX handshake migration: DONE (2026-07-04, branch feat/noise-xx-handshake)
- PlatformIO component wrapper: CREATED (microfips-esp32-component)
- Wire-protocol compat audit: WRITTEN (340-line doc, in worktree)

### Build Matrix (all compile-tested)

| Transport | ESP32-D0WD | ESP32-S3 | Hardware Verified |
|-----------|-----------|----------|-------------------|
| UART | YES | YES | STM32 UART verified |
| BLE GATT | YES | YES | D0WD BLE bridge verified |
| BLE L2CAP | YES | YES | Both to FIPS verified |
| WiFi | YES | YES | Both verified |

### Parity with upstream FIPS: 95%+ wire-level

See `docs/fips-microfips-parity.md` for the full module-by-module mapping.

---

## Key Learnings

### What worked
- Full FIPS protocol stack on microcontrollers — leaf nodes in real FIPS mesh
- Noise IK handshake verified with VPS (STM32F746, 2026-05-04)
- Noise XX migration completed (matches upstream v2 direction)
- Multi-transport: UART, BLE GATT, BLE L2CAP, WiFi all hardware-verified
- PlatformIO wrapper enables ESP-IDF/Arduino ecosystem access
- Embassy async HAL works well for ESP32 embedded Rust
- Compat audit identifies exact wire-protocol deltas

### What didn't work / blockers
- fips-core extraction premature — upstream tokio coupling prevents clean separation
- Upstream FIPS getting heavier — added NIM, more ethernet adapters
- Translation burden — upstream changes require manual porting
- No ESP32 build target in upstream FIPS yet (deferred indefinitely)
- FIPS v2 protocols incoming — breaking changes will require microFIPS rework

---

## Hardware

| Board | Chip | Architecture | Role |
|-------|------|-------------|------|
| ESP32-D0WD | ESP32 | Xtensa LX6 | Primary target, all transports verified |
| ESP32-S3 | ESP32-S3 | Xtensa LX7 | Secondary target, all transports verified |
| ESP32-C3 Mini V1 (x20) | ESP32-C3 | RISC-V | NOT yet a microFIPS target (balloon project uses these) |
| STM32F469I-DISCO | STM32F4 | Cortex-M4 | Primary STM32 target |
| STM32F746G-DISCO | STM32F7 | Cortex-M7 | Hardware-verified 2026-05-04 |

**Note**: Adding ESP32-C3 (RISC-V) support would require a new target configuration.
The C3 is used in the balloon project for LoRa firmware but NOT for microFIPS.

---

## Repositories

| Repo | Remote | Role |
|------|--------|------|
| ~/repos/microfips | origin: Amperstrand/microfips | microFIPS source |
| ~/repos/microfips | fork: c03rad0r/microfips | Our fork for PRs |
| ~/repos/fips | origin: jmcorgan/fips | Upstream FIPS (reference only) |

### VPS (FIPS test node)
```
VPS_HOST=orangeclaw.dns4sats.xyz
VPS_USER=routstr
```
FIPS binds `0.0.0.0:2121`, MCU peers at `127.0.0.1:31337` (STM32) / `31338` (ESP32)

---

## Kanban & Plans

- **Kanban board**: `microfips` (hermes kanban --board microfips ls)
- **Master plan**: ~/plans/microfips-master-plan.md
- **Broader TollGate/FIPS plan**: ~/plans/tollgate-fips-master-plan.md
- **Deferred extraction proposal**: ~/plans/fips-no-std-core-extraction.md
- **Compat audit** (worktree): ~/worktrees/feature-microfips-compat-audit/docs/fips-microfips-compat.md

---

## Glossary

| Term | Meaning |
|------|---------|
| FIPS | Free Internetworking Peering System (jmcorgan's mesh network) |
| microFIPS | MCU implementation of FIPS leaf node (this project) |
| FMP | FIPS Message Protocol (link-layer framing) |
| FSP | FIPS Session Protocol (session-layer, XK handshake) |
| MMP | Mesh Metrics Protocol (EWMA, jitter, SRTT) |
| Noise IK | Noise handshake pattern (initiator static key known) |
| Noise XX | Noise handshake pattern (both static keys exchanged interactively) |
| no_std | Rust embedded target without standard library |
| Embassy | Async embedded HAL framework for Rust |
| PlatformIO | Build system/IDE for embedded development |
| jmcorgan | Johnathan Corgan — upstream FIPS maintainer |
