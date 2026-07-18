# Integration Assessment — Balloon FIPS Track

Date
2026-07-18

## What Works Right Now

| Item | Platform | Evidence |
|------|----------|----------|
| FIPS Noise IK handshake protocol | Host (sim), ESP32-C3 (UART) | Commit a5ae334, verified against upstream FIPS v0.5.0-dev |
| UART transport binary | ESP32-C3 | Flashed to /dev/ttyACM1 (MAC 70:af:09:13:21:00), serial output confirmed |
| WiFi transport binary | ESP32-C3 | Flashed to /dev/ttyACM3, cargo build --release PASS (feat/fips-v0-compat branch) |
| USB transport binary | ESP32-C3 | cargo build --release PASS, flashed |
| FIPS protocol crate (framing, transport trait, node, FSP handler) | Host + ESP32-C3 | Compiles clean, unit tests pass with --features std |
| Frame compaction (framing.rs) | Host | Unit tests pass |
| LR2021 FLRC framing module (TxFramer/RxFramer) | Host (test crate) | 453 lines, 13 unit tests — tests written but NOT currently compiling due to import errors in standalone test crate |
| LR2021 SPI register constants + trait | Host (code only) | 394 lines, Lr2021Radio trait + MockLr2021Radio defined |
| LR2021 Transport adapter | Host (code only) | 287 lines, implements Transport trait, IRQ-driven RX design |
| Radio baseline (FLRC 2440 MHz, 2600 kbps, 0% loss) | ESP32-S3 + LR2021 module | Proven by balloon-range-tests track, 1377 kbps sustained |

## What Exists But Is Untested

| Item | Status | Notes |
|------|--------|-------|
| LR2021 framing roundtrip (TX→radio→RX) | Code written, test crate broken (20 compile errors) | Test crate at crates/microfips-lr2021-test/ has import path issues (E0432, E0433, E0596). Needs fixing before tests can run. |
| LR2021 SPI driver (Lr2021Radio impl) | Trait + mock defined, no real hardware impl | EspHalLr2021Radio not written. Mock exists but can't test real SPI. |
| LR2021 Transport (Lr2021Transport<R>) | Implements Transport trait, never flashed | Depends on EspHalLr2021Radio which doesn't exist. No integration test with real radio. |
| ESP-NOW transport (espnow.rs binary) | cargo check PASSES, cargo build FAILS at link | ESP-IDF framework symbols undefined (nvs_flash_init, esp_wifi_init, etc.). Stub transport committed (b02c6b0) — send() logs, recv() blocks. No real radio. |
| FIPS mesh routing (STP + bloom filters) | ~10% on feat/mac-mapping branch (unmerged) | MAC mapping code exists but not merged. STP and bloom filter not implemented. |
| Noise IK responder path | Initiator works, responder still uses Noise XX | Bidirectional handshake incomplete. Only initiator→responder direction works. |

## What Does NOT Exist Yet

1. **Erasure coding pipeline** — balloon-fresh has 325 lines of PRBS23-XOR erasure C code. NOT ported to Rust. Without this, FIPS frames > 244 bytes (after ESP-NOW overhead) or > 253 bytes (after LR2021 framing) cannot be sent. Most FIPS protocol frames (MSG1=114B, heartbeats=32B) fit, but full session setup (148B+) and data frames need fragmentation + erasure.

2. **EspHalLr2021Radio** — real SPI implementation of Lr2021Radio trait for esp-hal. No code written. Need SPI init, register config, TX/RX sequence, IRQ handling via GPIO5 (DIO9).

3. **ESP-NOW real radio init** — esp-radio 0.18 `EspNow::new_internal()` is `pub(crate)`. Either fork esp-radio to expose constructor, or find alternative API. Stub transport is the only option currently.

4. **FIPS STP (spanning tree protocol)** — mesh routing intelligence. Root election, path tracking. Not started.

5. **Bloom filter path tracking** — FIPS mesh forwarding mechanism. Not started.

6. **MAC-to-node-address mapping** — on unmerged feat/mac-mapping branch, needs review + merge.

7. **Noise IK responder** — bidirectional FIPS handshake. Only initiator direction implemented.

