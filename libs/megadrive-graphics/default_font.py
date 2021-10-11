import math
import os
from typing import TextIO

from PIL import Image, ImageDraw, ImageFont
# Regarding Pillow license: the Historical Permission Notice and Disclaimer
# https://github.com/python-pillow/Pillow/blob/master/LICENSE
# Since we re-use the "default font" from Pillow, we thought it prudent to mention the license and to declare that we
# believe it's compatible with this project's MIT license.

# chars 1-31 are SOH, STX, ... TAB, LF etc., are non-drawable. char 128 is DEL
# chr(0) is the NUL char, which some IDEs don't like in a .rs file 
drawable_chars = list(range(32, 128))

boilerplate = """// This file was generated from the Python default_font.py script, please don't edit directly
use megadrive_sys::vdp::Tile;
use megadrive_sys::vdp::SpriteSize::Size1x1;
use crate::font::Font;

pub static DEFAULT_FONT_1X1: Font = Font { tile_data: TILE_DATA, sprite_size: Size1x1, start_index: 1 };

pub static TILE_DATA: &'static [Tile] = &[
"""


def generate_image_arrays():
    output_path = os.path.join(os.environ['OUT_DIR'], 'default_ascii.rs')

    # Remove in order to overwrite
    if os.path.isfile(output_path):
        os.remove(output_path)

    with open(output_path, 'at') as rust_file:
        # Write the boilerplate imports and inits
        rust_file.write(boilerplate)

        font = ImageFont.load_default()

        for char_idx in range(128):
            write_char(char_idx, font, rust_file)

        # Closing quote for the Tiles slice
        rust_file.write("];\n")


def write_char(char_idx: int, font: ImageFont, rust_file: TextIO):
    tile_width = 8
    tile_height = 8
    image = Image.new('RGB', (tile_width, tile_height))

    draw = ImageDraw.Draw(image)

    if char_idx in drawable_chars:
        char = chr(char_idx)
    else:
        char = '?'

    draw.text(xy=(0, -2), text=char, font=font)
    # Convert to grayscale
    image = image.convert(mode="L")

    rust_file.write(f'    // idx {char_idx}: {char}\n')
    # Write the start of the tile array
    rust_file.write('    [\n')

    for row in range(tile_height):
        # Write the indent
        rust_file.write('        ')

        # Step by two: two pixels are combined into one hex value
        for column in range(tile_width)[::2]:
            pixel1 = image.getpixel((column, row))
            # Scale down the palette from 256 to 8 colors. Offset by 1 since color 0 is transparent.
            pixel1 = math.floor((pixel1 + 1) / 32)

            # Add the second pixel
            pixel2 = image.getpixel((column + 1, row))
            pixel2 = math.floor((pixel2 + 1) / 32)

            rust_file.write(f'0x{pixel1}{pixel2}, ')

        rust_file.write('\n')
    # The end of the tile array
    rust_file.write('    ],\n')


if __name__ == '__main__':
    generate_image_arrays()
