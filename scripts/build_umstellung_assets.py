#!/usr/bin/env python3
"""
Bake the Umstellungssatz text-image + the Umstellungsknospe logo into a single
combined PNG, one per language x variant, for the Knospe label preview.

Layout: logo on the LEFT, text-image on the RIGHT, both normalized to a common
height, vertically centered, separated by a small transparent gap — the mandatory
Umstellungssatz sits to the right of the free-standing Knospe logo, as one flat image.

Inputs (per language L in {de, fr, it, en}):
  assets/logos/src/umstellungssatz-<L>-CH.png   raw official text-image (user-provided)
  assets/logos/umstellungsknospe-<L>-CH.png             logo, regular (Swiss cross)
  assets/logos/umstellungsknospe-import-<L>-CH.png      logo, import (no cross)

Outputs:
  assets/logos/umstellungsknospe-satz-<L>-CH.png
  assets/logos/umstellungsknospe-import-satz-<L>-CH.png

If a raw text-image is missing, a PLACEHOLDER is rendered from the known
Umstellungssatz string so the build/layout can be verified before the official
graphics are downloaded. Re-run this script after dropping the official files in
assets/logos/src/ — output paths are identical, so it is a drop-in swap.

Usage:  python3 scripts/build_umstellung_assets.py
"""

import os
import sys

from PIL import Image, ImageDraw, ImageFont

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
LOGO_DIR = os.path.join(ROOT, "assets", "logos")
SRC_DIR = os.path.join(LOGO_DIR, "src")

LANGS = ["de", "fr", "it", "en"]

# Two logo variants sharing the same text-image per language.
VARIANTS = [
    # (logo filename template, output filename template)
    ("umstellungsknospe-{L}-CH.png", "umstellungsknospe-satz-{L}-CH.png"),
    ("umstellungsknospe-import-{L}-CH.png", "umstellungsknospe-import-satz-{L}-CH.png"),
]

# Placeholder Umstellungssatz strings (mirror locales/*.yml -> preview.umstellungssatz).
PLACEHOLDER_TEXT = {
    "de": "Hergestellt im Rahmen der Umstellung auf die biologische Landwirtschaft.",
    "fr": "Produit dans le cadre de la reconversion à l'agriculture biologique.",
    "it": "Prodotto nel quadro della conversione all'agricoltura biologica.",
    "en": "Produced as part of the conversion to organic farming.",
}

# Tunables — proportions can be adjusted here once the official graphics are in.
TARGET_HEIGHT = 260   # common height (px) both text + logo are scaled to
GAP = 24              # transparent gap between text and logo (px)
TEXT_COLOR = (60, 60, 60, 255)
PLACEHOLDER_MAX_WIDTH = 560  # wrap width for placeholder text render (px)


def load_font(size):
    for path in (
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
    ):
        if os.path.exists(path):
            return ImageFont.truetype(path, size)
    return ImageFont.load_default()


def render_placeholder_text(text):
    """Render the Umstellungssatz as dark, word-wrapped text on transparent bg."""
    font = load_font(34)
    # Word-wrap to PLACEHOLDER_MAX_WIDTH.
    words, lines, cur = text.split(), [], ""
    scratch = ImageDraw.Draw(Image.new("RGBA", (1, 1)))
    for w in words:
        trial = (cur + " " + w).strip()
        if scratch.textlength(trial, font=font) <= PLACEHOLDER_MAX_WIDTH:
            cur = trial
        else:
            lines.append(cur)
            cur = w
    if cur:
        lines.append(cur)

    ascent, descent = font.getmetrics()
    line_h = ascent + descent + 6
    width = max((int(scratch.textlength(ln, font=font)) for ln in lines), default=1)
    height = line_h * len(lines)
    img = Image.new("RGBA", (width, height), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)
    for i, ln in enumerate(lines):
        draw.text((0, i * line_h), ln, font=font, fill=TEXT_COLOR)
    return img


def load_text_image(lang):
    path = os.path.join(SRC_DIR, f"umstellungssatz-{lang}-CH.png")
    if os.path.exists(path):
        return Image.open(path).convert("RGBA"), False
    return render_placeholder_text(PLACEHOLDER_TEXT[lang]), True


def scale_to_height(img, height):
    w, h = img.size
    if h == height:
        return img
    return img.resize((max(1, round(w * height / h)), height), Image.LANCZOS)


def combine(text_img, logo_img):
    text_s = scale_to_height(text_img, TARGET_HEIGHT)
    logo_s = scale_to_height(logo_img, TARGET_HEIGHT)
    width = logo_s.width + GAP + text_s.width
    canvas = Image.new("RGBA", (width, TARGET_HEIGHT), (0, 0, 0, 0))
    canvas.alpha_composite(logo_s, (0, 0))
    canvas.alpha_composite(text_s, (logo_s.width + GAP, 0))
    return canvas


def main():
    used_placeholder = False
    made = 0
    for lang in LANGS:
        text_img, is_placeholder = load_text_image(lang)
        used_placeholder = used_placeholder or is_placeholder
        for logo_tpl, out_tpl in VARIANTS:
            logo_path = os.path.join(LOGO_DIR, logo_tpl.format(L=lang))
            if not os.path.exists(logo_path):
                print(f"  SKIP {lang}: missing logo {os.path.relpath(logo_path, ROOT)}")
                continue
            logo_img = Image.open(logo_path).convert("RGBA")
            out = combine(text_img, logo_img)
            out_path = os.path.join(LOGO_DIR, out_tpl.format(L=lang))
            out.save(out_path)
            made += 1
            print(f"  wrote {os.path.relpath(out_path, ROOT)}  ({out.width}x{out.height})")

    print(f"\nDone: {made} combined PNG(s).")
    if used_placeholder:
        print(
            "NOTE: one or more text-images were PLACEHOLDERS. Drop the official "
            f"files into {os.path.relpath(SRC_DIR, ROOT)}/umstellungssatz-<lang>-CH.png "
            "and re-run to swap them in."
        )


if __name__ == "__main__":
    sys.exit(main())
