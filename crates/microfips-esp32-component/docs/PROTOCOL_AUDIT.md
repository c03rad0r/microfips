# Phase 1 — Wire Protocol Compatibility Audit

**Scope:** Re-audit wire-protocol compatibility between FIPS (upstream
`github.com/Amperstrand/fips`, branch `integrate/macos-linux-sync`) and the
`microfips-protocol` crate, and confirm framing compatibility with the
`tollgate-protocol` crate (tollgate-rs).

**Audited baseline:**
- microfips repo `main` @ `b6bfc9d` ("feat(core): migrate FMP wire format to v1 (Noise XX)")
- Existing parity doc: `docs/fips-microfips-parity.md` (written against the *old* IK/XK baseline)
- Verification: `cargo test -p microfips-protocol --features std` (host, this environment)

---

## 1. Headline finding — FMP v1 / Noise XX migration is INCOMPLETE (release blocker)

The HEAD commit claims to migrate the FMP wire format from v0 (Noise IK, 2-message)
to v1 (Noise XX, 3-message). The migration landed in the **wire + noise primitive
layers** but **not in the Node state machine**. The result is an inconsistent crate
that does not interoperate with either an IK or an XX peer and fails 2 unit tests.

### Evidence

**Layer 1 — wire constants (`microfips-core/src/wire.rs`): updated to XX**

```
FMP_VERSION = 1
HANDSHAKE_MSG1_SIZE = noise::XX_HANDSHAKE_MSG1_SIZE   // 33
HANDSHAKE_MSG2_SIZE = noise::XX_HANDSHAKE_MSG2_SIZE   // 106
HANDSHAKE_MSG3_SIZE = noise::XX_HANDSHAKE_MSG3_SIZE   // 73
MSG1_WIRE_SIZE = COMMON_PREFIX_SIZE + IDX_SIZE + 33            = 41
MSG2_WIRE_SIZE = COMMON_PREFIX_SIZE + IDX_SIZE*2 + 106         = 118
MSG3_WIRE_SIZE = COMMON_PREFIX_SIZE + IDX_SIZE*2 + 73          = 85
PHASE_MSG3 = 0x03 (new), FLAG_SP removed
```

**Layer 2 — noise primitives (`microfips-core/src/noise.rs`): XX types added**

`NoiseXxInitiator` / `NoiseXxResponder`, `PROTOCOL_NAME_XX =
"Noise_XX_secp256k1_ChaChaPoly_SHA256"`, XX message-size constants. The module
doc explicitly states IK's deviation D2 is "eliminated in 0.4.0-dev by switching
to Noise XX."

**Layer 3 — Node state machine (`microfips-protocol/src/node.rs`): NOT migrated**

`node.rs` still drives a **2-message IK handshake**:

- `NoiseIkInitiator` / `NoiseIkResponder` referenced **14 times**; `NoiseXx*`
  referenced **0 times**.
- `Node::handshake` performs `write_message1` → wait → `read_message2` (the IK
  flow). There is no `msg3` send/receive step.
- Initiator emits a **114-byte** FMP msg1 (4 prefix + 4 idx + 106 IK noise),
  not the 41-byte XX msg1 the wire layer now advertises.

**Layer 4 — unit tests: 2 FAIL** (`cargo test -p microfips-protocol --features std`):

| Test | File:line | Failure |
|------|-----------|---------|
| `test_handshake_msg1_wire_size` | node.rs:2344 | `assert_eq!(msg1_len, MSG1_WIRE_SIZE)` → `left: 114, right: 41`. Node sends IK-size msg1; constant expects XX. |
| `test_extract_raw_frame_msg2_mid_buffer` | node.rs:3035 | `Option::unwrap()` on `None`. Builds a 65-byte MSG2 payload (IK size) but `MSG2_WIRE_SIZE` is now 118 (XX), so `extract_raw_frame` returns `None`. |

129 passed, 2 failed, 1 ignored. The commit message's claim "All 246 tests pass
(2 ignored)" is inaccurate for this baseline — the two failures are real
assertion/panic failures, not the documented PCAP ignores.

### Consequence

