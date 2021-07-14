#![no_std]
#![feature(const_mut_refs)]
#![feature(alloc_error_handler)]
#![feature(default_alloc_error_handler)]

mod heap;
mod hole;

use crate::heap::Heap;

#[global_allocator]
pub static mut ALLOCATOR: Heap = Heap::empty();

#[alloc_error_handler]
fn alloc_error_handler(layout: core::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}

