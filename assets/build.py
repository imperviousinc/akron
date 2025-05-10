#!/usr/bin/python3

import os
from io import BytesIO
from PIL import Image
import fontforge

def build_rgba(image_path, rgba_path):
    with open(image_path, "rb") as f:
        data = BytesIO(f.read())
    img = Image.open(data)
    img = img.convert("RGBA")
    data = img.tobytes()
    with open(rgba_path, "wb") as f:
        f.write(data)

def build_icons_font(icons_path, font_path, rs_path, font_name):
    font = fontforge.font()
    font.fontname = font_name
    font.familyname = font_name
    font.fullname = font_name
    font.em = 1000
    icons = []
    for i, svg in enumerate(os.listdir(icons_path)):
        if not svg.endswith('.svg'):
            continue
        name = ''.join(x.title() for x in svg[:-4].split('-'))
        char = 0xE000 + i
        glyph = font.createChar(char)
        glyph.importOutlines(os.path.join(icons_path, svg))
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
    font.generate(font_path)
    font.close()

if __name__ == "__main__":
    assets_dir = os.path.dirname(os.path.abspath(__file__))
    build_rgba(
        os.path.join(assets_dir, "akron.png"),
        os.path.join(assets_dir, "akron.rgba"),
    )
    build_icons_font(
        os.path.join(assets_dir, "icons"),
        os.path.join(assets_dir, "icons.ttf"),
        os.path.join(assets_dir, "icons.rs"),
        "icons",
    )
