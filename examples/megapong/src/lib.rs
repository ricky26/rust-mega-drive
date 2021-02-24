#![no_std]
#![feature(array_chunks)]

use core::panic::PanicInfo;
use megadrive_sys::vdp::{VDP, Sprite, SpriteSize};
use megadrive_sys::fm::{FM, Note, Panning, Channel};
use megadrive_input::{Controllers, Button};
use core::ptr::{read_volatile, write_volatile};
use megadrive_graphics::Renderer;

static mut NEW_FRAME: u16 = 0;

extern "C" {
    fn wait_for_interrupt();
}

fn setup_piano(ch: &Channel) {
    let op0 = ch.operator(0);
    op0.set_multiplier(1, 7);
    op0.set_total_level(35);
    op0.set_attack_rate(63, 1);
    op0.set_decay_rate(5, false);
    op0.set_sustain_rate(2);
    op0.set_release_rate(1, 1);

    let op1 = ch.operator(1);
    op1.set_multiplier(13, 0);
    op1.set_total_level(45);
    op1.set_attack_rate(25, 2);
    op1.set_decay_rate(5, false);
    op1.set_sustain_rate(2);
    op1.set_release_rate(1, 1);

    let op2 = ch.operator(2);
    op2.set_multiplier(3, 3);
    op2.set_total_level(38);
    op2.set_attack_rate(63, 1);
    op2.set_decay_rate(5, false);
    op2.set_sustain_rate(2);
    op2.set_release_rate(1, 1);

    let op3 = ch.operator(3);
    op3.set_multiplier(1, 0);
    op3.set_total_level(0);
    op3.set_attack_rate(20, 2);
    op3.set_decay_rate(7, false);
    op3.set_sustain_rate(2);
    op3.set_release_rate(6, 10);

    ch.set_algorithm(2, 6);
    ch.set_panning(Panning::Both, 0, 0);
    ch.set_frequency(Note::F, 5);
}

fn upload_graphics(vdp: &mut VDP) {
    // Load graphics.
    static TILE_DATA: [u8; 32 * 12] = [
        // H - 8
        0x08, 0x00, 0x08, 0x00,
        0x08, 0x00, 0x08, 0x00,
        0x08, 0x00, 0x08, 0x00,
        0x08, 0x88, 0x88, 0x00,
        0x08, 0x00, 0x08, 0x00,
        0x08, 0x00, 0x08, 0x00,
        0x08, 0x00, 0x08, 0x00,
        0x00, 0x00, 0x00, 0x00,
        // E - 2
        0x08, 0x88, 0x88, 0x00,
        0x08, 0x00, 0x00, 0x00,
        0x08, 0x00, 0x00, 0x00,
        0x08, 0x88, 0x88, 0x00,
        0x08, 0x00, 0x00, 0x00,
        0x08, 0x00, 0x00, 0x00,
        0x08, 0x88, 0x88, 0x00,
        0x00, 0x00, 0x00, 0x00,
        // L - 3
        0x08, 0x00, 0x00, 0x00,
        0x08, 0x00, 0x00, 0x00,
        0x08, 0x00, 0x00, 0x00,
        0x08, 0x00, 0x00, 0x00,
        0x08, 0x00, 0x00, 0x00,
        0x08, 0x00, 0x00, 0x00,
        0x08, 0x88, 0x88, 0x00,
        0x00, 0x00, 0x00, 0x00,
        // O - 4
        0x08, 0x88, 0x88, 0x00,
        0x08, 0x00, 0x08, 0x00,
        0x08, 0x00, 0x08, 0x00,
        0x08, 0x00, 0x08, 0x00,
        0x08, 0x00, 0x08, 0x00,
        0x08, 0x00, 0x08, 0x00,
        0x08, 0x88, 0x88, 0x00,
        0x00, 0x00, 0x00, 0x00,
        // W - 5
        0x08, 0x00, 0x08, 0x00,
        0x08, 0x00, 0x08, 0x00,
        0x08, 0x00, 0x08, 0x00,
        0x08, 0x00, 0x08, 0x00,
        0x08, 0x08, 0x08, 0x00,
        0x08, 0x80, 0x88, 0x00,
        0x08, 0x00, 0x08, 0x00,
        0x00, 0x00, 0x00, 0x00,
        // R - 6
        0x08, 0x80, 0x00, 0x00,
        0x08, 0x08, 0x80, 0x00,
        0x08, 0x00, 0x08, 0x00,
        0x08, 0x00, 0x80, 0x00,
        0x08, 0x88, 0x00, 0x00,
        0x08, 0x08, 0x00, 0x00,
        0x08, 0x00, 0x88, 0x00,
        0x00, 0x00, 0x00, 0x00,
        // D - 7
        0x08, 0x80, 0x00, 0x00,
        0x08, 0x08, 0x80, 0x00,
        0x08, 0x00, 0x08, 0x00,
        0x08, 0x00, 0x08, 0x00,
        0x08, 0x00, 0x08, 0x00,
        0x08, 0x08, 0x80, 0x00,
        0x08, 0x80, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00,
        // ! - 8
        0x08, 0x00, 0x00, 0x00,
        0x08, 0x00, 0x00, 0x00,
        0x08, 0x00, 0x00, 0x00,
        0x08, 0x00, 0x00, 0x00,
        0x08, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00,
        0x08, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00,
        // Underscore - 9
        0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00,
        0x88, 0x88, 0x88, 0x00,
        0x00, 0x00, 0x00, 0x00,
        // Ball - 10
        0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00,
        0x00, 0x11, 0x11, 0x00,
        0x00, 0x11, 0x11, 0x00,
        0x00, 0x11, 0x11, 0x00,
        0x00, 0x11, 0x11, 0x00,
        0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00,
        // Paddle Half - 11
        0x00, 0x00, 0x10, 0x00,
        0x00, 0x01, 0x10, 0x00,
        0x00, 0x01, 0x10, 0x00,
        0x00, 0x01, 0x10, 0x00,
        0x00, 0x01, 0x10, 0x00,
        0x00, 0x01, 0x10, 0x00,
        0x00, 0x01, 0x10, 0x00,
        0x00, 0x01, 0x10, 0x00,
        // Paddle Half - 12
        0x00, 0x01, 0x10, 0x00,
        0x00, 0x01, 0x10, 0x00,
        0x00, 0x01, 0x10, 0x00,
        0x00, 0x01, 0x10, 0x00,
        0x00, 0x01, 0x10, 0x00,
        0x00, 0x01, 0x10, 0x00,
        0x00, 0x01, 0x10, 0x00,
        0x00, 0x00, 0x10, 0x00,
    ];
    vdp.set_tiles(1, TILE_DATA.array_chunks());
}

