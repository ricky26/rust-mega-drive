import os.path

from PIL import Image, ImageDraw, ImageFont
# Regarding Pillow license: the Historical Permission Notice and Disclaimer
# https://github.com/python-pillow/Pillow/blob/master/LICENSE
# Since we re-use the "default font" from Pillow, we thought it prudent to mention the license and to declare that we
# believe it's compatible with this project's MIT license.

# chars 1-31 are SOH, STX, ... TAB, LF etc., are non-drawable. char 128 is DEL
# chr(0) is the NUL char, so we'll just render this as a black tile
drawable_chars = [0]
drawable_chars.extend(list(range(32, 128)))


def generate_image_arrays():
    output_path = 'libs/megadrive-graphics/src/default_ascii.rs'

    # Remove in order to overwrite
    if os.path.isfile(output_path):
        os.remove(output_path)

    rust_file = open(output_path, 'at')

    # Write the boilerplate
    rust_file.write("use megadrive_sys::vdp::Tile;\n\n")
    rust_file.write("pub static DEFAULT_FONT: &'static [Tile] = &[\n")

    font = ImageFont.load_default()

    for char_idx in range(128):
        write_char(char_idx, font, rust_file)

    # Closing quote for the Tiles slice
    rust_file.write("];\n")
    rust_file.close()


def write_char(char_idx, font, rust_file):
    image = Image.new('RGB', (8, 16))
    tile_width = 4
    tile_height = 8

    draw = ImageDraw.Draw(image)

    if char_idx in drawable_chars:
        char = chr(char_idx)
    else:
        char = '?'

    draw.text((0, 0), char, font=font)

    # Convert image to 4x8 4-bit-per-pixel hex values
    image = image.resize((tile_width, tile_height)).convert(mode="P", palette=Image.ADAPTIVE, colors=16)
    image_hex_vals = image.tobytes().hex(bytes_per_sep=2)

    rust_file.write(f'    // idx {char_idx}: {char}\n')
    # Write the start of the tile array
    rust_file.write('    [\n')

    for row in range(tile_height):
        # Write the indent
        rust_file.write('        ')

        for column in range(tile_width):
            # Two hex values per pixel
            offset = row * column * 2
            pixel = image_hex_vals[offset:offset + 2]
            rust_file.write(f'0x{pixel}, ')

        rust_file.write('\n')
    # The end of the tile array
    rust_file.write('\n    ],\n')


if __name__ == '__main__':
    generate_image_arrays()
