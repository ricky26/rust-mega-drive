#![crate_type="staticlib"]
#![no_std]

mod vdp;

#[no_mangle]
pub fn run_game() -> ! {
    loop {
        let vdp = vdp::VDP::new();

        unsafe {
            *(0x123456 as *mut i32) = 1234;
        }
    }
}
