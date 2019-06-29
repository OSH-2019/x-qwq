#![no_std]
#![feature(custom_attribute)]

use core::panic::PanicInfo;
pub mod object;

#[macro_use]
pub mod model;

pub mod types;
pub mod structures;
pub mod thread;

#[allow(unused_attributes)]
#[no_mangle] pub unsafe extern "C" fn rust_add(a: i32, b: i32) -> i32 {
    a + b
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}