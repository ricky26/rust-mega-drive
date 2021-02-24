use core::ptr::{read_volatile, write_volatile};
use core::ops::Deref;

const REG_VDP_BASE: usize = 0xc00000;
const REG_VDP_DATA16: *mut u16 = REG_VDP_BASE as _;
const REG_VDP_CONTROL16: *mut u16 = (REG_VDP_BASE + 4) as _;
const REG_VDP_CONTROL32: *mut u32 = (REG_VDP_BASE + 4) as _;

const DEFAULT_PALETTE: [u16; 16] = [
    0x000, 0xFFF, 0xF00, 0x0F0, 0x00B, 0xFF0, 0xF0F, 0x0FF,
    0x666, 0xBBB, 0x800, 0x080, 0x008, 0x880, 0x808, 0x088,
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

fn flag_32(v: u32, b: bool) -> u32 {
    if b { v } else { 0 }
}

enum AddrKind {
    VRAM,
    CRAM,
    VSRAM,
}

/// A typedef for tile contents.
type Tile = [u8; 32];

/// An enumeration of valid sprite sizes in tiles.
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

/// This enumeration is for configuring how vertical scrolling works.
#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum VScrollMode {
    FullScroll = 0,
    DoubleCellScroll = 1,
}

/// This enumeration is for configuring how horizontal scrolling works.
#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum HScrollMode {
    FullScroll = 0b00,
    CellScroll = 0b10,
    LineScroll = 0b11,
}

/// The interlacing rendering mode.
#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum InterlaceMode {
    None = 0b00,
    Interlace = 0b01,
    DoubleRes = 0b11,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum ScrollSize {
    Cell32 = 0b00,
    Cell64 = 0b01,
    Cell128 = 0b11,
}

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

pub struct VDP {
    mode: u32,
    sprites_base: u16,
    plane_a_base: u16,
    plane_b_base: u16,
    scroll_h_base: u16,
    window_base: u16,
}

impl VDP {
    /// Initialise and return the VDP.
    pub fn new() -> VDP {
        let mut vdp = VDP {
            mode: 0x81000404,
            sprites_base: 0xf000,
            plane_a_base: 0xc00,
            plane_b_base: 0xe00,
            scroll_h_base: 0xf40,
            window_base: 0xd00,
        };
        vdp.init();
        vdp
    }

    fn init(&mut self) {
        self.read_state();

        // Initialise mode.
        self.modify_mode(!0, self.mode);

        self.set_register(registers::PLANE_A, ((self.plane_a_base >> 6) & 0x38) as u8);
        self.set_register(registers::PLANE_B, (self.plane_b_base >> 9) as u8);
        self.set_register(registers::SPRITE, (self.sprites_base >> 9) as u8);
        self.set_register(registers::WINDOW, ((self.window_base >> 6) & 0x38) as u8);
        self.set_register(registers::HSCROLL, (self.scroll_h_base >> 6) as u8);

        self.set_register(registers::SIZE, 1);
        self.set_register(registers::WINX, 0);
        self.set_register(registers::WINY, 0);
        self.set_register(registers::INCR, 2);
        self.set_register(registers::BG_COLOUR, 0);
        self.set_h_interrupt_interval(0xff);

        // Wipe RAM. This should not be strictly necessary since we should
        // write it as we use it and does have a slight performance penalty.
        unsafe {
            self.set_addr(AddrKind::VRAM, 0);
            for _ in 0..registers::VRAM_SIZE / 2 {
                write_volatile(REG_VDP_DATA16, 0);
            }
            self.set_addr(AddrKind::VSRAM, 0);
            for _ in 0..registers::VSRAM_SIZE / 2 {
                write_volatile(REG_VDP_DATA16, 0);
            }
            self.set_addr(AddrKind::CRAM, 0);
            for _ in 0..registers::CRAM_SIZE / 2 {
                write_volatile(REG_VDP_DATA16, 0);
            }
        }

        // Default the palette
        self.set_palette(0, &DEFAULT_PALETTE);
    }

    fn read_state(&self) -> u16 {
        unsafe {
            read_volatile(REG_VDP_CONTROL16)
        }
    }

