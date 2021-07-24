use megadrive_sys::vdp::{Tile, SpriteSize, VDP, Sprite, TileFlags};
use crate::Renderer;

/// Font struct, having tile data and a size definition
pub struct Font {
    pub tile_data: &'static [Tile],
    pub sprite_size: SpriteSize,
    pub start_index: u16
}

impl Font {
    /// Loads the font to the start index, using an already initialized visual display
    pub fn load(&self, vdp: &mut VDP) {
        vdp.set_tiles(self.start_index, self.tile_data);
    }

    /// Displays text using renderer at position (x, y)
    /// Note remember to call renderer.render() afterwards
    pub fn blit_text(&self, renderer: &mut Renderer, text: &str, x: u16, y: u16) {
        for (idx, byte) in text.as_bytes().into_iter().enumerate() {
            let char_as_tile_idx = *byte as u16 + self.start_index;

            let mut sprite = Sprite::with_flags(
                TileFlags::for_tile(char_as_tile_idx as u16, 0),
                self.sprite_size);
            sprite.x = x + 9 * idx as u16;
            sprite.y = y as u16;
            sprite.set_priority(true);

            renderer.draw_sprite(sprite);
        }
    }
}
