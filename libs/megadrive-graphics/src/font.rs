use megadrive_sys::vdp::{Tile, SpriteSize, VDP, Sprite, TileFlags};
use crate::Renderer;

/// Font struct, having tile data and a size definition
pub struct Font {
    pub tile_data: &'static [Tile],
    pub sprite_size: SpriteSize
}

impl Font {
    /// Uses a minimal setup in order to initialize a new visual display and upload the font tiles
    /// to the 0-index of the tile memory
    pub fn init_default(&self) {
        let mut vdp = VDP::new();
        let start_index= 0;

        vdp.enable_interrupts(false, true, false);
        vdp.enable_display(true);
        vdp.set_tiles(start_index, self.tile_data);
    }

    /// Initialize the font to a particular start index, using an already initialized visual
    /// display
    pub fn init(&self, vdp: &mut VDP, start_index: u16) {
        vdp.enable_interrupts(false, true, false);
        vdp.enable_display(true);
        vdp.set_tiles(start_index, self.tile_data);
    }

    /// Displays text on a
    pub fn blit_text(&self, renderer: &mut Renderer, text: &str, x: u16, y: u16) {
        let mut sprite = Sprite::with_flags(
            TileFlags::for_tile(33, 0),
            // TileFlags::for_tile(char_as_ascii_number as u16, 0),
            self.sprite_size);

        // let x_offset = idx * (self.tile_size as usize & 0b0011 as usize) + 1;
        sprite.x = x + (0 as u16 * 9 );
        sprite.y = y;

        renderer.draw_sprite(sprite);

        // for (idx, char) in text.chars().enumerate() {
        //     // let char_as_ascii_number = char as u32;
        // }
    }
}
