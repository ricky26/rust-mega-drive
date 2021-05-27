#![no_std]

use core::panic::PanicInfo;
use core::ptr::{read_volatile, write_volatile};

use rand::{Rng, SeedableRng};
use rand::rngs::SmallRng;

use megadrive_input::Controllers;
use megadrive_graphics::Renderer;

static mut NEW_FRAME: u16 = 0;

extern "C" {
    fn wait_for_interrupt();
}

#[no_mangle]
pub fn main() -> ! {
    let mut renderer = Renderer::new();
    let mut controllers = Controllers::new();

    loop {
        renderer.clear();
        controllers.update();

        let mut small_rng = SmallRng::seed_from_u64(42);
        let _random_number: u8 = small_rng.gen_range(0..10);
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
