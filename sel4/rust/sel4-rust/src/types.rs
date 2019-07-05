#![allow(non_upper_case_globals)]
#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
use crate::object::arch_structures::pde_t;
use crate::object::arch_structures::pte_t;
use crate::structures::cte_t;

//include/arch/x86/arch/64/mode/types.h
pub const wordRadix: u64 = 6;
pub const wordBits: u64 = 1 << 6;

//include/types.h
pub type word_t = u64;
pub type sword_t = i64;
pub type vptr_t = word_t;
pub type paddr_t = word_t;
pub type pptr_t = word_t;
pub type cptr_t = word_t;
pub type dev_id_t = word_t;
pub type cpu_id_t = word_t;
pub type logical_id_t = u32;
pub type node_id_t = word_t;
pub type dom_t = word_t;

//include/api/types.h
pub type prio_t = word_t;

//include/basic_types.h
#[repr(C)]
pub enum _bool {
    r#false = 0,
    r#true = 1,
}
pub type bool_t = word_t;

//include/compound_types.h
#[repr(C)]
pub struct pde_range {
    base: *mut pde_t,
    length: word_t,
}
pub type pde_range_t = pde_range;

#[repr(C)]
pub struct pte_range {
    base: *mut pte_t,
    length: word_t,
}
pub type pte_range_t = pte_range;
pub type cte_ptr_t = *mut cte_t;

const seL4_MsgExtraCapBits: usize = 2;
pub const seL4_MsgMaxExtraCaps: usize = (1usize << seL4_MsgExtraCapBits) - 1;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct extra_caps {
    pub excaprefs: [cte_ptr_t; seL4_MsgMaxExtraCaps],
}
pub type extra_caps_t = extra_caps;

//generated/mode/api/shared_types_gen.h
#[repr(C)]
#[derive(Copy, Clone)]
pub struct seL4_MessageInfo {
    words: [u64; 1],
}
pub type seL4_MessageInfo_t = seL4_MessageInfo;

#[inline]
pub fn seL4_MessageInfo_get_length(seL4_MessageInfo: seL4_MessageInfo_t) -> u64 {
    seL4_MessageInfo.words[0] & 0x7fu64
}
#[inline]
pub fn seL4_MessageInfo_set_length(
    mut seL4_MessageInfo: seL4_MessageInfo_t,
    v64: u64,
) -> seL4_MessageInfo_t {
    seL4_MessageInfo.words[0] &= !0x7fu64;
    seL4_MessageInfo.words[0] |= v64 & 0x7fu64;
    seL4_MessageInfo
}

#[inline]
pub fn seL4_MessageInfo_get_extraCaps(seL4_MessageInfo: seL4_MessageInfo_t) -> u64 {
    (seL4_MessageInfo.words[0] & 0x180u64) >> 7
}

#[inline]
pub fn seL4_MessageInfo_set_extraCaps(
    mut seL4_MessageInfo: seL4_MessageInfo_t,
    v64: u64,
) -> seL4_MessageInfo_t {
    seL4_MessageInfo.words[0] &= !0x180u64;
    seL4_MessageInfo.words[0] |= (v64 << 7) & 0x180u64;
    seL4_MessageInfo
}

#[inline]
pub fn seL4_MessageInfo_get_capsUnwrapped(seL4_MessageInfo: seL4_MessageInfo_t) -> u64 {
    (seL4_MessageInfo.words[0] & 0xe00u64) >> 9
}

#[inline]
pub fn seL4_MessageInfo_set_capsUnwrapped(
    mut seL4_MessageInfo: seL4_MessageInfo_t,
    v64: u64,
) -> seL4_MessageInfo_t {
    seL4_MessageInfo.words[0] &= !0xe00u64;
    seL4_MessageInfo.words[0] |= (v64 << 9) & 0xe00u64;
    seL4_MessageInfo
}

#[inline]
pub fn seL4_MessageInfo_new(
    label: u64,
    capsUnwrapped: u64,
    extraCaps: u64,
    length: u64,
) -> seL4_MessageInfo_t {
    let ret: u64 = 0
        | (label & 0xfffffffffffffu64) << 12
        | (capsUnwrapped & 0x7u64) << 9
        | (extraCaps & 0x3u64) << 7
        | (length & 0x7fu64) << 0;
    seL4_MessageInfo_t { words: [ret] }
}

#[inline]
pub fn seL4_CapRights_get_capAllowGrant(seL4_CapRights: seL4_CapRights_t) -> u64 {
    (seL4_CapRights.words[0] & 0x4u64) >> 2
}

#[inline]
pub fn seL4_CapRights_get_capAllowRead(seL4_CapRights: seL4_CapRights_t) -> u64 {
    (seL4_CapRights.words[0] & 0x2u64) >> 1
}

#[inline]
pub fn seL4_CapRights_get_capAllowWrite(seL4_CapRights: seL4_CapRights_t) -> u64 {
    seL4_CapRights.words[0] & 0x1u64
}

//include/api/types.h
pub const seL4_MsgMaxLength: u64 = 120;
#[inline]
pub fn messageInfoFromWord(w: word_t) -> seL4_MessageInfo_t {
    let mut mi: seL4_MessageInfo_t = seL4_MessageInfo_t { words: [w] };
    let len: word_t = seL4_MessageInfo_get_length(mi);
    if len > seL4_MsgMaxLength {
        mi = seL4_MessageInfo_set_length(mi, seL4_MsgMaxLength);
    }
    mi
}

#[inline]
pub fn wordFromMessageInfo(mi: seL4_MessageInfo_t) -> word_t {
    mi.words[0]
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct seL4_CapRights_t {
    words: [u64; 1],
}

#[inline]
pub fn rightsFromWord(w: u64) -> seL4_CapRights_t {
    seL4_CapRights_t { words: [w] }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct cap_transfer_t {
    pub ctReceiveRoot: u64,
    pub ctReceiveIndex: u64,
    pub ctReceiveDepth: u64,
}

#[inline]
pub unsafe fn capTransferFromWords(wptr: *mut u64) -> cap_transfer_t {
    cap_transfer_t {
        ctReceiveRoot: *wptr.offset(0),
        ctReceiveIndex: *wptr.offset(1),
        ctReceiveDepth: *wptr.offset(2),
    }
}
