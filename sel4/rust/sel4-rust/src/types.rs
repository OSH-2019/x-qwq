#![allow(non_upper_case_globals)]
#![allow(dead_code)]
#![allow(non_camel_case_types)]

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
pub enum _bool{
    r#false=0,
    r#true=1
}
pub type bool_t=word_t;