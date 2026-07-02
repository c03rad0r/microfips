#!/usr/bin/env python3
"""PlatformIO extra-script: build the microfips Rust firmware image.

esp-hal firmware is a complete no_std *binary* (its own async ``main``), not a
C-linkable library. So this wrapper does not pretend to be a C component: it
runs ``cargo +esp build`` to produce the firmware image for the active board and
exposes its path so PlatformIO's upload step (or espflash) can flash it.

What it does:
1. Maps the active board → a Rust Xtensa target triple + cargo chip feature.
2. Reads the transport from ``-DMICROFIPS_TRANSPORT=<ble|l2cap|wifi|uart>``
   (default ``uart``).
3. Runs ``cargo +esp build --release`` on ``../Cargo.toml`` for the right
   ``[[bin]]`` and records the resulting ELF path in the env.

Requirements: the ``esp`` Rust toolchain (espup) and the Xtensa GCC. C-ABI
interop (calling into microfips from C/C++) is a documented future extension —
today the recommended integration is a pure-cargo ESP project (see the
``examples/uart-leaf`` and the crate README).
"""
from __future__ import annotations

import os
import sys

env = None
try:
    from SCons.Script import Import, DefaultEnvironment  # type: ignore

    Import("env")
    env = DefaultEnvironment()
except Exception:
    pass

# board id fragment -> (rust target triple, wrapper chip feature, default bin)
BOARD_MAP = {
    "esp32dev":       ("xtensa-esp32-none-elf",   "esp32",   "microfips-esp32"),
    "esp-wrover-kit": ("xtensa-esp32-none-elf",   "esp32",   "microfips-esp32"),
    "esp32thing":     ("xtensa-esp32-none-elf",   "esp32",   "microfips-esp32"),
    "esp32s3box":     ("xtensa-esp32s3-none-elf", "esp32s3", "microfips-esp32s3"),
    "esp32-s3-devkitc-1":       ("xtensa-esp32s3-none-elf", "esp32s3", "microfips-esp32s3"),
    "adafruit_feather_esp32s3": ("xtensa-esp32s3-none-elf", "esp32s3", "microfips-esp32s3"),
}

DEFAULT_TRANSPORT = "uart"
TRANSPORTS = {"uart", "ble", "l2cap", "wifi"}
# bin name suffix per transport (matches the [[bin]] names in the chip crates)
BIN_BY_TRANSPORT = {
    "uart":  None,                       # default bin, no suffix
    "ble":   "microfips-esp32-ble",      # esp32; esp32s3 uses the same suffix
    "l2cap": "microfips-esp32-l2cap",
    "wifi":  "microfips-esp32-wifi",
}


def resolve_board(board: str):
    key = board.lower()
    for frag, mapping in BOARD_MAP.items():
        if frag in key:
            return mapping
    raise SystemExit(
        f"microfips-esp32: board '{board}' unknown; add it to BOARD_MAP "
        f"({sorted(BOARD_MAP)})"
    )


def resolve_transport(build_flags) -> str:
    for flag in build_flags or []:
        if flag.startswith("-DMICROFIPS_TRANSPORT="):
            t = flag.split("=", 1)[1].strip().lower()
            if t not in TRANSPORTS:
                raise SystemExit(f"-DMICROFIPS_TRANSPORT={t} not in {sorted(TRANSPORTS)}")
            return t
    return DEFAULT_TRANSPORT


def build_firmware(triple, features, manifest, profile, bin_name):
    cargo = os.environ.get("MICROFIPS_CARGO", "cargo")
    toolchain = os.environ.get("MICROFIPS_RUST_TOOLCHAIN", "+esp")
    args = [cargo]
    if toolchain:
        args.append(toolchain)
    args += ["build", "--profile", profile, "--target", triple,
             "--manifest-path", manifest, "--features", ",".join(features)]
    if bin_name:
        args += ["--bin", bin_name]
    sys.stderr.write("[microfips-esp32] $ " + " ".join(args) + "\n")
    import subprocess

    rc = subprocess.call(args)
    if rc != 0:
        raise SystemExit(f"microfips-esp32: cargo build failed (exit {rc})")

    elf = os.path.normpath(
        os.path.join(manifest, "..", "target", triple, profile, bin_name or "microfips-esp32")
    )
    if not os.path.exists(elf):
        raise SystemExit(f"microfips-esp32: firmware ELF not found at {elf}")
    return elf


def main():
    if env is None:
        sys.stderr.write("[microfips-esp32] (dry-run: not inside PlatformIO)\n")
        return
    board = env.subst("$BOARD")
    build_flags = env.GetProjectOption("build_flags", []) or []
    triple, chip, default_bin = resolve_board(board)
    transport = resolve_transport(build_flags)

    # esp32s3 transport bins use the s3 base name
    bin_name = None
    if transport != "uart":
        base = "microfips-esp32s3" if chip == "esp32s3" else "microfips-esp32"
        bin_name = f"{base}-{transport}"

    features = [chip]
    if transport != "uart":
        features.append(transport)

    here = os.path.dirname(os.path.abspath(__file__))
    manifest = os.path.normpath(os.path.join(here, "..", "Cargo.toml"))
    elf = build_firmware(triple, features, manifest, "release", bin_name or default_bin)

    # Hand the built ELF to PlatformIO's upload step so `pio run -t upload`
    # (or espflash) flashes it. We also surface it as an env var for custom upload.
    env.SetDefault(UPGRADE_APP_DATA=elf)
    env.Replace(MICROFIPS_FIRMWARE_ELF=elf)
    sys.stderr.write(
        f"[microfips-esp32] firmware ready: {elf} "
        f"(chip={chip}, transport={transport})\n"
    )


if __name__ == "__main__" or env is not None:
    main()
