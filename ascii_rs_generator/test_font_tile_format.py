import os
import unittest
from tempfile import TemporaryDirectory

from PIL import ImageFont

from ascii_rs_generator.pillow_default_font import write_char


class TestASCIIFontGenerator(unittest.TestCase):
    def test_write_exclamation_mark_in_white(self):
        """
        Write an exclamation mark to a temp file and
        :return:
        """
        with TemporaryDirectory() as temp_dir:
            output_file = os.path.join(temp_dir, 'test_font_file.rs')

            # Write the file
            with open(output_file, 'at') as f:
                write_char(ord('!'), ImageFont.load_default(), f)

            # Reopen to inspect for testing
            with open(output_file, 'rt') as f:
                rust_file_lines = f.readlines()

            pixel_lines = [l for l in rust_file_lines if '0x' in l]
            with self.subTest('There are eight lines of pixels (since a tile is 8 pixels high'):
                self.assertEqual(len(pixel_lines), 8)

            for line_idx, line in enumerate(pixel_lines):
                # Get rid of whitespace
                line = line.strip()
                pixels = line.split(', ')

                with self.subTest(f'The eight pixels in line {line_idx} are grouped into four groups of two pixels'):
                    self.assertEqual(len(pixels), 4)
