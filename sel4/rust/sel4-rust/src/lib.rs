#![no_std]
//#![feature(custom_attribute)]

use core::panic::PanicInfo;

#[macro_use]
pub mod model;

#[macro_use]
pub mod errors;

pub mod object;
pub mod types;
pub mod structures;
pub mod thread;
pub mod cspace;
pub mod registerset;
pub mod failures;

#[allow(unused_attributes)]
#[no_mangle] pub unsafe extern "C" fn rust_add(a: i32, b: i32) -> i32 {
    a + b
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}