    fn set_register(&self, reg: u8, value: u8) {
        let v = ((reg as u16) << 8) | (value as u16);
        unsafe { write_volatile(REG_VDP_CONTROL16, v) };
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

    fn modify_mode(&mut self, mask: u32, set: u32) {
        self.mode = (self.mode &! mask) | (set & mask);

        if mask & 0xff != 0 {
            self.set_register(registers::MODE_1, self.mode as u8);
        }

        if mask & 0xff00 != 0 {
            self.set_register(registers::MODE_2, (self.mode >> 8) as u8);
        }

        if mask & 0xff0000 != 0 {
            self.set_register(registers::MODE_3, (self.mode >> 16) as u8);
        }

        if mask & 0xff000000 != 0 {
            self.set_register(registers::MODE_4, (self.mode >> 24) as u8);
        }
    }

    /// Enable the display.
    ///
    /// Without this set, the entire screen shows the background colour.
    pub fn enable_display(&mut self, enable: bool) {
        self.modify_mode(0x4000, flag_32(0x4000, enable));
    }

    /// Enable the horizontal blanking interrupt.
    ///
    /// The IRQ level on the CPU still needs to be set accordingly to allow the
    /// interrupt to happen.
    ///
    /// `h` triggers an interrupt for every horizontal line drawn.
    /// `v` triggers an interrupt at the start of the vblank period.
    /// `x` triggers an interrupt on the external interrupt.
    pub fn enable_interrupts(&mut self, h: bool, v: bool, x: bool) {
        self.modify_mode(0x82010,
                         flag_32(0x10, h) |
                             flag_32(0x2000, v) |
                             flag_32(0x80000, x));
    }

    /// Stop the HV counter.
    pub fn stop_hv_counter(&mut self, stopped: bool) {
        self.modify_mode(2, flag_32(2, stopped));
    }

    /// Enable the increased resolution 30-cell mode.
    ///
    /// Vertical 30-cell mode is only available on PAL systems.
    pub fn enable_30_cell_mode(&mut self, h: bool, v: bool) {
        self.modify_mode(0x81000800,
            flag_32(0x800, v) | flag_32(0x81000000, h));
    }

    /// Enable DMA transfer.
    pub fn enable_dma(&mut self, enabled: bool) {
        self.modify_mode(0x1000, flag_32(0x1000, enabled));
    }

    /// Configure scrolling mode.
    pub fn set_scroll_mode(&mut self, h: HScrollMode, v: VScrollMode) {
        self.modify_mode(0x30000, ((h as u32) << 16) | ((v as u32) << 18));
    }

    /// Enable shadow / highlight mode.
    pub fn enable_shadow_mode(&mut self, enable: bool) {
        self.modify_mode(0x8000000, flag_32(0x8000000, enable));
    }

    /// Configure interlaced output.
    pub fn set_interlace(&mut self, mode: InterlaceMode) {
        self.modify_mode(0x6000000, (mode as u32) << 25);
    }

    /// Configure how frequently the H-blank interrupt fires.
    pub fn set_h_interrupt_interval(&mut self, interval: u8) {
        self.set_register(registers::HBLANK_RATE, interval);
    }

    /// Set one of the 4 configurable palettes.
    pub fn set_palette(&mut self, index: usize, palette: &[u16; 16]) {
        assert!(index < 4, "only 4 palettes");
        self.set_addr(AddrKind::CRAM, (index as u32) << 5);

        unsafe {
            for x in palette.iter().cloned() {
                write_volatile(REG_VDP_DATA16, x);
            }
        }
    }

    /// Set the contents of some tiles in VRAM.
    pub fn set_tiles<T>(&mut self, start_index: usize, tiles: impl Iterator<Item=T>)
        where T: Deref<Target=Tile>
    {
        self.set_addr(AddrKind::VRAM, (start_index as u32) << 5);

        for tile in tiles {
            unsafe {
                let ptr: *const u16 = core::mem::transmute(tile.deref());
                for i in 0..16isize {
                    write_volatile(REG_VDP_DATA16, *ptr.offset(i));
                }
            }
        }
    }

    /// Set the contents of some sprites in the sprite table.
    pub fn set_sprites<T>(&mut self, first_index: usize, sprites: impl Iterator<Item=T>)
        where T: Deref<Target=Sprite>
    {
        self.set_addr(AddrKind::VRAM, ((first_index as u32) << 3) + (self.sprites_base as u32));

        for sprite in sprites {
            unsafe {
                let src: *const u16 = core::mem::transmute(sprite.deref());
                for i in 0..4isize {
                    write_volatile(REG_VDP_DATA16, *src.offset(i));
                }
            }
        }
    }
}