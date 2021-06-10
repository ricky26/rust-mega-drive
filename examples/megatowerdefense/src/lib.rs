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
        // https://github.com/Stephane-D/SGDK/blob/908926201af8b48227be4dbc8fbb0d5a18ac971b/src/tools.c#L36
        let hv_counter = read_volatile(&GFX_HVCOUNTER_PORT) as u16;
        self.current_rand ^= (self.current_rand >> 1) ^ hv_counter;
        self.current_rand ^= self.current_rand << 1;
        self.current_rand
    }
}

fn upload_graphics(vdp: &mut VDP) {
    // Load graphics.
    static TILE_DATA: &'static [Tile] = &[
        // 0
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
    let mut vdp = VDP::new();
    let mut rng = PseudoRng::from_seed(42);
    upload_graphics(&mut vdp);

    let resolution = vdp.resolution();
    let half_screen_width = (resolution.0 >> 1) as i16;
    let half_screen_height = (resolution.1 >> 1) as i16;
    let game_border = 9;
    let half_border_width = half_screen_width - game_border;
    let half_border_height = half_screen_height - game_border;

    let x_off = 128 + half_screen_width;
    let y_off = 128 + half_screen_height;

    vdp.enable_interrupts(false, true, false);
    vdp.enable_display(true);
    let mut frame = 0u16;

    loop {
        renderer.clear();
        controllers.update();

        unsafe {
            let random_number = rng.random();

            let coin_flip = random_number & 1; // mask with 1, so either 0 or 1

            let mut sprite = Sprite::with_flags(
                TileFlags::for_tile(coin_flip + 1, 0), //not working: test with id 1
                SpriteSize::Size1x1);

            sprite.x = x_off as u16;
            sprite.y = y_off as u16;
            renderer.draw_sprite(sprite);
        }

        frame = (frame + 1) & 0x7fff;
        renderer.render(&mut vdp);
        // vsync
        wait_for_vblank();
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
