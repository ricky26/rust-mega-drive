use volatile_register::{RW, RO};
use crate::vdp::registers::VRAM_SIZE;

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

#[repr(C)]
struct Registers {
    data_hi: RW<u16>,
    data_lo: RW<u16>,
    control_hi: RW<u16>,
    control_lo: RW<u16>,
    hv_counter: RO<u16>,
}

pub struct VDP {
    registers: *mut Registers,
}

impl VDP {
    pub fn new() -> VDP {
        let vdp = VDP {
            registers: 0xc00000 as *mut Registers,
        };
        vdp.init();
        vdp
    }

    unsafe fn registers(&self) -> &mut Registers {
        &mut *self.registers
    }

    fn init(&self) {
        unsafe {
            self.registers().control_hi.read();

            // Initialise mode.
            self.set_register(registers::MODE_1, 0x04);
            self.set_register(registers::MODE_2, 0x04);
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

            // Wipe RAM.
            self.set_addr(AddrKind::VRAM, 0);
            for _ in 0..registers::VRAM_SIZE/2 {
                self.registers().data_hi.write(0);
            }
            self.set_addr(AddrKind::VSRAM, 0);
            for _ in 0..registers::VSRAM_SIZE/2 {
                self.registers().data_hi.write(0);
            }
            self.set_addr(AddrKind::CRAM, 0);
            for _ in 0..registers::CRAM_SIZE/2 {
                self.registers().data_hi.write(0);
            }

            // Default the palette
            self.set_palette(0, &DEFAULT_PALETTE);

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

            // HACK: write sprite
            self.set_addr(AddrKind::VRAM, 0xf000);
            self.registers().data_hi.write(128);
            self.registers().data_hi.write(1);
            self.registers().data_hi.write(0x8001);
            self.registers().data_hi.write(128);
            self.registers().data_hi.write(256);
            self.registers().data_hi.write(0);
            self.registers().data_hi.write(0x8001);
            self.registers().data_hi.write(256);
        }
    }

    pub unsafe fn set_register(&self, reg: u8, value: u8) {
        let v = ((reg as u16) << 8) | (value as u16);
        self.registers().control_hi.write(v);
    }

    fn set_addr(&self, kind: AddrKind, ptr: u32) {
        let (hi, lo) = match kind {
            AddrKind::VRAM => (0x4000u16, 0u16),
            AddrKind::VSRAM => (0x4000, 0x10),
            AddrKind::CRAM => (0xc000, 0),
        };


        let hi = hi | ((ptr & 0x3fff) as u16);
        let lo = lo | (((ptr >> 14) & 3) as u16);

        unsafe {
            self.registers().control_hi.write(hi);
            self.registers().control_lo.write(lo);
        }
    }

    pub fn set_palette(&self, index: usize, palette: &[u16; 16]) {
        self.set_addr(AddrKind::CRAM, (index as u32) << 5);

        unsafe {
            for x in palette.iter().cloned() {
                self.registers().data_hi.write(x);
            }
        }
    }

    pub fn set_tile(&self, index: usize, tile: &[u8; 32]) {
        self.set_addr(AddrKind::VRAM, ((index as u32) << 5));

        unsafe {
            let ptr: *const u16 = core::mem::transmute(&*tile);
            for i in 0..16 {
                self.registers().data_hi.write(*ptr.offset(i as isize))
            }
        }
    }
}