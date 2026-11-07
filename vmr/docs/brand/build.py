#!/usr/bin/env python3
"""Rebuild the vmr logo assets from any source photograph or illustration.

Usage:
    python3 build.py --src cat.jpg --box 120,40,1940,1860
    python3 build.py --src anime.png --box 0,0,1000,1000 --weight 6

Emits logo.png (rounded card: circular medallion + wordmark) and mark.png
(square avatar) next to this script. Pillow only.
"""

from __future__ import annotations

import argparse
import os

from PIL import Image, ImageDraw, ImageFont

CREAM = (251, 245, 234)
INK = (40, 30, 22)
ACCENT = (181, 67, 47)
FONT_CANDIDATES = (
    "/System/Library/Fonts/SFCompactRounded.ttf",
    "/System/Library/Fonts/SFNSRounded.ttf",
    "/System/Library/Fonts/HelveticaNeue.ttc",
    "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf",
)
MARK_SIZE = 512
CARD = (1240, 340)
CARD_RADIUS = 76
MEDALLION = 232


def load_font(size: int):
    for path in FONT_CANDIDATES:
        if os.path.exists(path):
            try:
                return ImageFont.truetype(path, size)
            except OSError:
                continue
    return ImageFont.load_default(size)


def medallion(src: Image.Image, box, size: int, ring: float = 0.028) -> Image.Image:
    """Circle-crop `box` out of `src`, add a white ring, return an RGBA square."""
    ss = 4  # supersample for a clean circle edge
    d = size * ss
    photo = src.crop(box).resize((d, d), Image.LANCZOS).convert("RGB")
    mask = Image.new("L", (d, d), 0)
    ImageDraw.Draw(mask).ellipse((0, 0, d - 1, d - 1), fill=255)
    med = Image.new("RGBA", (d, d), (0, 0, 0, 0))
    med.paste(photo, (0, 0), mask)
    med.putalpha(mask)
    ring_img = Image.new("RGBA", (d, d), (0, 0, 0, 0))
    ImageDraw.Draw(ring_img).ellipse(
        (0, 0, d - 1, d - 1), outline=(255, 255, 255, 255), width=max(2, int(d * ring))
    )
    return Image.alpha_composite(med, ring_img).resize((size, size), Image.LANCZOS)


def build_mark(src: Image.Image, box) -> Image.Image:
    canvas = Image.new("RGBA", (MARK_SIZE, MARK_SIZE), (0, 0, 0, 0))
    ImageDraw.Draw(canvas).ellipse(
        (0, 0, MARK_SIZE - 1, MARK_SIZE - 1), fill=CREAM + (255,)
    )
    med = medallion(src, box, int(MARK_SIZE * 0.90), ring=0.026)
    canvas.alpha_composite(med, ((MARK_SIZE - med.width) // 2, (MARK_SIZE - med.height) // 2))
    return canvas


def build_logo(src: Image.Image, box, weight: int = 5) -> Image.Image:
    w, h = CARD
    card = Image.new("RGBA", (w, h), (0, 0, 0, 0))
    ImageDraw.Draw(card).rounded_rectangle((0, 0, w - 1, h - 1), radius=CARD_RADIUS, fill=CREAM + (255,))

    med = medallion(src, box, MEDALLION)
    draw = ImageDraw.Draw(card)
    font = load_font(176)
    text = "vmr"
    stroke = weight
    bb = draw.textbbox((0, 0), text, font=font, stroke_width=stroke)
    tw, th = bb[2] - bb[0], bb[3] - bb[1]
    gap = 56
    total = med.width + gap + tw + 56
    x0 = (w - total) // 2
    card.alpha_composite(med, (x0, (h - med.height) // 2))

    tx = x0 + med.width + gap
    ty = (h - th) // 2 - bb[1] - 4
    draw.text((tx, ty), text, font=font, fill=INK + (255,), stroke_width=stroke, stroke_fill=INK + (255,))

    dot = 15
    dx = tx + tw + 30
    dy = ty + bb[3] - dot * 2 - 8
    draw.ellipse((dx, dy, dx + dot * 2, dy + dot * 2), fill=ACCENT + (255,))
    return card


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--src", required=True, help="source image")
    ap.add_argument("--box", required=True, help="crop box for the face: L,T,R,B")
    ap.add_argument("--weight", type=int, default=5, help="wordmark stroke width")
    ap.add_argument("--out-dir", default=os.path.dirname(os.path.abspath(__file__)))
    args = ap.parse_args()

    box = tuple(int(v) for v in args.box.split(","))
    src = Image.open(args.src).convert("RGB")

    mark = build_mark(src, box)
    logo = build_logo(src, box, args.weight)
    mark.save(os.path.join(args.out_dir, "mark.png"), optimize=True)
    logo.save(os.path.join(args.out_dir, "logo.png"), optimize=True)
    print(f"mark.png {mark.size}  logo.png {logo.size}")


if __name__ == "__main__":
    main()
