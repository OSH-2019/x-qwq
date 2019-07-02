#![allow(non_snake_case)]
#![allow(unused_imports)]

use crate::structures::*;
use crate::object::arch_structures::*;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct deriveCap_ret_t {
    pub status: u64,
    pub cap: cap_t,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct finaliseCap_ret_t {
    pub remainder: cap_t,
    pub cleanupInfo: cap_t,
}