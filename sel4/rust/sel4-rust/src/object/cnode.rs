#![allow(non_snake_case)]

use crate::structures::*;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct slot_range_t {
    pub cnode: *mut cte_t,
    pub offset: u64,
    pub length: u64,
}