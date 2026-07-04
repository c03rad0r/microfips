# microFIPS — Master Plan

> Created: 2026-07-04 | Signal group: microFIPS-esp32

## Structural Index
- [Strategy](#strategy)
- [Current State](#current-state)
- [What Worked](#what-worked)
- [What Didn't / Blockers](#what-didnt--blockers)
- [Action Items](#action-items)
- [Relationship with Upstream FIPS](#relationship-with-upstream-fips)
- [Key Files & Repos](#key-files--repos)
- [Hardware](#hardware)
- [Glossary](#glossary)

---

## Strategy

**Maintain microFIPS as a standalone, hardened implementation.** Do NOT attempt
fips-core extraction until upstream FIPS v2 protocols stabilize and the maintainer
(jmcorgan) provides guidance on integration.

The upstream maintainer's guidance (2026-07-04):
- Protocol state machines are tightly coupled to tokio runtime — extraction is
  not a quick refactor, it's "a very long timeframe methodical evolution"
- v2 protocols are in development; extraction must wait until those stabilize
- jmcorgan is writing detailed v2 protocol specs that will enable independent
  implementations to interoperate
- Small pieces at a time — not a discrete event
- More discussion at Madeira in-person meetup

**Our role**: Keep microFIPS healthy, prove interoperability, be ready to
contribute small runtime-agnostic pieces upstream when asked.

---

## Current State

**Repo**: ~/repos/microfips (origin: Amperstrand/microfips, fork: c03rad0r/microfips)
**Active branch**: feat/noise-xx-handshake (committed 2026-07-04)
**Language**: Rust (Embassy async HAL, no_std targets)
**Targets**: ESP32-D0WD, ESP32-S3, STM32F469I-DISCO, STM32F746G-DISCO

### Milestones
- M0–M9, M11: ALL COMPLETE
- End-to-end encrypted comms proven across all transports
- Noise XX handshake migration: DONE (today) — matches upstream direction
- PlatformIO component wrapper: CREATED (microfips-esp32-component)
- Wire-protocol compat audit: WRITTEN (340 lines, in worktree)

### Build Matrix (all compile-tested)
| Transport | ESP32-D0WD | ESP32-S3 | Hardware Verified |
|-----------|-----------|----------|-------------------|
| UART | YES | YES | STM32 UART verified |
| BLE GATT | YES | YES | D0WD BLE bridge verified |
| BLE L2CAP | YES | YES | Both to FIPS verified |
| WiFi | YES | YES | Both verified |

### Workspace Crates (15)
microfips-core, microfips-link, microfips-protocol, microfips-service,
microfips-http-demo, microfips-http-test, microfips-sim, microfips,
microfips-esp32, microfips-esp32s3, microfips-esp-common,
microfips-esp-transport, microfips-l2cap-test, microfips-esp32-component,
microfips-build, fips-decrypt

---

## What Worked
- Full FIPS protocol stack on microcontrollers — leaf nodes participating in real FIPS mesh
- 95%+ wire-level parity with upstream FIPS proven
- Noise IK handshake verified with VPS (STM32F746, 2026-05-04)
- Noise XX migration completed (matches upstream v2 direction)
- Multi-transport: UART, BLE GATT, BLE L2CAP, WiFi all hardware-verified
- PlatformIO wrapper enables ESP-IDF/Arduino ecosystem access
- Embassy async HAL works well for ESP32 embedded Rust
- Compat audit identifies exact wire-protocol deltas

---

## What Didn't / Blockers
- **fips-core extraction premature** — upstream architecture (tokio coupling) prevents clean separation. Issue #122 closed. Waiting for v2 protocol stabilization.
- **Upstream FIPS getting heavier** — added NIM, more ethernet adapters. Extraction target keeps moving.
- **Translation burden** — microFIPS maintains its own protocol implementation; upstream changes require manual porting.
- **No ESP32 build target in upstream** — strategic goal deferred indefinitely.
- **FIPS v2 protocols incoming** — breaking changes ahead; microFIPS will need significant rework to maintain interop.

---

## Action Items

### Active
- [ ] Push feat/noise-xx-handshake to fork remote (verify it's pushed)
- [ ] Review the 340-line compat audit doc (in worktree) — clean up and publish
- [ ] Set up ESP32 testing lab for interop verification (multiple ESP32 boards + VPS)
- [ ] Study FIPS v2 protocol specs as jmcorgan publishes them
- [ ] Maintain interop test suite: microFIPS node <-> upstream FIPS VPS
- [ ] Consider: which small upstream contributions move toward runtime-agnosticism?

### Deferred (waiting on upstream)
- [ ] fips-core no_std extraction — wait for v2 protocols to stabilize
- [ ] ESP32 build target in upstream FIPS — wait for maintainer guidance
- [ ] Consume fips-core as dependency — only after extraction happens

### From Whiteboard Session (broader FIPS ecosystem, tracked here for visibility)
- [ ] Follow up with Mark M0: send TollGate demo video for UI feedback
- [ ] Finish IOI association voting + payouts
- [ ] Madeira meetup: discuss fips-core path in person with jmcorgan

---

## Relationship with Upstream FIPS

### What jmcorgan said (2026-07-04)
1. "Start with what the problems are before suggesting the solution"
2. Protocol state machines tightly integrated with tokio — can't just extract structs
3. "A very long timeframe methodical evolution, not a week-long changeover"
4. v2 protocols in development — extraction must wait
5. Writing detailed v2 specs to enable independent interop
6. "Small pieces at a time that move things in the right direction"
7. Sympathetic to ESP32 port, but "a lot more work than appears on the surface"

### What we do now
- Maintain microFIPS as the MCU FIPS implementation
- Test interoperability with upstream FIPS at each release
- Watch for v2 protocol specs
- When small runtime-agnostic opportunities arise upstream, contribute them
- Revisit extraction after v2 stabilizes

---

## Key Files & Repos

| Artifact | Path |
|----------|------|
| microFIPS repo | ~/repos/microfips |
| microFIPS fork (GitHub) | github.com/c03rad0r/microfips |
| microFIPS origin (GitHub) | github.com/Amperstrand/microfips |
| Upstream FIPS repo | ~/repos/fips (origin: github.com/jmcorgan/fips) |
| FIPS compat audit | ~/worktrees/feature-microfips-compat-audit/docs/fips-microfips-compat.md |
| fips-core extraction proposal (DEFERRED) | ~/plans/fips-no-std-core-extraction.md |
| TollGate/FIPS broader plan | ~/plans/tollgate-fips-master-plan.md |
| **This plan** | ~/plans/microfips-master-plan.md |

### VPS Access (FIPS test node)
```
VPS_HOST=orangeclaw.dns4sats.xyz
VPS_USER=routstr
```
FIPS binds 0.0.0.0:2121, MCU peers at 127.0.0.1:31337 (STM32) / 31338 (ESP32)

---

## Hardware

| Board | Chip | Role |
|-------|------|------|
| ESP32-C3 Mini V1 (x20) | ESP32-C3 (RISC-V) | Not yet tested with microFIPS (current targets are D0WD/S3) |
| ESP32-D0WD | ESP32 (Xtensa) | Primary microFIPS target, all transports verified |
| ESP32-S3 | ESP32-S3 (Xtensa) | Secondary target, all transports verified |
| STM32F469I-DISCO | STM32F4 (Cortex-M4) | Primary STM32 target |
| STM32F746G-DISCO | STM32F7 (Cortex-M7) | Hardware-verified 2026-05-04 |

**Note**: ESP32-C3 (RISC-V) is NOT yet a microFIPS build target. The balloon
project uses ESP32-C3 for LoRa firmware, but microFIPS targets are D0WD/S3.
Adding C3 support would require RISC-V target configuration.

---

## Glossary

| Term | Meaning |
|------|---------|
| FIPS | Free Internetworking Peering System (jmcorgan's mesh network) |
| microFIPS | MCU implementation of FIPS leaf node (this project) |
| FMP | FIPS Message Protocol (link-layer framing) |
| FSP | FIPS Session Protocol (session-layer, XK handshake) |
| MMP | Mesh Metrics Protocol (EWMA, jitter, SRTT) |
| Noise IK | Noise handshake pattern (initiator static key known to responder) |
| Noise XX | Noise handshake pattern (both static keys exchanged interactively) |
| no_std | Rust embedded target without standard library |
| Embassy | Async embedded HAL framework for Rust |
| PlatformIO | Build system/IDE for embedded development |
