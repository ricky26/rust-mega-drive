use core::ptr::{read_volatile, write_volatile};

const REG_VDP_BASE: usize = 0xc00000;
const REG_VDP_DATA16: *mut u16 = REG_VDP_BASE as _;
const REG_VDP_CONTROL16: *mut u16 = (REG_VDP_BASE + 4) as _;
const REG_VDP_CONTROL32: *mut u32 = (REG_VDP_BASE + 4) as _;

const DEFAULT_PALETTE: [u16; 16] = [
    0xF0F, 0x000, 0xFFF, 0xF00, 0x0F0, 0x00F, 0x0FF, 0xFF0,
    0x000, 0x000, 0x000, 0x000, 0x000, 0x000, 0x000, 0x000,
];

pub mod registers {
    pub const MODE_1: u8 = 0x80;
    pub const MODE_2: u8 = 0x81;
    pub const MODE_3: u8 = 0x8b;
    pub const MODE_4: u8 = 0x8c;

    pub const PLANE_A: u8 = 0x82;
    pub const PLANE_B: u8 = 0x84;
    pub const SPRITE: u8 = 0x85;
    pub const WINDOW: u8 = 0x83;
    pub const HSCROLL: u8 = 0x8d;

    pub const SIZE: u8 = 0x90;
    pub const WINX: u8 = 0x91;
    pub const WINY: u8 = 0x92;
    pub const INCR: u8 = 0x8f;
    pub const BG_COLOUR: u8 = 0x87;
    pub const HBLANK_RATE: u8 = 0x8a;

    //pub const DMA_LEN_L: u8 = 0x93;
    //pub const DMA_LEN_H: u8 = 0x94;
    //pub const DMA_SRC_L: u8 = 0x95;
    //pub const DMA_SRC_M: u8 = 0x96;
    //pub const DMA_SRC_H: u8 = 0x97;

    pub const VRAM_SIZE: usize = 65536;
    pub const CRAM_SIZE: usize = 128;
    pub const VSRAM_SIZE: usize = 80;
}

enum AddrKind {
    VRAM,
    CRAM,
    VSRAM,
}

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub enum SpriteSize {
    Size1x1 = 0b0000,
    Size2x1 = 0b0100,
    Size3x1 = 0b1000,
    Size4x1 = 0b1100,
    Size1x2 = 0b0001,
    Size2x2 = 0b0101,
    Size3x2 = 0b1001,
    Size4x2 = 0b1101,
    Size1x3 = 0b0010,
    Size2x3 = 0b0110,
    Size3x3 = 0b1010,
    Size4x3 = 0b1110,
    Size1x4 = 0b0011,
    Size2x4 = 0b0111,
    Size3x4 = 0b1011,
    Size4x4 = 0b1111,
}

impl SpriteSize {
    /// Get the `SpriteSize` given the width and height of the sprite in tiles.
    pub fn for_size(w: u8, h: u8) -> SpriteSize {
        assert!((w <= 4) && (h <= 4), "invalid sprite size");
        unsafe { core::mem::transmute((w - 1) << 2 | (h - 1)) }
    }
}

const SPRITE_FLAG_PRIORITY: u16 = 0x8000;
const SPRITE_FLAG_FLIP_H: u16 = 0x800;
const SPRITE_FLAG_FLIP_V: u16 = 0x1000;

/// A representation of the hardware sprites supported by the Mega Drive VDP.
#[repr(C)]
#[derive(Clone, Debug)]
pub struct Sprite {
    pub y: u16,
    pub size: SpriteSize,
    pub link: u8,
    pub flags: u16,
    pub x: u16,
}

impl Sprite {
    /// Create a new sprite with the tile ID and size set.
    pub fn for_tile(first_tile_id: u16, size: SpriteSize) -> Self {
        Sprite {
            y: 0,
            size,
            link: 0,
            flags: first_tile_id & 0x7ff,
            x: 0,
        }
    }

    /// Return the first tile ID this sprite references.
    pub fn first_tile_id(&self) -> u16 {
        self.flags & 0x7ff
    }

    /// Set the tile this sprite refers to.
    pub fn set_first_tile_id(&mut self, first_tile_id: u16) {
        assert_eq!(first_tile_id &! 0x7ff, 0, "tile IDs can only be 11 bits");
        self.flags = (self.flags &! 0x7ff) | first_tile_id;
    }

    /// Get the palette index this sprite uses.
    pub fn palette(&self) -> u8 {
        (self.flags >> 13) as u8
    }

    /// Set the palette index this sprite uses.
    pub fn set_palette(&mut self, palette: u8) {
        assert!(palette < 4, "only 4 palettes available");
        self.flags = (self.flags &! 0xe000) | ((palette as u16) << 13);
    }

    /// Check whether this sprite is high-priority.
    ///
    /// High priority sprites render over the top of low priority planes.
    pub fn high_priority(&self) -> bool {
        (self.flags & SPRITE_FLAG_PRIORITY) != 0
    }

