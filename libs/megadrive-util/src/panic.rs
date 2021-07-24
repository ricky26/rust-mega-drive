use core::panic::PanicInfo;
use core::ptr::read_volatile;

use megadrive_sys::vdp::{VDP, Sprite, TileFlags};
use megadrive_graphics::Renderer;
use megadrive_graphics::default_ascii::DEFAULT_FONT_1X1;

static mut NEW_FRAME: u16 = 0;

#[panic_handler]
#[no_mangle]
fn panic(_info: &PanicInfo) -> ! {
    // Since we don't know where the panic occurred, we can't assume the vdp and renderer are
    // initialized yet
    let mut renderer = Renderer::new();
    let mut vdp = VDP::new();

    vdp.enable_interrupts(false, true, false);
    vdp.enable_display(true);

    // Initialize the default font tiles
    DEFAULT_FONT_1X1.load(&mut vdp);

    let resolution = vdp.resolution();
    let half_screen_width = (resolution.0 >> 1) as i16;
    let half_screen_height = (resolution.1 >> 1) as i16;

    let x_off = 64 + half_screen_width;
    let y_off = 128 + half_screen_height;

    loop {
        renderer.clear();

        // This, unfortunately does not compile due to an undefined __mulsi3 in ld.ldd
        // triggered by the downcast_ref::<&str>()
        // let mut panic_text: &str = &_info.payload().downcast_ref::<&str>().unwrap();
        let panic_text= "Panic!";

        DEFAULT_FONT_1X1.blit_text(&mut renderer, panic_text, x_off as u16, y_off as u16);
        renderer.render(&mut vdp);
        // vsync
        wait_for_vblank();
    }
}

extern "C" {
    fn wait_for_interrupt();
}

fn wait_for_vblank() {
    unsafe {
        while read_volatile(&NEW_FRAME) == 0 {
            wait_for_interrupt();
        }
        NEW_FRAME = 0;
    }
}
