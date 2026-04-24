from pathlib import Path

from PIL import Image, ImageDraw, ImageFilter


ROOT = Path(__file__).resolve().parents[1]
ICONS_DIR = ROOT / "src-tauri" / "icons"
ICONSET_DIR = ICONS_DIR / "icon.iconset"


def draw_icon(size: int) -> Image.Image:
    scale = size / 1024.0
    image = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(image)

    bg_radius = round(224 * scale)
    draw.rounded_rectangle((0, 0, size, size), radius=bg_radius, fill=(244, 244, 244, 255))

    shadow = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    shadow_draw = ImageDraw.Draw(shadow)
    shadow_draw.rounded_rectangle(
        (
            round(476 * scale),
            round(348 * scale),
            round((476 + 220) * scale),
            round((348 + 316) * scale),
        ),
        radius=round(48 * scale),
        fill=(0, 0, 0, 34),
    )
    shadow = shadow.filter(ImageFilter.GaussianBlur(radius=max(2, round(16 * scale))))
    image.alpha_composite(shadow)

    back_box = (
        round(476 * scale),
        round(348 * scale),
        round((476 + 220) * scale),
        round((348 + 316) * scale),
    )
    front_box = (
        round(384 * scale),
        round(256 * scale),
        round((384 + 220) * scale),
        round((256 + 316) * scale),
    )
    radius = round(48 * scale)
    stroke = max(2, round(28 * scale))

    draw.rounded_rectangle(
        back_box,
        radius=radius,
        fill=(221, 221, 221, 255),
        outline=(205, 205, 205, 255),
        width=stroke,
    )
    draw.rounded_rectangle(
        front_box,
        radius=radius,
        fill=(244, 244, 244, 255),
        outline=(20, 20, 20, 255),
        width=stroke,
    )

    return image


def save_png(path: Path, size: int) -> None:
    draw_icon(size).save(path, format="PNG")


def build_iconset() -> None:
    ICONSET_DIR.mkdir(parents=True, exist_ok=True)
    sizes = {
        "icon_16x16.png": 16,
        "icon_16x16@2x.png": 32,
        "icon_32x32.png": 32,
        "icon_32x32@2x.png": 64,
        "icon_128x128.png": 128,
        "icon_128x128@2x.png": 256,
        "icon_256x256.png": 256,
        "icon_256x256@2x.png": 512,
        "icon_512x512.png": 512,
        "icon_512x512@2x.png": 1024,
    }
    for name, size in sizes.items():
        save_png(ICONSET_DIR / name, size)


def main() -> None:
    ICONS_DIR.mkdir(parents=True, exist_ok=True)

    png_targets = {
        "32x32.png": 32,
        "128x128.png": 128,
        "128x128@2x.png": 256,
        "icon.png": 1024,
        "Square30x30Logo.png": 30,
        "Square44x44Logo.png": 44,
        "Square71x71Logo.png": 71,
        "Square89x89Logo.png": 89,
        "Square107x107Logo.png": 107,
        "Square142x142Logo.png": 142,
        "Square150x150Logo.png": 150,
        "Square284x284Logo.png": 284,
        "Square310x310Logo.png": 310,
        "StoreLogo.png": 50,
    }

    for name, size in png_targets.items():
        save_png(ICONS_DIR / name, size)

    build_iconset()

    icon_ico = draw_icon(1024)
    icon_ico.save(
        ICONS_DIR / "icon.ico",
        format="ICO",
        sizes=[(16, 16), (24, 24), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)],
    )


if __name__ == "__main__":
    main()
