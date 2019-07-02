#![allow(non_upper_case_globals)]
#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use crate::structures::*;
use crate::object::arch_structures::*;

extern "C" {
    fn deletedIRQHandler(irq: u8);
    fn Arch_postCapDeletion(cap: cap_t);
}

#[inline]
pub unsafe fn postCapDeletion(cap: cap_t) {
    if cap_get_capType(cap) == cap_tag_t::cap_irq_handler_cap as u64 {
        let irq: u8 = cap_irq_handler_cap_get_capIRQ(cap) as u8;
        deletedIRQHandler(irq);
    } else if isArchCap(cap) != 0u64 {
        Arch_postCapDeletion(cap);
    }
}