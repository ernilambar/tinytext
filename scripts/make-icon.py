#!/usr/bin/env python3
"""Generate assets/icon.icns — a placeholder app icon.

Renders a rounded-square gradient with a white "T", resizes it into the
standard .iconset sizes with `sips`, and packs it with `iconutil`.
Replace assets/icon.icns with real artwork whenever you have it.
"""

import os
import shutil
import struct
import subprocess
import tempfile
import zlib

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
ASSETS = os.path.join(ROOT, "assets")
ICNS = os.path.join(ASSETS, "icon.icns")

BASE = 1024
SS = 2  # supersample factor for edge smoothing
TOP = (56, 132, 255)
BOTTOM = (147, 89, 255)


def write_png(path, width, height, rgba):
    def chunk(kind, data):
        payload = kind + data
        crc = zlib.crc32(payload) & 0xFFFFFFFF
        return struct.pack(">I", len(data)) + payload + struct.pack(">I", crc)

    rows = bytearray()
    stride = width * 4
    for y in range(height):
        rows.append(0)
        rows += rgba[y * stride : (y + 1) * stride]

    ihdr = struct.pack(">IIBBBBB", width, height, 8, 6, 0, 0, 0)
    with open(path, "wb") as f:
        f.write(b"\x89PNG\r\n\x1a\n")
        f.write(chunk(b"IHDR", ihdr))
        f.write(chunk(b"IDAT", zlib.compress(bytes(rows), 9)))
        f.write(chunk(b"IEND", b""))


def render(size):
    s = size * SS
    margin = 0.06 * s
    radius = 0.225 * s
    x0 = y0 = margin
    x1 = y1 = s - margin

    cx = s / 2.0
    hori_w2 = 0.23 * s
    hori_h = 0.115 * s
    bar_w2 = 0.06 * s
    vert_h = 0.42 * s
    top_y = 0.29 * s

    out = bytearray(size * size * 4)
    span = y1 - y0
    for oy in range(size):
        yo = oy * SS
        for ox in range(size):
            xo = ox * SS
            r = g = b = 0
            covered = 0
            for sy in range(SS):
                y = yo + sy + 0.5
                if y < y0 or y > y1:
                    continue
                for sx in range(SS):
                    x = xo + sx + 0.5
                    if x < x0 or x > x1:
                        continue
                    nx = x0 + radius if x < x0 + radius else (x1 - radius if x > x1 - radius else x)
                    ny = y0 + radius if y < y0 + radius else (y1 - radius if y > y1 - radius else y)
                    dx = x - nx
                    dy = y - ny
                    if dx * dx + dy * dy > radius * radius:
                        continue
                    covered += 1
                    if cx - hori_w2 <= x <= cx + hori_w2 and top_y <= y <= top_y + hori_h:
                        r += 255
                        g += 255
                        b += 255
                    elif cx - bar_w2 <= x <= cx + bar_w2 and top_y <= y <= top_y + vert_h:
                        r += 255
                        g += 255
                        b += 255
                    else:
                        t = (y - y0) / span
                        r += int(TOP[0] + (BOTTOM[0] - TOP[0]) * t)
                        g += int(TOP[1] + (BOTTOM[1] - TOP[1]) * t)
                        b += int(TOP[2] + (BOTTOM[2] - TOP[2]) * t)

            if covered:
                i = (oy * size + ox) * 4
                out[i] = r // covered
                out[i + 1] = g // covered
                out[i + 2] = b // covered
                out[i + 3] = (covered * 255) // (SS * SS)
    return out


def main():
    if shutil.which("iconutil") is None or shutil.which("sips") is None:
        raise SystemExit("make-icon.py needs macOS `sips` and `iconutil`")

    os.makedirs(ASSETS, exist_ok=True)
    work = tempfile.mkdtemp()
    try:
        base = os.path.join(work, "base.png")
        write_png(base, BASE, BASE, render(BASE))

        iconset = os.path.join(work, "AppIcon.iconset")
        os.makedirs(iconset)
        sizes = [
            (16, "16x16"),
            (32, "16x16@2x"),
            (32, "32x32"),
            (64, "32x32@2x"),
            (128, "128x128"),
            (256, "128x128@2x"),
            (256, "256x256"),
            (512, "256x256@2x"),
            (512, "512x512"),
            (1024, "512x512@2x"),
        ]
        for px, name in sizes:
            subprocess.run(
                ["sips", "-z", str(px), str(px), base, "--out", os.path.join(iconset, f"icon_{name}.png")],
                check=True,
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
            )

        subprocess.run(["iconutil", "-c", "icns", iconset, "-o", ICNS], check=True)
        print(f"Wrote {ICNS}")
    finally:
        shutil.rmtree(work, ignore_errors=True)


if __name__ == "__main__":
    main()
