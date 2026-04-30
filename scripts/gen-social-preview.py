#!/usr/bin/env python3
"""
Generate the GitHub social preview image (1280x640) and the smaller in-README hero.

Run:
    python3 scripts/gen-social-preview.py

Outputs:
    assets/social-preview.png   1280x640  used for repo open-graph image
    assets/hero.png             1600x900  used at top of README (downsampled by GH)
"""

from PIL import Image, ImageDraw, ImageFont
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
ASSETS = ROOT / "assets"
ASSETS.mkdir(exist_ok=True)

BG = (10, 12, 17)            # near-black, matches app dark mode
PANEL = (16, 19, 26)          # slightly lighter
ACCENT = (110, 231, 183)      # teal
ACCENT_DIM = (52, 211, 153)
TEXT = (237, 240, 246)
MUTED = (148, 163, 184)
RED = (251, 113, 133)
LINE = (30, 35, 45)


def find_font(candidates, size):
    for name in candidates:
        for prefix in ("/System/Library/Fonts/", "/System/Library/Fonts/Supplemental/", "/Library/Fonts/"):
            p = Path(prefix) / name
            if p.exists():
                try:
                    return ImageFont.truetype(str(p), size)
                except Exception:
                    pass
    return ImageFont.load_default()


SANS_BOLD = ["Helvetica.ttc", "HelveticaNeue.ttc", "Arial.ttf", "Arial Bold.ttf"]
SANS_REG = ["Helvetica.ttc", "HelveticaNeue.ttc", "Arial.ttf"]
MONO = ["Menlo.ttc", "Monaco.ttf", "Courier.ttc"]


def gradient_dot(d, x, y, r, color, alpha_falloff=0.7):
    for i in range(r, 0, -1):
        a = int(255 * (i / r) ** 2 * alpha_falloff)
        d.ellipse([x - i, y - i, x + i, y + i], fill=(*color, a))


def draw_grid(d, w, h, step=40):
    for x in range(0, w, step):
        d.line([(x, 0), (x, h)], fill=(20, 24, 32), width=1)
    for y in range(0, h, step):
        d.line([(0, y), (w, y)], fill=(20, 24, 32), width=1)


def draw_glow(img, cx, cy, radius, color, intensity=80):
    """Soft radial glow over an existing image, using a separate alpha layer."""
    glow = Image.new("RGBA", img.size, (0, 0, 0, 0))
    gd = ImageDraw.Draw(glow)
    for i in range(radius, 0, -4):
        a = int(intensity * (1 - i / radius) ** 2)
        gd.ellipse([cx - i, cy - i, cx + i, cy + i], fill=(*color, a))
    img.alpha_composite(glow)


def render_social(path, size=(1280, 640)):
    w, h = size
    img = Image.new("RGBA", size, (*BG, 255))
    d = ImageDraw.Draw(img)
    draw_grid(d, w, h, 48)

    # left-side glow
    draw_glow(img, 200, 380, 380, ACCENT_DIM, intensity=55)
    draw_glow(img, w - 240, 200, 320, (96, 165, 250), intensity=40)

    d = ImageDraw.Draw(img)

    # right-side mock waterfall first so we can size the left column to clear it
    panel_w, panel_h = 360, 448
    panel_x = w - panel_w - 56
    panel_y = 96

    left_col_max_x = panel_x - 32  # text must end here

    # tag pill
    pill_text = "v0.1.0   Apache 2.0   MCP 2025-06-18"
    pill_font = find_font(SANS_BOLD, 20)
    pill_w = int(d.textlength(pill_text, font=pill_font)) + 32
    pill_h = 36
    d.rounded_rectangle([72, 72, 72 + pill_w, 72 + pill_h], radius=18, fill=PANEL, outline=LINE)
    d.text((72 + 16, 72 + (pill_h - 20) // 2 - 2), pill_text, fill=MUTED, font=pill_font)

    # title - sized to fit left column
    title_font = find_font(SANS_BOLD, 78)
    d.text((72, 134), "MCP Observatory", fill=TEXT, font=title_font)

    # tagline
    tagline_font = find_font(SANS_BOLD, 34)
    d.text((72, 234), "See every message", fill=ACCENT, font=tagline_font)
    d.text((72, 274), "your AI agents send.", fill=ACCENT, font=tagline_font)

    # subline
    sub_font = find_font(SANS_REG, 22)
    d.text(
        (72, 332),
        "Local-first proxy and trace viewer",
        fill=MUTED,
        font=sub_font,
    )
    d.text(
        (72, 362),
        "for the Model Context Protocol.",
        fill=MUTED,
        font=sub_font,
    )
    d.text((72, 394), "No telemetry.  No signup.  One binary.", fill=MUTED, font=sub_font)

    # install command box - pinned to left col width
    cmd_font = find_font(MONO, 19)
    cmd = "$ mcpobs init && mcpobs start"
    cmd_w = int(d.textlength(cmd, font=cmd_font)) + 40
    box_y = 458
    d.rounded_rectangle([72, box_y, 72 + cmd_w, box_y + 50], radius=10, fill=PANEL, outline=LINE)
    d.text((72 + 20, box_y + 14), cmd, fill=TEXT, font=cmd_font)

    # bottom row: github handle
    foot_font = find_font(SANS_REG, 20)
    d.text((72, h - 60), "github.com/vnmoorthy/mcpobservatory", fill=MUTED, font=foot_font)
    d.rounded_rectangle([panel_x, panel_y, panel_x + panel_w, panel_y + panel_h], radius=14, fill=PANEL, outline=LINE)
    # window chrome
    for i, col in enumerate([(239, 68, 68), (245, 158, 11), (16, 185, 129)]):
        cx = panel_x + 20 + i * 18
        d.ellipse([cx, panel_y + 18, cx + 10, panel_y + 28], fill=col)
    d.text((panel_x + 86, panel_y + 14), "session  ·  filesystem", fill=MUTED, font=find_font(SANS_REG, 14))

    # waterfall lanes
    lanes = [
        ("c2s", "tools/list", 48, ACCENT, 78),
        ("s2c", "result", 144, ACCENT_DIM, 28),
        ("c2s", "tools/call", 196, ACCENT, 132),
        ("s2c", "result", 348, ACCENT_DIM, 36),
        ("c2s", "tools/call", 408, ACCENT, 88),
        ("s2c", "error", 512, RED, 24),
    ]
    method_font = find_font(MONO, 12)
    label_font = find_font(SANS_BOLD, 11)
    base_x = panel_x + 24
    track_x = panel_x + 130
    track_w = panel_w - 152
    for direction, method, start_ms, col, dur_ms in lanes:
        y = panel_y + 60 + (lanes.index((direction, method, start_ms, col, dur_ms))) * 40

        d.rounded_rectangle([base_x, y, base_x + 36, y + 18], radius=4, fill=LINE, outline=None)
        d.text((base_x + 6, y + 3), direction, fill=col, font=label_font)
        d.text((base_x + 44, y + 1), method, fill=TEXT, font=method_font)
        # lane line
        d.line([(track_x, y + 9), (track_x + track_w, y + 9)], fill=LINE, width=1)
        # bar
        bx0 = track_x + int(start_ms / 600 * track_w)
        bx1 = bx0 + max(8, int(dur_ms / 600 * track_w))
        d.rounded_rectangle([bx0, y + 4, bx1, y + 14], radius=4, fill=col)

    img.convert("RGB").save(path, "PNG", optimize=True)
    print(f"wrote {path}  {size[0]}x{size[1]}")


if __name__ == "__main__":
    render_social(ASSETS / "social-preview.png", (1280, 640))
