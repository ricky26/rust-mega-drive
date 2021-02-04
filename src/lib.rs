#![crate_type="staticlib"]
#![no_std]

#[no_mangle]
pub fn run_game() -> ! {
    loop {
        unsafe {
            *(0x123456 as *mut i32) = 1234;
        }
    }
}
