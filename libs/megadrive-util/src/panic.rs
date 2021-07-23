use core::panic::PanicInfo;
use core::ptr::read_volatile;

use megadrive_sys::vdp::VDP;
use megadrive_graphics::Renderer;
use megadrive_graphics::default_ascii::DEFAULT_FONT_1X1;

static mut NEW_FRAME: u16 = 0;

extern "C" {
    fn wait_for_interrupt();
}

#[panic_handler]
#[no_mangle]
fn panic(_info: &PanicInfo) -> ! {
    let mut renderer = Renderer::new();
    let mut vdp = VDP::new();
    DEFAULT_FONT_1X1.init_default();

    let resolution = vdp.resolution();
    let half_screen_width = (resolution.0 >> 1) as i16;
    let half_screen_height = (resolution.1 >> 1) as i16;

    let x_off = 128 + half_screen_width;
    let y_off = 128 + half_screen_height;

    vdp.enable_interrupts(false, true, false);
    vdp.enable_display(true);
    let mut frame = 0u16;

    loop {
        renderer.clear();

        DEFAULT_FONT_1X1.blit_text(&mut renderer, "!", x_off as u16, y_off as u16);
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