    /// Toggle the priority of this sprite.
    pub fn set_high_priority(&mut self, prio: bool) {
        if prio {
            self.flags |= SPRITE_FLAG_PRIORITY;
        } else {
            self.flags &=! SPRITE_FLAG_PRIORITY;
        }
    }

    /// Check whether this sprite is flipped horizontally.
    pub fn horizontal_flip(&self) -> bool {
        (self.flags & SPRITE_FLAG_FLIP_H) != 0
    }

    /// Set whether this sprite is flipped horizontally.
    pub fn set_horizontal_flip(&mut self, flip: bool) {
        if flip {
            self.flags |= SPRITE_FLAG_FLIP_H;
        } else {
            self.flags &=! SPRITE_FLAG_FLIP_H;
        }
    }

    /// Check whether this sprite is flipped vertically.
    pub fn vertical_flip(&self) -> bool {
        (self.flags & SPRITE_FLAG_FLIP_V) != 0
    }

    /// Toggle whether this sprite is vertically flipped.
    pub fn set_vertical_flip(&mut self, flip: bool) {
        if flip {
            self.flags |= SPRITE_FLAG_FLIP_H;
        } else {
            self.flags &=! SPRITE_FLAG_FLIP_V;
        }
    }
}

impl Default for Sprite {
    fn default() -> Self {
        Sprite {
            y: 0,
            size: SpriteSize::Size1x1,
            link: 0,
            flags: 0,
            x: 0,
        }
    }
}

pub struct VDP;

impl VDP {
    /// Initialise and return the VDP.
    pub fn new() -> VDP {
        let vdp = VDP;
        vdp.init();
        vdp
    }

    fn init(&self) {
        unsafe {
            read_volatile(REG_VDP_CONTROL16);

            // Initialise mode.
            self.set_register(registers::MODE_1, 0x04);
            self.set_register(registers::MODE_2, 0x44);
            self.set_register(registers::MODE_3, 0x00);
            self.set_register(registers::MODE_4, 0x81);

            self.set_register(registers::PLANE_A, 0x30);
            self.set_register(registers::PLANE_B, 0x07);
            self.set_register(registers::SPRITE, 0x78);
            self.set_register(registers::WINDOW, 0x34);
            self.set_register(registers::HSCROLL, 0x3d);

            self.set_register(registers::SIZE, 1);
            self.set_register(registers::WINX, 0);
            self.set_register(registers::WINY, 0);
            self.set_register(registers::INCR, 2);
            self.set_register(registers::BG_COLOUR, 0);
            self.set_register(registers::HBLANK_RATE, 0xFF);

            // Wipe RAM. This should not be strictly necessary since we should
            // write it as we use it and does have a slight performance penalty.
            self.set_addr(AddrKind::VRAM, 0);
            for _ in 0..registers::VRAM_SIZE/2 {
                write_volatile(REG_VDP_DATA16, 0);
            }
            self.set_addr(AddrKind::VSRAM, 0);
            for _ in 0..registers::VSRAM_SIZE/2 {
                write_volatile(REG_VDP_DATA16, 0);
            }
            self.set_addr(AddrKind::CRAM, 0);
            for _ in 0..registers::CRAM_SIZE/2 {
                write_volatile(REG_VDP_DATA16, 0);
            }

            // Default the palette
            self.set_palette(0, &DEFAULT_PALETTE);

            // H
            self.set_tile(1, &[
                0x01, 0x00, 0x01, 0x00,
                0x01, 0x00, 0x01, 0x00,
                0x01, 0x00, 0x01, 0x00,
                0x01, 0x11, 0x11, 0x00,
                0x01, 0x00, 0x01, 0x00,
                0x01, 0x00, 0x01, 0x00,
                0x01, 0x00, 0x01, 0x00,
                0x00, 0x00, 0x00, 0x00,
            ]);
            // E
            self.set_tile(2, &[
                0x01, 0x11, 0x11, 0x00,
                0x01, 0x00, 0x00, 0x00,
                0x01, 0x00, 0x00, 0x00,
                0x01, 0x11, 0x11, 0x00,
                0x01, 0x00, 0x00, 0x00,
                0x01, 0x00, 0x00, 0x00,
                0x01, 0x11, 0x11, 0x00,
                0x00, 0x00, 0x00, 0x00,
            ]);
            // L
            self.set_tile(3, &[
                0x01, 0x00, 0x00, 0x00,
                0x01, 0x00, 0x00, 0x00,
                0x01, 0x00, 0x00, 0x00,
                0x01, 0x00, 0x00, 0x00,
                0x01, 0x00, 0x00, 0x00,
                0x01, 0x00, 0x00, 0x00,
                0x01, 0x11, 0x11, 0x00,
                0x00, 0x00, 0x00, 0x00,
            ]);
            // O
            self.set_tile(4, &[
                0x01, 0x11, 0x11, 0x00,
                0x01, 0x00, 0x01, 0x00,
                0x01, 0x00, 0x01, 0x00,
                0x01, 0x00, 0x01, 0x00,
                0x01, 0x00, 0x01, 0x00,
                0x01, 0x00, 0x01, 0x00,
                0x01, 0x11, 0x11, 0x00,
                0x00, 0x00, 0x00, 0x00,
            ]);
            // W
            self.set_tile(5, &[
                0x01, 0x00, 0x01, 0x00,
                0x01, 0x00, 0x01, 0x00,
                0x01, 0x00, 0x01, 0x00,
                0x01, 0x00, 0x01, 0x00,
                0x01, 0x01, 0x01, 0x00,
                0x01, 0x10, 0x11, 0x00,
                0x01, 0x00, 0x01, 0x00,
                0x00, 0x00, 0x00, 0x00,
            ]);
            // R
            self.set_tile(6, &[
                0x01, 0x10, 0x00, 0x00,
                0x01, 0x01, 0x10, 0x00,
                0x01, 0x00, 0x01, 0x00,
                0x01, 0x00, 0x10, 0x00,
                0x01, 0x11, 0x00, 0x00,
                0x01, 0x01, 0x00, 0x00,
                0x01, 0x00, 0x11, 0x00,
                0x00, 0x00, 0x00, 0x00,
            ]);
            // D
            self.set_tile(7, &[
                0x01, 0x10, 0x00, 0x00,
                0x01, 0x01, 0x10, 0x00,
                0x01, 0x00, 0x01, 0x00,
                0x01, 0x00, 0x01, 0x00,
                0x01, 0x00, 0x01, 0x00,
                0x01, 0x01, 0x10, 0x00,
                0x01, 0x10, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00,
            ]);
            // !
            self.set_tile(8, &[
                0x01, 0x00, 0x00, 0x00,
                0x01, 0x00, 0x00, 0x00,
                0x01, 0x00, 0x00, 0x00,
                0x01, 0x00, 0x00, 0x00,
                0x01, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00,
                0x01, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00,
            ]);

            // HACK: write sprites
            let mut x = 200;
            let y = 200;

            self.set_addr(AddrKind::VRAM, 0xf000);
            let sprites = [1, 2, 3, 3, 4, 0, 5, 4, 6, 3, 7, 8, 9];
            let next = [1, 2, 3, 4, 5, 0, 6, 7, 8, 9, 10, 11];

            for (i, next) in sprites.iter().cloned().zip(next.iter().cloned()) {
                if i == 0 {
                    x += 7;
                    continue;
                }

                let mut sprite = Sprite::for_tile(i, SpriteSize::Size1x1);
                sprite.link = next;
                sprite.y = y;
                sprite.x = x;

                // HACK: it looks like set_sprite() is generating invalid code atm.
                unsafe {
                    write_volatile(REG_VDP_DATA16, sprite.y);
                    write_volatile(REG_VDP_DATA16, ((sprite.size as u16) << 8) | (sprite.link as u16));
                    write_volatile(REG_VDP_DATA16, sprite.flags);
                    write_volatile(REG_VDP_DATA16, sprite.x);
                }

                //self.set_sprite(i as usize, &sprite);
                x += 7;
            }
        }
    }

