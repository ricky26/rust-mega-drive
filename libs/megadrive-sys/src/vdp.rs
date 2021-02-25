use core::ptr::{read_volatile, write_volatile};
use core::ops::{Deref, DerefMut};

const REG_VDP_BASE: usize = 0xc00000;
const REG_VDP_DATA16: *mut u16 = REG_VDP_BASE as _;
const REG_VDP_CONTROL16: *mut u16 = (REG_VDP_BASE + 4) as _;

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

    pub const DMA_LEN_L: u8 = 0x93;
    pub const DMA_LEN_H: u8 = 0x94;
    pub const DMA_SRC_L: u8 = 0x95;
    pub const DMA_SRC_M: u8 = 0x96;
    pub const DMA_SRC_H: u8 = 0x97;

    pub const VRAM_SIZE: u32 = 0x10000;
    pub const CRAM_SIZE: u16 = 128;
    pub const VSRAM_SIZE: u16 = 80;
}

fn flag_32(v: u32, b: bool) -> u32 {
    if b { v } else { 0 }
}

/// A struct representing the various segments of VRAM available on the VDP.
#[derive(Clone, Copy)]
pub enum AddrKind {
    VRAM,
    CRAM,
    VSRAM,
}

/// A struct representing where the window is drawn instead of plane A for an axis.
///
/// For example x: After(10), would make the window render to the right of tile 10 onwards.
#[derive(Copy, Clone, Debug)]
pub enum WindowDivide {
    Before(u8),
    After(u8),
}

impl WindowDivide {
    fn reg_value(self) -> u8 {
        match self {
            WindowDivide::Before(v) => v & 0x1f,
            WindowDivide::After(v) => 0x80 | (v & 0x1f),
        }
    }
}

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

/// The size of the planes in tiles.
#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum ScrollSize {
    Cell32 = 0b00,
    Cell64 = 0b01,
    Cell128 = 0b11,
}

const TILE_FLAG_PRIORITY: u16 = 0x8000;
const TILE_FLAG_FLIP_H: u16 = 0x800;
const TILE_FLAG_FLIP_V: u16 = 0x1000;

/// A struct representing the display flags of a single tile.
///
/// This is shared between sprite definitions and tiles rendered on one of the 3
/// render planes.
#[derive(Clone, Copy, Debug)]
pub struct TileFlags(u16);

impl TileFlags {
    /// Create a new flag set for a given tile index.
    pub fn for_tile(tile_idx: u16, palette: u8) -> TileFlags {
        TileFlags(0)
            .set_tile_index(tile_idx)
            .set_palette(palette)
    }

    /// Get the tile index these flags refer to.
    pub fn tile_index(self) -> u16 { self.0 & 0x7ff }

    /// Set the tile index for these flags.
    pub fn set_tile_index(self, tile_index: u16) -> TileFlags {
        TileFlags((self.0 & 0xf800) | (tile_index & 0x7ff))
    }

    /// Get the palette index these flags use.
    pub fn palette(self) -> u8 {
        ((self.0 >> 13) & 3) as u8
    }

    /// Set the palette used by these flags.
    pub fn set_palette(self, palette: u8) -> TileFlags {
        TileFlags((self.0 & 0x9fff) | (((palette & 3) as u16) << 13))
    }

    /// Returns true if this tile will be rendered with priority.
    pub fn priority(self) -> bool { (self.0 & TILE_FLAG_PRIORITY) != 0 }

    /// Configure whether these flags render tiles with priority.
    pub fn set_priority(self, p: bool) -> TileFlags {
        TileFlags(if p {
            self.0 | TILE_FLAG_PRIORITY
        } else {
            self.0 & !TILE_FLAG_PRIORITY
        })
    }

    /// Returns true if this tile is flipped horizontally.
    pub fn flip_h(self) -> bool { (self.0 & TILE_FLAG_FLIP_H) != 0 }

    /// Set whether these flags will render horizontally flipped tiles.
    pub fn set_flip_h(self, p: bool) -> TileFlags {
        TileFlags(if p {
            self.0 | TILE_FLAG_FLIP_H
        } else {
            self.0 & !TILE_FLAG_FLIP_H
        })
    }

