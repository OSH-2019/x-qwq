#![allow(non_snake_case)]

#[repr(C)]
#[derive(Copy, Clone)]
pub struct cap_t {
    pub words: [u64; 2],
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct mdb_node_t {
    pub words: [u64; 2],
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct cte_t {
    pub cap: cap_t,
    pub cteMDBNode: mdb_node_t,
}

pub fn cap_untyped_cap_get_capPtr(cap: cap_t) -> u64 {
    let mut ret = cap.words[0] & 0xffffffffffffu64;
    if ret & (1u64 << 47) != 0 {
        ret |= 0xffff000000000000u64;
    }
    ret
}

pub fn cap_untyped_cap_set_capFreeIndex(mut cap: cap_t, v64: u64) -> cap_t {
    cap.words[1] &= !0xffffffffffff0000u64;
    cap.words[1] |= (v64 << 16) & 0xffffffffffff0000u64;
    cap
}

pub fn cap_untyped_cap_get_capFreeIndex(cap: cap_t) -> u64 {
    (cap.words[1] & 0xffffffffffff0000u64) >> 16
}