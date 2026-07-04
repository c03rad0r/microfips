# Madeira Meetup — Discussion Points with Upstream Maintainer

## Context

microFIPS is a standalone MCU implementation of FIPS leaf nodes.
Issue #122 (fips-core extraction) closed after maintainer explained
tokio coupling. This is the in-person follow-up.

## Questions (prioritized)

### 1. v2 Protocol Timeline
- When will the v2 protocol specs be published?
- Is there a target release date for FIPS v2?
- Which v2 modules will be runtime-agnostic?

### 2. Small Runtime-Agnostic Contributions
- You mentioned "small pieces at a time" — what specific pieces?
- Are there utility modules (bloom filters, EWMA estimators, CRC)
  that could be extracted with low risk?
- Would you accept a CI target check for no_std compatibility
  on specific modules?

### 3. Noise XX + FMP v1
- Is FIPS `next` branch stable enough for interop testing?
- Can we get the v2 spec for the XX handshake to verify our
  implementation matches?
- Are there test vectors we can validate against?

### 4. ESP32 as Build Target
- What would the minimum viable path look like?
- Feature flags to exclude: tokio, transports, TUN, rtnetlink?
- Would a `fips-leaf` crate (subset with only leaf-node functionality)
  be more realistic than full ESP32 support?

### 5. microFIPS Role in the Ecosystem
- Should microFIPS be the reference implementation for embedded FIPS?
- How can we help test v2 interop from the MCU side?
- Is there interest in a FIPS conformance test suite?

## What We Bring
- Working FIPS leaf node on 4 MCU targets (ESP32, STM32)
- 95%+ wire-level parity proven
- Hardware-verified Noise handshake + heartbeat
- Testing infrastructure (sim + VPS + bridge tools)
- Willingness to contribute upstream

## What We Need
- v2 protocol specs for independent implementation
- Clear guidance on acceptable contribution scope
- Interop test vectors