8. **ESP32-C3 workspace integration** — microfips-esp32c3 crate NOT in workspace.members (excluded for std-leak prevention per pitfall #19). Cannot build from workspace root. Must cd into crate dir or use --package flag with filtered workspace.

9. **Two-node ESP-NOW demo** — no flashable binary exists. Blocked by ESP-NOW linker issue (#3 above).

10. **Two-node LR2021 demo** — no real hardware driver. Blocked by EspHalLr2021Radio (#2 above).

## Blockers for ESP32-C3 Port

| Component | Limit | Current Usage | Status |
|-----------|-------|---------------|--------|
| Flash (4MB) | 4MB | UART binary ~ fits; WiFi binary ~ fits | OK — binaries compile to <2MB release |
| RAM (400KB) | 400KB total, ~100KB for app | FIPS + transport ~ estimated 80-120KB | TIGHT — no measurement done. WiFi stack alone is ~100KB. ESP-NOW is lighter (~20KB). Need actual measurement. |
| Single-core | No parallelism | Embassy async executor | OK — async is the right model |
| No PSRAM | All state in internal RAM | heapless::Vec used in LR2021 framing (no alloc) | OK — LR2021 modules are no-alloc by design |
| Atomic operations | riscv32imc lacks target_has_atomic=ptr | portable_atomic used instead | OK — resolved (pitfall #10), but fragile (features=["fallback"] regression risk) |
| ESP-NOW radio init | esp-radio pub(crate) constructor | Stub only | BLOCKED — needs fork or upstream fix |
| ESP-IDF symbols | Not available in no_std | esp_now_transport.rs uses raw FFI | BLOCKED — needs refactor to esp-radio safe API |

**Key blocker:** ESP-NOW real radio cannot be initialized without either forking esp-radio or waiting for upstream. LR2021 path is NOT blocked by this (uses SPI directly, no ESP-IDF dependency).

## Estimated Effort

| Work Item | Effort | Confidence |
|-----------|--------|------------|
| Fix LR2021 test crate compile errors | 2-4 hours | HIGH — import path issues, known pattern (pitfall #29) |
| Write EspHalLr2021Radio (SPI driver) | 2-3 days | MEDIUM — SPI register sequence documented, but real hardware debugging always finds issues |
| Port erasure coding to Rust no_std | 1-2 days | HIGH — 325 lines C, straightforward port |
| Integration: LR2021 transport + FIPS node | 1-2 days | MEDIUM — wire Lr2021Transport into node runner |
| Flash LR2021 binary to ESP32-C3 + verify radio init | 1 day | MEDIUM — depends on hardware availability |
| Two-node LR2021 FIPS demo | 2-3 days | LOW — full chain: radio init → framing → FIPS handshake → message |
| ESP-NOW real radio (fork esp-radio or alternative) | 3-5 days | LOW — unknown API surface, may need deep esp-radio internals |
| FIPS STP + bloom filter routing | 1-2 weeks | LOW — complex distributed algorithm, no existing Rust impl |
| Noise IK responder | 1-2 days | MEDIUM — mirror of initiator path |
| MAC mapping merge + integration | 1 day | HIGH — code exists, just needs review |

**Total to first LR2021 FIPS demo (critical path):** ~7-10 days focused work.
**Total to full mesh routing:** ~3-4 weeks.

## Dependencies on Other Tracks

| Dependency | From Track | What We Need | Blocking? |
|------------|-----------|--------------|-----------|
| Radio baseline confirmation | balloon-range-tests | FLRC params verified (2440 MHz, 2600 kbps, +12 dBm) — ALREADY PROVIDED | No — already have this |
| LR2021 SPI pin mapping | (internal, from skill) | GPIO mapping documented (MOSI=7, MISO=2, SCLK=6, CS=10, RESET=3, BUSY=4, DIO9=5) | No — documented |
| Erasure coding source | balloon-fresh | tracker/firmware/components/erasure/erasure.c (325 lines) — AVAILABLE for port | No — source exists |
| FIPS upstream protocol compat | (external) | Upstream FIPS v0.5.0-dev uses Noise IK — we match. Upstream can't run on MCUs yet. | No — we run leaner version |
| Nostr event transport | balloon-nostr | NOT needed for FIPS mesh. FIPS handles routing internally. Nostr is application layer on top. | No — independent |

**No hard blocking dependencies on other balloon tracks.** FIPS track can proceed independently. The LR2021 radio baseline is already proven. Erasure source code is available for porting.

## Shared Resources Needed

| Resource | Who Else Needs It | When |
|----------|-------------------|------|
| ESP32-C3 boards (minimum 2 for demo) | All tracks use C3 as target. Currently 2 on bench (ACM1, ACM2/ACM3) | NOW — for flashing + two-node demo |
| LR2021 modules (minimum 2 for radio demo) | balloon-range-tests, balloon-speed-tests use these | NOW — for EspHalLr2021Radio development + testing |
| ESP32-S3 boards | balloon-tollgate, balloon-pow also want S3 | NOT NEEDED — FIPS targets C3, not S3 |
| DQ05 build server | All Rust tracks | NOW — T470 OOMs on Rust builds, DQ05 is the build machine |

**Key constraint:** LR2021 modules are the critical shared resource. Need at least 2 for the FIPS radio demo. These are the same modules balloon-range-tests used for the baseline. Need coordination to ensure they're available for FIPS integration testing.

## Integration Checklist

1. [ ] LR2021 test crate compiles and all 13 framing tests pass on host
2. [ ] EspHalLr2021Radio implemented for esp-hal SPI (real hardware driver)
3. [ ] Lr2021Transport roundtrip test passes with MockLr2021Radio (host)
4. [ ] Lr2021Transport compiles for riscv32imc-unknown-none-elf target
5. [ ] Erasure coding ported to Rust no_std (from balloon-fresh C source)
6. [ ] Erasure coding unit tests pass on host
7. [ ] Erasure coding compiles for riscv32imc target
8. [ ] LR2021 binary flashes to ESP32-C3 and radio init succeeds (serial log: frequency, TX power, FLRC mode)
9. [ ] Two-node LR2021 demo: Node A sends FIPS MSG1 via FLRC radio, Node B receives and logs it
10. [ ] FIPS Noise IK handshake completes over LR2021 radio (MSG1 → MSG2)
11. [ ] FIPS encrypted message sent + received over LR2021 (end-to-end crypto over radio)
12. [ ] MAC mapping branch reviewed and merged into feat/lr2021-transport
13. [ ] Noise IK responder implemented (bidirectional handshake)
14. [ ] ESP-NOW transport: either fork esp-radio OR document LR2021 as sole radio path
15. [ ] RAM usage measured on ESP32-C3 (verify < 300KB used, leaving margin)
16. [ ] 24-hour stability test: two nodes running, heartbeat alive, no panics

## Key Risks

1. **esp-radio EspNow pub(crate) constructor** — If we can't initialize ESP-NOW, the ESP32-C3's built-in WiFi radio is unusable for FIPS mesh. LR2021 becomes the ONLY radio path. This is acceptable for balloon (LR2021 is the intended long-range radio anyway), but limits short-range mesh options. Mitigation: focus on LR2021, treat ESP-NOW as future work.

2. **LR2021 SPI driver debugging** — SPI peripheral drivers always have timing issues, pin mux problems, and errata. The documented pin mapping (from NiceRF LoRa2021 module) is unverified on our specific board. Risk: 1-2 extra days of oscilloscope/logic-analyzer debugging. Mitigation: use RP2040 logic analyzer (3MHz bare, sufficient for SPI debug) to verify signals.

3. **RAM budget on ESP32-C3** — 400KB total RAM, WiFi stack claims ~100KB, FIPS + Embassy + transport need their share. No actual measurement done. If we're over budget, we may need to drop WiFi entirely and use only LR2021 + UART. Risk: redesign of transport selection. Mitigation: measure early (item #15 in checklist).

4. **portable-atomic regression** — The `features = ["fallback"]` in workspace Cargo.toml re-triggers the riscv32imc gating bug (pitfall #10). This has been "fixed" 3 times but the fix was never committed. Any workspace Cargo.toml change can re-introduce it. Risk: build breaks silently after unrelated dep update. Mitigation: add CI check that grep's for `features = ["fallback"]` in workspace Cargo.toml.

5. **Workspace std leak** — Including host-only crates in workspace.members leaks `std` to no_std targets (pitfall #19). Current workspace has ALL members included. ESP32-C3 crate is excluded from members to prevent this. Risk: if someone adds it back, builds break with confusing errors. Mitigation: document the constraint, add workspace.exclude for esp32c3 if needed.

6. **FIPS upstream protocol drift** — Upstream is doing sans-io rewrite (v0.5.0-dev). If they break Noise IK compatibility, our handshake stops working. Risk: silent handshake failure (pitfall #21 — FIPS drops silently with zero logs). Mitigation: pin to v0.4.0 for server, maintain v0 compat on our side.

7. **Erasure coding correctness** — Porting C to Rust is straightforward but crypto-adjacent code must be exactly right. A single bit error in PRBS23 XOR makes all redundant fragments useless. Risk: erasure coding "works" in tests but fails in production with real packet loss. Mitigation: port the C tests too, test against known vectors from balloon-fresh.

## Questions for the Coordinator

1. **ESP-NOW vs LR2021 priority:** Should I abandon ESP-NOW entirely and focus 100% on LR2021? ESP-NOW is blocked by esp-radio pub(crate) with no clear fix path. LR2021 is the intended balloon radio anyway (long-range, 2.4GHz FLRC). Abandoning ESP-NOW saves 3-5 days.

2. **Hardware allocation:** I need 2x LR2021 modules + 2x ESP32-C3 boards for the radio demo. Are these available now, or are balloon-range-tests/speed-tests still using them? If shared, what's the schedule?

3. **Erasure coding scope:** For the first demo (FIPS handshake over LR2021), erasure coding is NOT needed — MSG1 is 114B, fits in single 255B FLRC packet. Should I defer erasure porting to after the demo, or do it first as foundational work?

4. **FIPS routing scope:** STP + bloom filter routing is 1-2 weeks of work. For the balloon mission, is mesh routing needed (multiple hops), or is point-to-point (two nodes directly) sufficient for first flight? If point-to-point is enough, we can skip routing entirely for v1.

5. **Branch strategy:** LR2021 work is on feat/lr2021-transport branched from feat/fips-v0-compat. Should I merge back to fips-v0-compat when the demo works, or keep as a long-lived branch?

6. **DQ05 build server:** Can I assume DQ05 (192.168.2.12) is available for Rust builds? T470 OOMs. Need to verify microfips repo is cloned there with current branches.