    /// Returns true if this tile is flipped vertically.
    pub fn flip_v(self) -> bool { (self.0 & TILE_FLAG_FLIP_V) != 0 }

    /// Set whether these flags will render vertically flipped tiles.
    pub fn set_flip_v(self, p: bool) -> TileFlags {
        TileFlags(if p {
            self.0 | TILE_FLAG_FLIP_V
        } else {
            self.0 & !TILE_FLAG_FLIP_V
        })
    }
}

impl Default for TileFlags {
    fn default() -> Self {
        TileFlags(0)
    }
}


/// A typedef for tile contents.
pub type Tile = [u8; 32];

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

/// A representation of the hardware sprites supported by the Mega Drive VDP.
#[repr(C)]
#[derive(Clone, Debug)]
pub struct Sprite {
    pub y: u16,
    pub size: SpriteSize,
    pub link: u8,
    pub flags: TileFlags,
    pub x: u16,
}

impl Sprite {
    /// Create a new sprite with the given rendering flags.
    pub fn with_flags(flags: TileFlags, size: SpriteSize) -> Self {
        Sprite {
            y: 0,
            size,
            link: 0,
            flags,
            x: 0,
        }
    }

    /// Fetch the rendering flags for this sprite.
    pub fn flags(&self) -> TileFlags { self.flags }

    /// Get a mutable reference to this sprite's rendering flags.
    pub fn flags_mut(&mut self) -> &mut TileFlags { &mut self.flags }

    /// Set the rendering flags for this sprite.
    pub fn set_flags(&mut self, flags: TileFlags) { self.flags = flags; }
}

impl Default for Sprite {
    fn default() -> Self {
        Sprite {
            y: 0,
            size: SpriteSize::Size1x1,
            link: 0,
            flags: TileFlags::default(),
            x: 0,
        }
    }
}

impl Deref for Sprite {
    type Target = TileFlags;

    fn deref(&self) -> &Self::Target {
        &self.flags
    }
}

impl DerefMut for Sprite {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.flags
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
        self.read_state_raw();

        // Initialise mode.
        self.modify_mode(!0, self.mode);
        self.set_plane_a_address(self.plane_a_base);
        self.set_plane_b_address(self.plane_b_base);
        self.set_sprite_address(self.sprites_base);
        self.set_window_base(self.window_base);
        self.set_scroll_base(self.scroll_h_base);

        self.set_increment(2);
        self.set_plane_size(ScrollSize::Cell32, ScrollSize::Cell32);
        self.set_window(WindowDivide::Before(0), WindowDivide::Before(0));
        self.set_background(0, 0);
        self.set_h_interrupt_interval(0xff);

        // Wipe RAM. This should not be strictly necessary since we should
        // write it as we use it and does have a slight performance penalty.
        self.dma_set(AddrKind::VRAM, 0, 0, 0xffff);//registers::VRAM_SIZE as u16);
        self.dma_set(AddrKind::CRAM, 0, 0, registers::CRAM_SIZE);
        self.dma_set(AddrKind::VSRAM, 0, 0, registers::VSRAM_SIZE);