    /// Directly set a VDP register.
    ///
    /// This can interfere with the display state.
    pub unsafe fn set_register(&self, reg: u8, value: u8) {
        let v = ((reg as u16) << 8) | (value as u16);
        write_volatile(REG_VDP_CONTROL16, v);
    }

    fn set_addr(&self, kind: AddrKind, ptr: u32) {
        let base = match kind {
            AddrKind::VRAM => 0x4000_0000,
            AddrKind::VSRAM => 0x4000_0010,
            AddrKind::CRAM => 0xc000_0000,
        };

        let value = base | ((ptr & 0x3fff) << 16) | ((ptr >> 14) & 3);
        unsafe { write_volatile(REG_VDP_CONTROL32, value) };
    }

    /// Set one of the 4 configurable palettes.
    pub fn set_palette(&self, index: usize, palette: &[u16; 16]) {
        assert!(index < 4, "only 4 palettes");
        self.set_addr(AddrKind::CRAM, (index as u32) << 5);

        unsafe {
            for x in palette.iter().cloned() {
                write_volatile(REG_VDP_DATA16, x);
            }
        }
    }

    /// Set the contents of one of the tiles in VRAM.
    pub fn set_tile(&self, index: usize, tile: &[u8; 32]) {
        self.set_addr(AddrKind::VRAM, (index as u32) << 5);

        unsafe {
            let ptr: *const u16 = core::mem::transmute(&*tile);
            for i in 0..16isize {
                write_volatile(REG_VDP_DATA16, *ptr.offset(i));
            }
        }
    }

    /// Set the contents of a single sprite in the sprite table.
    pub fn set_sprite(&self, index: usize, sprite: &Sprite) {
        self.set_addr(AddrKind::VRAM, ((index as u32) << 3) + 0xf000);

        unsafe {
            let src: *const u16 = core::mem::transmute(sprite);
            for i in 0..4isize {
                write_volatile(REG_VDP_DATA16, *src.offset(i));
            }
        }
    }
}