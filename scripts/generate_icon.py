#!/usr/bin/env python3
"""Generate NotchPilot's app icon (1024x1024 PNG) with only the stdlib.

Why a script instead of a committed PNG? The GitHub snapshot push used for
this repo only accepts UTF-8 text files, so binary icon files cannot be
pushed that way. The CI workflow runs this script on the macOS runner, then
runs `npx @tauri-apps/cli icon` to derive every size Tauri needs
(including icon.icns) before building.

The icon is a black rounded pill (the collapsed notch bar) with a small
camera dot, on a transparent background. No third-party packages needed.
"""

import math
import struct
import zlib
from pathlib import Path

SIZE = 1024
OUT = Path(__file__).resolve().parent.parent / "src-tauri" / "icons" / "icon.png"

# Pill geometry: x 112..912, y 392..632, corner radius 120.
PILL = {"cx": 512.0, "cy": 512.0, "hw": 400.0, "hh": 120.0, "r": 120.0}
# Camera dot: dark ring with a blue-grey lens centre.
DOT = {"cx": 512.0, "cy": 512.0, "r_out": 20.0, "r_in": 10.0}


def _sd_round_rect(x, y, p):
    """Signed distance from (x, y) to a rounded rect centred at (cx, cy)."""
    qx = abs(x - p["cx"]) - (p["hw"] - p["r"])
    qy = abs(y - p["cy"]) - (p["hh"] - p["r"])
    ax, ay = max(qx, 0.0), max(qy, 0.0)
    return math.hypot(ax, ay) + min(max(qx, qy), 0.0) - p["r"]


def _coverage(dist):
    """1 px antialiased coverage from a signed distance (0 outside)."""
    return max(0.0, min(1.0, 0.5 - dist))


def _png_chunk(kind, data):
    chunk = struct.pack(">I", len(data)) + kind + data
    return chunk + struct.pack(">I", zlib.crc32(kind + data) & 0xFFFFFFFF)


def build_png():
    raw = bytearray()
    for y in range(SIZE):
        raw.append(0)  # filter type: None
        cy = y + 0.5
        for x in range(SIZE):
            cx = x + 0.5
            pill_a = _coverage(_sd_round_rect(cx, cy, PILL))
            if pill_a == 0.0:
                raw.extend((0, 0, 0, 0))
                continue
            r, g, b = 0, 0, 0
            # Camera dot, composited over the pill.
            d = math.hypot(cx - DOT["cx"], cy - DOT["cy"])
            ring_a = _coverage(d - DOT["r_out"]) - _coverage(d - DOT["r_in"])
            lens_a = _coverage(d - DOT["r_in"])
            if ring_a > 0:
                w = ring_a
                r = round(r * (1 - w) + 30 * w)
                g = round(g * (1 - w) + 30 * w)
                b = round(b * (1 - w) + 34 * w)
            if lens_a > 0:
                w = lens_a
                r = round(r * (1 - w) + 70 * w)
                g = round(g * (1 - w) + 90 * w)
                b = round(b * (1 - w) + 130 * w)
            raw.extend((r, g, b, round(pill_a * 255)))
    ihdr = struct.pack(">IIBBBBB", SIZE, SIZE, 8, 6, 0, 0, 0)
    png = (
        b"\x89PNG\r\n\x1a\n"
        + _png_chunk(b"IHDR", ihdr)
        + _png_chunk(b"IDAT", zlib.compress(bytes(raw), 9))
        + _png_chunk(b"IEND", b"")
    )
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_bytes(png)
    print(f"Wrote {OUT} ({len(png)} bytes)")


if __name__ == "__main__":
    build_png()