A firmware built from current `main` performs an IK handshake (msg1=114B) while
advertising `FMP_VERSION=1` (XX). It will fail against:
- a FMP v1 (XX) FIPS daemon — wrong message sizes / phase sequence, and
- a FMP v0 (IK) FIPS daemon — the version nibble (1 vs 0) is rejected by
  `wire::parse_prefix`.

**This blocks Phase 6 (publish).** The fix is a dedicated task: migrate
`microfips-protocol/src/node.rs` `Node::handshake` from IK (2-msg) to XX (3-msg),
thread `msg3` through the state machine + `FspDualHandler`, and update the two
stale tests to XX sizes. (Follow-up task spawned — see task comments.)

### Note on the existing parity doc

`docs/fips-microfips-parity.md` is **stale**: it documents the IK/XK surface
(msg1=114, msg2=69, no MSG3, `PROTOCOL_NAME`/`PROTOCOL_NAME_XK`, `FLAG_SP`).
It should be regenerated against FMP v1 once `node.rs` is migrated. Until then it
must not be cited as the current wire contract.

---

## 2. Framing layer — COMPATIBLE (no change required for packaging)

The length-prefixed framing is identical across FIPS, microfips, and tollgate and
needs no modification for the ESP32 packaging work:

| Concern | FIPS / microfips-protocol | tollgate-protocol | Compatible? |
|---------|---------------------------|--------------------|-------------|
| Serial / stream framing | `[2-byte LE length][payload]` via `FrameWriter`/`FrameReader` | `[2-byte LE length][payload]` via `encode_frame`/`decode_frames` | ✅ byte-identical |
| UDP / TCP transport | raw frames (no prefix) via `Node::set_raw_framing(true)` | n/a (HTTP-poll/WebSocket) | ✅ FIPS-specific, correct |
| BLE L2CAP SDU prefix | 2-byte **BE** length, PSM `0x0085` | n/a | ✅ matches upstream FIPS |
| Max frame | `MAX_FRAME = 1500` (framing.rs), `MAX_FRAME_SIZE = 2048` (node recv buf) | `MAX_FRAME_LEN = u16::MAX` | ⚠️ app-level; see §3 |

FIPS and TollGate are **independent protocols that share a transport-framing
convention only**. TollGate is CBOR-mapped (`minicbor`, field key 0 = message
type, 15 message types). FMP is a custom binary format (Noise handshake + FSP
session + MMP reports). There is no message-level interop between them, and none
is required for this task — the ESP32 leaf speaks FMP to a FIPS daemon/host.

---

## 3. ESP32 transport-specific compat notes (unchanged, confirmed)

From ADR 0007 and the parity doc, still accurate for packaging:

- **FRAME_CAP vs MTU split**: BLE L2CAP negotiates MTU 2048 (matches FIPS), but
  the D0WD app-level `L2CAP_FRAME_CAP = 768` (binary-searched RAM budget). FIPS
  skips `FilterAnnounce`/oversized mesh messages to MTU-limited peers, so this is
  invisible to FIPS. No action for packaging; documented for consumers.
- **PSM `0x0085` (133)**, service UUID `0x9c90…8f4c`, capability UUID `FI` — all
  match upstream FIPS.
- **raw framing flag**: UDP/WiFi/L2CAP paths call `node.set_raw_framing(true)` to
  match FIPS's raw-frame UDP/TCP wire format.

---

## 4. Verdict per phase

| Phase | Status | Notes |
|-------|--------|-------|
| Framing compatibility | ✅ PASS | Identical `[2-byte LE len][payload]`; no changes needed. |
| Noise handshake compatibility | ❌ BLOCKER | node.rs still IK; wire/noise are XX. 2 tests fail. |
| Transport constants (L2CAP/PSM/UUID) | ✅ PASS | Match upstream FIPS. |
| Parity documentation | ⚠️ STALE | `fips-microfips-parity.md` predates FMP v1. |

**Packaging impact:** Phases 2–5 (wrapper crate, PlatformIO, ESP-IDF, example,
docs) describe *distribution structure* and are independent of the handshake bug —
they can and should proceed. **Phase 6 (publish) must not run** until the node.rs
XX migration lands and `cargo test -p microfips-protocol --features std` is green.
