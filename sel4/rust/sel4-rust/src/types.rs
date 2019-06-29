#![allow(non_upper_case_globals)]
#![allow(dead_code)]
#![allow(non_camel_case_types)]
use crate::object::arch_structures::pde_t;
use crate::object::arch_structures::pte_t;
use crate::structures::cte_t;

//include/arch/x86/arch/64/mode/types.h
pub const wordRadix:u64=6;
pub const wordBits:u64=1<<6;

//include/types.h
pub type word_t=u64;
pub type sword_t=i64;
pub type vptr_t=word_t;
pub type paddr_t=word_t;
pub type pptr_t=word_t;
pub type cptr_t=word_t;
pub type dev_id_t=word_t;
pub type cpu_id_t=word_t;
pub type logical_id_t=u32;
pub type node_id_t=word_t;
pub type dom_t=word_t;

//include/api/types.h
pub type prio_t=word_t;

//include/basic_types.h
#[repr(C)]
pub enum _bool{
    r#false=0,
    r#true=1
}
pub type bool_t=word_t;

//include/compound_types.h
#[repr(C)]
pub struct pde_range{
    base:*mut pde_t,
    length:word_t
}
pub type pde_range_t=pde_range;

#[repr(C)]
pub struct pte_range{
    base:*mut pte_t,
    length:word_t
}
pub type pte_range_t=pte_range;
pub type cte_ptr_t=*mut cte_t;

const seL4_MsgExtraCapBits:usize=2;
const seL4_MsgMaxExtraCaps:usize=(1usize<<seL4_MsgExtraCapBits)-1;
#[repr(C)]
pub struct extra_caps{
    excaprefs:[cte_ptr_t;seL4_MsgMaxExtraCaps]
}
pub type extra_caps_t=extra_caps;