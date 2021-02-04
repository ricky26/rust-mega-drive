#![feature(no_core)]
#![feature(lang_items)]
#![crate_type="staticlib"]
#![no_core]

mod lang_items;

#[no_mangle]
pub fn run_game() -> ! {
    loop {
        unsafe {
            *(0x123456 as *mut i32) = 1234;
        }
    }
}
