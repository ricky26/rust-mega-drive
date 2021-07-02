#![no_std]
#![feature(allocator_api)]
#![feature(const_mut_refs)]
#![feature(alloc_error_handler)]
#![feature(default_alloc_error_handler)]

pub mod heap;
pub mod hole;

use crate::heap::Heap;

#[global_allocator]
static mut ALLOCATOR: Heap = Heap::empty();

#[alloc_error_handler]
fn alloc_error_handler(layout: core::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}

