"""
Generate stylized 128x128 RGBA PNG Employee Portraits for Startup Tycoon.
Follows docs/ASSETS_PREPARATION.md Section 4.4 & 11.
"""

import math
import os
import struct
import zlib
from typing import Tuple


SIZE = 128


def write_png(path: str, width: int, height: int, rgba: bytes) -> None:
    def chunk(tag: bytes, data: bytes) -> bytes:
        return struct.pack(">I", len(data)) + tag + data + struct.pack(">I", zlib.crc32(tag + data) & 0xFFFFFFFF)

    raw = bytearray()
    stride = width * 4
    for y in range(height):
        raw.append(0)
        raw.extend(rgba[y * stride : (y + 1) * stride])

    ihdr = struct.pack(">IIBBBBB", width, height, 8, 6, 0, 0, 0)
    png = b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", ihdr) + chunk(b"IDAT", zlib.compress(bytes(raw), 9)) + chunk(b"IEND", b"")
    os.makedirs(os.path.dirname(os.path.abspath(path)), exist_ok=True)
    with open(path, "wb") as f:
        f.write(png)


def render_portrait(
    bg_color: Tuple[int, int, int],
    hoodie_color: Tuple[int, int, int],
    hair_color: Tuple[int, int, int],
    skin_color: Tuple[int, int, int],
    has_glasses: bool,
) -> bytes:
    buf = bytearray(SIZE * SIZE * 4)
    for y in range(SIZE):
        ny = (y + 0.5) / SIZE * 2.0 - 1.0
        for x in range(SIZE):
            nx = (x + 0.5) / SIZE * 2.0 - 1.0
            d = math.hypot(nx, ny)
            idx = (y * SIZE + x) * 4

            if d > 0.94:
                buf[idx : idx + 4] = b"\x00\x00\x00\x00"
                continue
            if d > 0.87:
                buf[idx : idx + 4] = bytes((52, 211, 153, 255))
                continue

            col = bg_color
            # Shoulders / hoodie
            if ny >= 0.30 and (nx / 0.65) ** 2 + ((ny - 1.05) / 0.78) ** 2 <= 1.0:
                col = hoodie_color
                if abs(nx) < 0.14 and ny < 0.60:
                    col = (38, 43, 52)
            # Neck
            elif abs(nx) < 0.13 and 0.14 <= ny <= 0.38:
                col = (max(0, skin_color[0] - 18), max(0, skin_color[1] - 18), max(0, skin_color[2] - 18))

            # Hair back/top
            if math.hypot(nx / 0.36, (ny + 0.24) / 0.38) <= 1.0 and ny < -0.05:
                col = hair_color

            # Head
            if math.hypot(nx / 0.31, (ny + 0.12) / 0.36) <= 1.0:
                col = skin_color
                # Hair fringe
                if ny < -0.26 or (ny < -0.18 and abs(nx) > 0.20):
                    col = hair_color
                # Eyes
                if math.hypot(nx + 0.11, ny + 0.09) < 0.04 or math.hypot(nx - 0.11, ny + 0.09) < 0.04:
                    col = (24, 28, 36)
                # Glasses
                if has_glasses:
                    in_l = abs(nx + 0.12) < 0.09 and abs(ny + 0.09) < 0.06
                    in_r = abs(nx - 0.12) < 0.09 and abs(ny + 0.09) < 0.06
                    in_b = abs(nx) < 0.05 and abs(ny + 0.10) < 0.02
                    if (in_l or in_r or in_b) and not (
                        abs(nx + 0.12) < 0.065 and abs(ny + 0.09) < 0.035
                    ) and not (
                        abs(nx - 0.12) < 0.065 and abs(ny + 0.09) < 0.035
                    ):
                        col = (20, 24, 30)
                # Smile
                if 0.04 <= ny <= 0.09 and abs(nx) < 0.09:
                    col = (180, 105, 95)

            buf[idx : idx + 4] = bytes((col[0], col[1], col[2], 255))
    return bytes(buf)


def main() -> None:
    specs = [
        ("developer_01.png", (30, 41, 59), (36, 112, 108), (45, 32, 26), (235, 184, 148), True),
        ("developer_02.png", (32, 46, 68), (59, 105, 168), (72, 42, 28), (242, 198, 166), False),
    ]
    for filename, bg, hoodie, hair, skin, glasses in specs:
        out = os.path.join("assets", "characters", "employees", "portraits", filename)
        write_png(out, SIZE, SIZE, render_portrait(bg, hoodie, hair, skin, glasses))
        print(f"Generated {out} ({os.path.getsize(out)} bytes)")


if __name__ == "__main__":
    main()
