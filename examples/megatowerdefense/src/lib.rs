#![no_std]

use core::panic::PanicInfo;
use core::ptr::{read_volatile, write_volatile};

use megadrive_input::Controllers;
use megadrive_graphics::Renderer;
use megadrive_sys::vdp::{VDP, Sprite, SpriteSize, TileFlags, Tile};

static mut NEW_FRAME: u16     = 0;
const GFX_HVCOUNTER_PORT: u32 = 0xC00008;

extern "C" {
    fn wait_for_interrupt();
}

struct PseudoRng {
    current_rand:  u16,
}

impl PseudoRng {
    // Thank you Stephane Dallongeville!
    pub fn from_seed(seed: u16) -> PseudoRng {
        PseudoRng {
            current_rand: seed ^ 0xD94B // XOR with some val to avoid 0
        }
    }

    pub unsafe fn random(&mut self) -> u16 {
        self.current_rand = self.current_rand >> 1;
        let hv_counter = read_volatile(&GFX_HVCOUNTER_PORT) as u16;
        self.current_rand = self.current_rand ^ hv_counter;
        self.current_rand << 1
    }
}

fn upload_graphics(vdp: &mut VDP) {
    // Load graphics.
    static TILE_DATA: &'static [Tile] = &[
        // H - 8
        [
            0x08, 0x00, 0x08, 0x00,
            0x08, 0x00, 0x08, 0x00,
            0x08, 0x00, 0x08, 0x00,
            0x08, 0x88, 0x88, 0x00,
            0x08, 0x00, 0x08, 0x00,
            0x08, 0x00, 0x08, 0x00,
            0x08, 0x00, 0x08, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ],
        // E - 2
        [
            0x08, 0x88, 0x88, 0x00,
            0x08, 0x00, 0x00, 0x00,
            0x08, 0x00, 0x00, 0x00,
            0x08, 0x88, 0x88, 0x00,
            0x08, 0x00, 0x00, 0x00,
            0x08, 0x00, 0x00, 0x00,
            0x08, 0x88, 0x88, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ],
    ];
    vdp.set_tiles(1, TILE_DATA);
}


#[no_mangle]
pub fn main() -> ! {
    let mut renderer = Renderer::new();
    let mut controllers = Controllers::new();
    let mut rng = PseudoRng::from_seed(42);

    loop {
        renderer.clear();
        controllers.update();

        unsafe {
            let _random_number = rng.random();
        }
    }
}

fn wait_for_vblank() {
    unsafe {
        while read_volatile(&NEW_FRAME) == 0 {
            wait_for_interrupt();
        }
        NEW_FRAME = 0;
    }
}

#[no_mangle]
fn vblank() {
    unsafe { write_volatile(&mut NEW_FRAME, 1) };
}

#[panic_handler]
#[no_mangle]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