        // Default the palette
        self.set_palette(0, &DEFAULT_PALETTE);
    }

    /// Read the VDP status register.
    pub fn read_state_raw(&self) -> u16 {
        unsafe {
            read_volatile(REG_VDP_CONTROL16)
        }
    }

    /// Set a single VDP register.
    ///
    /// This can cause the VDP to become out of sync with our state caching.
    /// Where possible it is best to use the specific methods in `VDP`.
    pub fn set_register(&mut self, reg: u8, value: u8) {
        let v = ((reg as u16) << 8) | (value as u16);
        unsafe { write_volatile(REG_VDP_CONTROL16, v) };
    }

    /// Set the address increment on write.
    ///
    /// This can be used to configure how many bytes are written per
    /// data write.
    pub fn set_increment(&mut self, incr: u8) {
        self.set_register(registers::INCR, incr);
    }

    /// Write data to VRAM at the current write address.
    pub fn write_data(&mut self, data: u16) {
        unsafe { write_volatile(REG_VDP_DATA16, data) };
    }

    fn set_addr_raw(&mut self, kind: AddrKind, ptr: u16, dma: bool) {
        let ctrl = match kind {
            AddrKind::VRAM => 0b00001,
            AddrKind::CRAM => 0b00011,
            AddrKind::VSRAM => 0b00101,
        };
        let dma_flag = if dma { 0x80 } else { 0 };
        let hi = ((ptr >> 14) & 3) | ((ctrl >> 2) << 4) | dma_flag;
        let lo = (ptr & 0x3fff) | (ctrl << 14);

        unsafe {
            if dma {
                static mut SCRATCH: [u16; 2] = [0, 0];
                write_volatile(&mut SCRATCH[0], lo);
                write_volatile(&mut SCRATCH[1], hi);
                write_volatile(REG_VDP_CONTROL16, read_volatile(&SCRATCH[0]));
                write_volatile(REG_VDP_CONTROL16, read_volatile(&SCRATCH[1]));
            } else {
                write_volatile(REG_VDP_CONTROL16, lo);
                write_volatile(REG_VDP_CONTROL16, hi);
            }
        }
    }

    /// Set the VRAM write address.
    ///
    /// This will be incremented after every write via `write_data`.
    pub fn set_address(&mut self, kind: AddrKind, ptr: u16) {
        self.set_addr_raw(kind, ptr, false);
    }

    fn wait_for_dma(&self) {
        unsafe {
            while read_volatile(REG_VDP_CONTROL16) & 2 != 0 {}
        }
    }

    /// Upload memory from ROM or RAM to VRAM.
    pub fn dma_upload(&mut self, kind: AddrKind, dst_addr: u16, src_addr: *const (), length: u16) {
        let mut length = length as u32;
        let mut src_addr = ((src_addr as u32) >> 1) & 0x7fffff;
        let mut dst_addr = dst_addr as u32;

        self.enable_dma(true);
        while length > 0 {
            let this_block = (0x20000 - (0x1ffff & src_addr)).min(length);

            self.set_register(registers::DMA_LEN_L, this_block as u8);
            self.set_register(registers::DMA_LEN_H, (this_block >> 8) as u8);
            self.set_register(registers::DMA_SRC_L, src_addr as u8);
            self.set_register(registers::DMA_SRC_M, (src_addr >> 8) as u8);
            self.set_register(registers::DMA_SRC_H, (src_addr >> 16) as u8);
            self.set_addr_raw(kind, dst_addr as u16, true);
            self.wait_for_dma();

            dst_addr += this_block;
            src_addr += this_block;
            length -= this_block;
        }
        self.enable_dma(false);
    }

    /// Fill VRAM memory with the given byte.
    ///
    /// Technically the VDP supports doing this in both byte and word forms, but the word
    /// form seems to not work as expected.
    pub fn dma_set(&mut self, kind: AddrKind, dst_addr: u16, fill: u8, length: u16) {
        self.enable_dma(true);
        self.set_increment(1);
        self.set_register(registers::DMA_LEN_L, length as u8);
        self.set_register(registers::DMA_LEN_H, (length >> 8) as u8);
        self.set_register(registers::DMA_SRC_H, 0x80);
        self.set_addr_raw(kind, dst_addr, true);
        self.write_data(fill as u16);
        self.wait_for_dma();
        self.enable_dma(false);
        self.set_increment(2);
    }

    /// Copy from VRAM to VRAM.
    pub fn dma_copy(&mut self, kind: AddrKind, dst_addr: u16, src_addr: u16, length: u16) {
        self.enable_dma(true);
        self.set_register(registers::DMA_LEN_L, length as u8);
        self.set_register(registers::DMA_LEN_H, (length >> 8) as u8);
        self.set_register(registers::DMA_SRC_L, src_addr as u8);
        self.set_register(registers::DMA_SRC_M, (src_addr >> 8) as u8);
        self.set_register(registers::DMA_SRC_H, 0xc0);
        self.set_addr_raw(kind, dst_addr, true);
        self.wait_for_dma();
        self.enable_dma(false);
    }

    /// Modify the MODE registers.
    ///
    /// This takes a mask of bits to replace and their new values.
    /// The integer is formatted with MODE_4 being the highest 8 bits, down to
    /// MODE_1 being the lowest.
    pub fn modify_mode(&mut self, mask: u32, set: u32) {
        self.mode = (self.mode & !mask) | (set & mask);

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

    /// Fetch the framerate of the VDP.
    pub fn framerate(&self) -> u8 {
        if super::version().is_pal() {
            50
        } else {
            60
        }
    }

    /// Fetch the current operating resolution.
    pub fn resolution(&self) -> (u16, u16) {
        let w = if (self.mode & 0x1000000) != 0 {
            320
        } else {
            256
        };
        let h = if (self.mode & 0x800) != 0 {
            240
        } else {
            224
        };

        (w, h)
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

    /// Enable the increased resolution 40x30-cell mode.
    ///
    /// Vertical 30-cell mode is only available on PAL systems.
    pub fn set_resolution(&mut self, h: bool, v: bool) {
        self.modify_mode(0x81000800,
                         flag_32(0x800, v) | flag_32(0x81000000, h));
    }

    /// Enable DMA transfer.
    fn enable_dma(&mut self, enabled: bool) {
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

    /// Set the size of the tile planes (plane A, plane B and the window plane).
    pub fn set_plane_size(&mut self, x: ScrollSize, y: ScrollSize) {
        let v = (x as u8) | ((y as u8) << 4);
        self.set_register(registers::SIZE, v);
    }

    /// Configure the address for the plane A tile map.
    ///
    /// This should ideally be set before the display is enabled if it is to be changed.
    pub fn set_plane_a_address(&mut self, address: u16) {
        self.plane_a_base = address;
        self.set_register(registers::PLANE_A, ((self.plane_a_base >> 6) & 0x38) as u8);
    }

    /// Configure the address for the plane B tile map.
    ///
    /// This should ideally be set before the display is enabled if it is to be changed.
    pub fn set_plane_b_address(&mut self, address: u16) {
        self.plane_b_base = address;
        self.set_register(registers::PLANE_B, (self.plane_b_base >> 9) as u8);
    }

    /// Set the base address for the sprite table.
    pub fn set_sprite_address(&mut self, address: u16) {
        self.sprites_base = address;
        self.set_register(registers::SPRITE, (self.sprites_base >> 9) as u8);
    }

    /// Set the base address for the window plane.
    pub fn set_window_base(&mut self, address: u16) {
        self.window_base = address;
        self.set_register(registers::WINDOW, ((self.window_base >> 6) & 0x38) as u8);
    }

    /// Set the base address for the scrolling matrix.
    pub fn set_scroll_base(&mut self, address: u16) {
        self.scroll_h_base = address;
        self.set_register(registers::HSCROLL, (self.scroll_h_base >> 6) as u8);
    }

    /// Configure where the window is drawn instead of plane A.
    pub fn set_window(&mut self, x: WindowDivide, y: WindowDivide) {
        self.set_register(registers::WINX, x.reg_value());
        self.set_register(registers::WINY, y.reg_value());
    }

    /// Set the palette entry for the background colour.
    pub fn set_background(&mut self, palette: u8, colour: u8) {
        self.set_register(registers::BG_COLOUR, ((palette & 3) << 4) | colour);
    }

    /// Set one of the 4 configurable palettes.
    pub fn set_palette(&mut self, index: usize, palette: &[u16; 16]) {
        assert!(index < 4, "only 4 palettes");

        let src = palette.as_ptr() as *const ();
        let len = (palette.len() * core::mem::size_of::<u16>()) as u16;
        self.dma_upload(AddrKind::CRAM,
                        (index as u16) << 5,
                        src,
                        len);
    }

    /// Set the contents of some tiles in VRAM.
    pub fn set_tiles_iter<T>(&mut self, start_index: usize, tiles: impl Iterator<Item=T>)
        where T: Deref<Target=Tile>
    {
        self.set_address(AddrKind::VRAM, (start_index as u16) << 5);

        for tile in tiles {
            unsafe {
                let ptr: *const u16 = core::mem::transmute(tile.deref());
                for i in 0..16isize {
                    write_volatile(REG_VDP_DATA16, *ptr.offset(i));
                }
            }
        }
    }

    /// Set tiles using DMA.
    ///
    /// This can be faster than `set_tiles()` but is slightly more restricted:
    ///   it has to take a slice.
    pub fn set_tiles(&mut self, start_index: usize, tiles: &[Tile]) {
        let src = tiles.as_ptr() as *const ();
        let len = (tiles.len() * core::mem::size_of::<Tile>()) as u16;
        self.dma_upload(AddrKind::VRAM,
                        (start_index as u16) << 5,
                        src,
                        len);
    }

    /// Set the contents of some sprites in the sprite table.
    pub fn set_sprites_iter<T>(&mut self, first_index: usize, sprites: impl Iterator<Item=T>)
        where T: Deref<Target=Sprite>
    {
        self.set_address(AddrKind::VRAM,
                          ((first_index as u16) << 3) + self.sprites_base);

        for sprite in sprites {
            unsafe {
                let src: *const u16 = core::mem::transmute(sprite.deref());
                for i in 0..4isize {
                    write_volatile(REG_VDP_DATA16, *src.offset(i));
                }
            }
        }
    }

    /// Load sprites into VRAM using DMA.
    ///
    /// This can be faster than `set_sprites()` but is slightly more restricted:
    ///   it has to take a slice.
    pub fn set_sprites(&mut self, first_index: usize, sprites: &[Sprite]) {
        let src = sprites.as_ptr() as *const ();
        let len = (sprites.len() * core::mem::size_of::<Sprite>()) as u16;
        self.dma_upload(AddrKind::VRAM,
                        ((first_index as u16) << 3) + self.sprites_base,
                        src,
                        len);
    }

    /// Set the horizontal scroll for planes A and B.
    pub fn set_h_scroll(&mut self, first_index: usize, values: &[u16]) {
        let src = values.as_ptr() as *const ();
        let len = (values.len() * core::mem::size_of::<u16>()) as u16;
        self.dma_upload(AddrKind::VRAM,
                        ((first_index as u16) << 1) + self.scroll_h_base,
                        src,
                        len);
    }

    /// Set the vertical scroll for planes A and B.
    pub fn set_v_scroll(&mut self, first_index: usize, values: &[u16]) {
        let src = values.as_ptr() as *const ();
        let len = (values.len() * core::mem::size_of::<u16>()) as u16;
        self.dma_upload(AddrKind::VSRAM,
                        (first_index as u16) << 1,
                        src,
                        len);
    }

    /// Set the tile flags for plane A.
    pub fn set_plane_a_flags(&mut self, first_index: usize, values: &[TileFlags]) {
        let src = values.as_ptr() as *const ();
        let len = (values.len() * core::mem::size_of::<TileFlags>()) as u16;
        self.dma_upload(AddrKind::VRAM,
                        ((first_index as u16) << 1) + self.plane_a_base,
                        src,
                        len);
    }

    /// Set the tile flags for plane B.
    pub fn set_plane_b_flags(&mut self, first_index: usize, values: &[TileFlags]) {
        let src = values.as_ptr() as *const ();
        let len = (values.len() * core::mem::size_of::<TileFlags>()) as u16;
        self.dma_upload(AddrKind::VRAM,
                        ((first_index as u16) << 1) + self.plane_b_base,
                        src,
                        len);
    }

    /// Set the tile flags for the window plane.
    pub fn set_window_flags(&mut self, first_index: usize, values: &[TileFlags]) {
        let src = values.as_ptr() as *const ();
        let len = (values.len() * core::mem::size_of::<TileFlags>()) as u16;
        self.dma_upload(AddrKind::VRAM,
                        ((first_index as u16) << 1) + self.window_base,
                        src,
                        len);
    }
}
