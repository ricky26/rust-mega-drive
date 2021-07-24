#![no_std]
use core::ptr::{read_volatile, write_volatile};

use megadrive_graphics::Renderer;
use megadrive_input::Controllers;
use megadrive_util::rng::PseudoRng;
use megadrive_sys::vdp::VDP;
use megadrive_graphics::default_ascii::DEFAULT_FONT_1X1;

static mut NEW_FRAME: u16 = 0;

#[no_mangle]
pub fn main() -> ! {
    let mut renderer = Renderer::new();
    let mut controllers = Controllers::new();
    let mut vdp = VDP::new();

    let mut rng = PseudoRng::from_seed(42);

    let resolution = vdp.resolution();
    let half_screen_width = resolution.0 >> 1;
    let half_screen_height = resolution.1 >> 1;

    let x_off = 128 + half_screen_width;
    let y_off = 128 + half_screen_height;

    vdp.enable_interrupts(false, true, false);
    vdp.enable_display(true);

    // Load the font tiles
    DEFAULT_FONT_1X1.load(&mut vdp);

    loop {
        renderer.clear();
        controllers.update();

        let random_number = rng.random();
        let flipped = random_number & 1; // mask with 1, so either 0 or 1

        let heads_or_tails_tile_idx = flipped + 1;

        DEFAULT_FONT_1X1.blit_text(&mut renderer, heads_or_tails, x_off, y_off);

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
