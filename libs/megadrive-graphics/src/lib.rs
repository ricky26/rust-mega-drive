#![no_std]

use core::mem::MaybeUninit;

use megadrive_sys::vdp::{Sprite, VDP};

pub mod default_ascii;

const MAX_SPRITES: usize = 80;

/// A renderer which assists with programming the VDP.
pub struct Renderer {
    num_sprites: u16,
    sprites: [Sprite; MAX_SPRITES],
}

impl Renderer {
    /// Allocate a new renderer.
    pub fn new() -> Renderer {
        Renderer {
            sprites: unsafe { MaybeUninit::uninit().assume_init() },
            num_sprites: 0,
        }
    }

    /// Clear the sprite buffer and prepare for rendering.
    pub fn clear(&mut self) {
        self.num_sprites = 0;
    }

    /// Render the sprite buffer to the screen.
    pub fn render(&mut self, vdp: &mut VDP) {
        let num_sprites = self.num_sprites as usize;
        let sprites = &mut self.sprites[..num_sprites];

        for (idx, s) in sprites.iter_mut().enumerate() {
            let next = if idx < num_sprites - 1 {
                (idx + 1) as u8
            } else {
                0
            };
            s.link = next;
        }

        vdp.set_sprites(0, sprites);
    }

    /// Add a sprite to the draw queue.
    pub fn draw_sprite(&mut self, s: Sprite) {
        let idx = self.num_sprites as usize;
        self.num_sprites += 1;
        self.sprites[idx] = s;
    }
}
