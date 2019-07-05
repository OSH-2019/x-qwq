#![no_std]
//#![feature(custom_attribute)]

use core::panic::PanicInfo;

#[macro_use]
pub mod model;

#[macro_use]
pub mod errors;

pub mod cspace;
pub mod failures;
pub mod invocation;
pub mod object;
pub mod registerset;
pub mod structures;
pub mod syscall;
pub mod thread;
pub mod types;

#[allow(unused_attributes)]
#[no_mangle]
pub unsafe extern "C" fn rust_add(a: i32, b: i32) -> i32 {
    a + b
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
