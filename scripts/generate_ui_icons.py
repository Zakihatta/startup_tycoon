"""
Generate minimalist 64x64 transparent RGBA PNG UI icons for Startup Tycoon.
Follows docs/ASSETS_PREPARATION.md Section 8 & 11 naming conventions.
"""

import math
import os
import struct
import zlib
from typing import Callable, Tuple


SIZE = 64


def write_png(path: str, width: int, height: int, rgba_bytes: bytes) -> None:
    def chunk(tag: bytes, data: bytes) -> bytes:
        return struct.pack(">I", len(data)) + tag + data + struct.pack(">I", zlib.crc32(tag + data) & 0xFFFFFFFF)

    raw = bytearray()
    stride = width * 4
    for y in range(height):
        raw.append(0)
        raw.extend(rgba_bytes[y * stride : (y + 1) * stride])

    ihdr = struct.pack(">IIBBBBB", width, height, 8, 6, 0, 0, 0)
    png = b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", ihdr) + chunk(b"IDAT", zlib.compress(bytes(raw), 9)) + chunk(b"IEND", b"")
    os.makedirs(os.path.dirname(os.path.abspath(path)), exist_ok=True)
    with open(path, "wb") as f:
        f.write(png)


def render_icon(pixel_fn: Callable[[float, float], Tuple[int, int, int, int]]) -> bytes:
    buf = bytearray(SIZE * SIZE * 4)
    for y in range(SIZE):
        ny = (y + 0.5) / SIZE * 2.0 - 1.0  # -1..1
        for x in range(SIZE):
            nx = (x + 0.5) / SIZE * 2.0 - 1.0
            r, g, b, a = pixel_fn(nx, ny)
            idx = (y * SIZE + x) * 4
            buf[idx : idx + 4] = bytes((r, g, b, a))
    return bytes(buf)


def icon_money(x: float, y: float) -> Tuple[int, int, int, int]:
    d = math.hypot(x, y)
    if d <= 0.82:
        if d >= 0.68:
            return (22, 163, 114, 255)
        # Inner coin with '$' bars
        in_vbar = abs(x) < 0.08 and abs(y) < 0.48
        in_top = abs(y + 0.22) < 0.11 and -0.26 < x < 0.26
        in_mid = abs(y) < 0.09 and -0.26 < x < 0.26
        in_bot = abs(y - 0.22) < 0.11 and -0.26 < x < 0.26
        in_left_leg = abs(x + 0.18) < 0.08 and -0.26 < y < 0.04
        in_right_leg = abs(x - 0.18) < 0.08 and -0.04 < y < 0.26
        if in_vbar or in_top or in_mid or in_bot or in_left_leg or in_right_leg:
            return (248, 250, 252, 255)
        return (34, 197, 142, 255)
    return (0, 0, 0, 0)


def icon_time(x: float, y: float) -> Tuple[int, int, int, int]:
    d = math.hypot(x, y)
    if d <= 0.82:
        if d >= 0.66:
            return (96, 165, 250, 255)
        # Clock hands
        hand_up = abs(x) < 0.07 and -0.45 < y < 0.06
        hand_right = abs(y) < 0.07 and -0.04 < x < 0.36
        if hand_up or hand_right:
            return (248, 250, 252, 255)
        return (30, 58, 110, 235)
    return (0, 0, 0, 0)


def icon_company(x: float, y: float) -> Tuple[int, int, int, int]:
    # Minimalist office building
    if -0.55 <= x <= 0.55 and -0.75 <= y <= 0.78:
        # Windows grid
        for wy in (-0.48, -0.16, 0.16):
            for wx in (-0.28, 0.0, 0.28):
                if abs(x - wx) < 0.09 and abs(y - wy) < 0.09:
                    return (226, 232, 240, 255)
        # Door
        if abs(x) < 0.12 and y > 0.42:
            return (34, 197, 142, 255)
        return (59, 130, 246, 255)
    return (0, 0, 0, 0)


def icon_employee(x: float, y: float) -> Tuple[int, int, int, int]:
    # Head
    if math.hypot(x, y + 0.30) <= 0.27:
        return (248, 250, 252, 255)
    # Shoulders
    if y >= 0.05 and y <= 0.75 and (x / 0.62) ** 2 + ((y - 0.75) / 0.70) ** 2 <= 1.0:
        return (56, 189, 248, 255)
    return (0, 0, 0, 0)


def icon_product(x: float, y: float) -> Tuple[int, int, int, int]:
    # Stylized monitor / app card
    if -0.78 <= x <= 0.78 and -0.60 <= y <= 0.45:
        if abs(x) > 0.65 or y < -0.47 or y > 0.32:
            return (148, 163, 184, 255)
        if -0.45 <= x <= 0.45 and -0.25 <= y <= 0.12:
            return (34, 197, 142, 255)
        return (30, 41, 59, 255)
    if abs(x) < 0.12 and 0.45 <= y <= 0.68:
        return (148, 163, 184, 255)
    if abs(x) < 0.38 and 0.66 <= y <= 0.78:
        return (148, 163, 184, 255)
    return (0, 0, 0, 0)


def icon_settings(x: float, y: float) -> Tuple[int, int, int, int]:
    d = math.hypot(x, y)
    ang = math.atan2(y, x)
    teeth = 0.72 + 0.12 * math.cos(6.0 * ang)
    if 0.26 <= d <= teeth:
        return (203, 213, 225, 255)
    return (0, 0, 0, 0)


def main() -> None:
    icons = {
        "ui_icon_money.png": icon_money,
        "ui_icon_time.png": icon_time,
        "ui_icon_company.png": icon_company,
        "ui_icon_employee.png": icon_employee,
        "ui_icon_product.png": icon_product,
        "ui_icon_settings.png": icon_settings,
    }
    for name, fn in icons.items():
        path = os.path.join("assets", "ui", "icons", name)
        write_png(path, SIZE, SIZE, render_icon(fn))
        print(f"Generated {path} ({os.path.getsize(path)} bytes)")


if __name__ == "__main__":
    main()
