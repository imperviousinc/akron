#!/usr/bin/env python3

from pathlib import Path
from io import BytesIO
from PIL import Image
import fontforge
import ffmpeg
import shutil

def build_hicolor_set(svg_path, output_dir):
    sizes = [128, 150, 16, 192, 22, 24, 256, 310, 32, 36, 44, 48, 512, 64, 72, 96]
    name = svg_path.stem

    for size in sizes:
        size_dir = output_dir / f"{size}x{size}" / "apps"
        size_dir.mkdir(parents=True, exist_ok=True)

        (
            ffmpeg
            .input(str(svg_path))
            .filter('scale', size, size)
            .output(str(size_dir / f"{name}.png"))
            .overwrite_output()
            .run()
        )

    scalable_dir = output_dir / "scalable" / "apps"
    scalable_dir.mkdir(parents=True, exist_ok=True)
    shutil.copy2(svg_path, scalable_dir / f"{name}.svg")

def build_rgba(image_path, rgba_path):
    with Image.open(image_path) as img:
        img = img.convert('RGBA')
        rgba_data = img.tobytes()

    with open(rgba_path, 'wb') as f:
        f.write(rgba_data)

def build_icons_font(icons_path, font_path, rs_path, font_name):
    font = fontforge.font()
    font.fontname = font_name
    font.familyname = font_name
    font.fullname = font_name
    font.em = 1000
    icons = []

    for i, svg_file in enumerate(icons_path.glob('*.svg')):
        name = ''.join(x.title() for x in svg_file.stem.split('-'))
        char = 0xE000 + i
        glyph = font.createChar(char)
        glyph.importOutlines(str(svg_file))
        glyph.width = 1000
        icons.append((name, char))

    with open(rs_path, "w") as f:
        f.write(f"pub const FONT: iced::Font = iced::Font::with_name(\"{font_name}\");\n")
        f.write("pub enum Icon {\n")
        f.write(''.join(f"    {name},\n" for (name, char) in icons))
        f.write("}\n")
        f.write("impl Icon {\n")
        f.write("    pub fn as_char(&self) -> char {\n")
        f.write("        match self {\n")
        f.write(''.join(f"            Icon::{name} => '\\u{{{char:04X}}}',\n" for (name, char) in icons))
        f.write("        }\n")
        f.write("    }\n")
        f.write("}\n")

    font.generate(str(font_path))
    font.close()

if __name__ == "__main__":
    assets_dir = Path(__file__).parent

    build_hicolor_set(
        assets_dir / "akron.svg",
        assets_dir / "hicolor",
    )

    build_rgba(
        assets_dir / "hicolor" / "64x64" / "apps" / "akron.png",
        assets_dir / "akron.rgba",
    )

    build_icons_font(
        assets_dir / "icons",
        assets_dir / "icons.ttf",
        assets_dir / "icons.rs",
        "icons",
    )