#[no_mangle]
pub fn main() -> ! {
    let version = megadrive_sys::version();
    let fm = FM::new();
    let mut vdp = VDP::new();
    let mut controllers = Controllers::new();

    upload_graphics(&mut vdp);
    let mut renderer = Renderer::new();

    let paddle_hit = fm.channel(0);
    setup_piano(&paddle_hit);

    let screen_hit = fm.channel(1);
    setup_piano(&screen_hit);
    screen_hit.set_frequency(Note::C, 4);

    let mut bx = 0;
    let mut by = 0;
    let mut dx = 3;
    let mut dy = 3;

    let mut p0y = 0;
    let mut p1y = 0;

    let paddle_speed = 5;
    let half_screen_width = (version.resolution().0 >> 1) as i16;
    let half_screen_height = (version.resolution().1 >> 1) as i16;
    let game_border = 9;
    let half_border_width = half_screen_width - game_border;
    let half_border_height = half_screen_height - game_border;

    let x_off = 128 + half_screen_width;
    let y_off = 128 + half_screen_height;

    let update_player = |controllers: &Controllers, idx: usize, y: &mut i16, by: i16| {
        if let Some(c) = controllers.controller_state(idx) {
            if c.down(Button::Up) {
                *y = (*y - paddle_speed).max(-half_border_height)
            } else if c.down(Button::Down) {
                *y = (*y + paddle_speed).min(half_border_height)
            }
        } else {
            *y = by;
        }
    };

    vdp.enable_interrupts(false, true, false);
    vdp.enable_display(true);

    let mut frame = 0u16;
    loop {
        renderer.clear();
        controllers.update();

        // Update players
        update_player(&controllers, 0, &mut p0y, by);
        update_player(&controllers, 1, &mut p1y, by);

        // Draw Text
        {
            let mut x = x_off - 6 * 7;
            let y = y_off;

            static TILE_INDICES: [u16; 12] = [1, 2, 3, 3, 4, 0, 5, 4, 6, 3, 7, 8];
            let anim_frame = (frame >> 1) & 0x3f;

            for (idx, tile_id) in TILE_INDICES.iter().cloned().enumerate() {
                let my_frame = (anim_frame + (idx as u16)) & 0x3f;
                let mut my_y = y + if my_frame >= 32 {
                    31 - (my_frame as i16)
                } else {
                    (my_frame as i16) - 32
                };

                let buttons = controllers.controller_state(0).map_or(0, |c| c.down_raw());
                let down = ((buttons >> idx) & 1) != 0;
                if down {
                    my_y += 4;
                }

                let tile_id = if tile_id == 0 && down {
                    9
                } else {
                    tile_id
                };
                let mut sprite = Sprite::for_tile(tile_id, SpriteSize::Size1x1);
                sprite.y = my_y as u16;
                sprite.x = x as u16;
                renderer.draw_sprite(sprite);
                x += 7;
            }
        }

        // Draw pong.
        {
            let mut s = Sprite::for_tile(11, SpriteSize::Size1x2);
            s.set_high_priority(true);

            // P1
            s.x = ((x_off - half_screen_width) + game_border - 4) as u16;
            s.y = (y_off + p0y - 8) as u16;
            renderer.draw_sprite(s.clone());

            // P2
            s.set_horizontal_flip(true);
            s.x = ((x_off + half_screen_width) - game_border - 4) as u16;
            s.y = (y_off + p1y - 8) as u16;
            renderer.draw_sprite(s);

            // Ball
            let mut s = Sprite::for_tile(10, SpriteSize::Size1x1);
            s.set_high_priority(true);
            s.x = (x_off + bx - 4) as u16;
            s.y = (y_off + by - 4) as u16;
            renderer.draw_sprite(s);
        }

        // Update pong.
        {
            bx += dx;
            by += dy;

            let mut hit = false;
            let mut paddle = false;

            let half_ball_width = half_border_width - 1;

            if bx < -half_ball_width {
                bx = -half_ball_width - half_ball_width - bx;
                dx = -dx;
                hit = true;
                paddle = (by > (p0y - 8)) && (by < (p0y + 8));
            } else if bx > half_ball_width {
                bx = half_ball_width + half_ball_width - bx;
                dx = -dx;
                hit = true;
                paddle = (by > (p1y - 8)) && (by < (p1y + 8));
            }

            if by < -half_border_height {
                by = -half_border_height - half_border_height - by;
                dy = -dy;
                hit = true;
            } else if by > half_border_height {
                by = half_border_height + half_border_height - by;
                dy = -dy;
                hit = true;
            }

            if hit {
                if paddle {
                    paddle_hit.set_key(true)
                } else {
                    screen_hit.set_key(true);
                }
            } else {
                paddle_hit.set_key(false);
                screen_hit.set_key(false);
            }
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
