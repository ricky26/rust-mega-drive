#![no_std]

use core::panic::PanicInfo;
use core::ptr::{read_volatile, write_volatile};

use rand::{Rng, SeedableRng};
use rand::rngs::SmallRng;
use rand::RngCore;

use megadrive_input::{Controllers, Button};
use megadrive_graphics::Renderer;

#[no_mangle]
pub fn main() -> ! {
    let mut renderer = Renderer::new();
    let mut controllers = Controllers::new();
    let mut small_rng = SmallRng::seed_from_u64(42);

    loop {
        renderer.clear();
        controllers.update();

        let random_number = small_rng.next_u32();
    }
}