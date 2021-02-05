use volatile_register::{RW, RO};

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
}

#[repr(C)]
struct Registers {
    data: RW<u16>,
    reserved0: u16,
    control: RW<u16>,
    reserved1: u16,
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
            self.registers().control.read();

            // Initialise mode.
            self.set_register(registers::MODE_1, 0x04);
            self.set_register(registers::MODE_2, 0x04);
            self.set_register(registers::MODE_3, 0x00);
            self.set_register(registers::MODE_4, 0x81);


        }
    }

    pub unsafe fn set_register(&self, reg: u8, value: u8) {
        let v = ((reg as u16) << 8) | (value as u16);
        self.registers().control.write(v);
    }
}