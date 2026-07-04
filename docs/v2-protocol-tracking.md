# FIPS v2 Protocol Changes — Impact Tracking

> jmcorgan is writing detailed v2 protocol specs to enable independent
> implementations to interoperate. This doc tracks each spec as it arrives
> and maps the microFIPS impact.

## Known v2 Changes (from issue #58 + maintainer conversation)

| Change | FIPS v0.x | FIPS v2 | microFIPS Status | Impact |
|--------|-----------|---------|-----------------|--------|
| Link handshake | Noise IK | Noise XX | DONE (feat/noise-xx) | LOW — already migrated |
| Session handshake | Noise XK | Noise XX | NOT STARTED | MEDIUM — need 3-msg session |
| FMP wire format | v0 | v1 | DONE (feat/noise-xx) | LOW — already bumped |
| Version negotiation | None | min/max + feature bitfield | NOT STARTED | HIGH — new protocol element |
| Profile negotiation | None | TLV extensions | NOT STARTED | MEDIUM — new protocol element |

## Spec Publication Watch

| Date | Spec | Source | microFIPS Action |
|------|------|--------|-----------------|
| (pending) | (pending) | [upstream maintainer] | (none yet) |

## Decision Points

1. When v2 specs arrive: assess scope of changes needed
2. When FIPS v2 ships: run interop test to detect all breaks
3. If changes are large: consider a v2-specific branch

## Maintainer Guidance (2026-07-04)

- Protocol state machines are tightly coupled to tokio runtime
- Extraction is "a very long timeframe methodical evolution, not a week-long changeover"
- v2 protocols must stabilize before any refactoring
- Detailed v2 specs being written specifically to enable independent interop
- "Small pieces at a time that move things in the right direction"
- Next in-person discussion: Madeira meetup
