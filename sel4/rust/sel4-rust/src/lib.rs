#![no_std]

use core::panic::PanicInfo;
pub mod object;
pub mod model;

pub mod types;
pub mod structures;

#[allow(unused_attributes)]
#[no_mangle] pub unsafe extern "C" fn rust_add(a: i32, b: i32) -> i32 {
    a + b
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}