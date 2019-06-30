#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
use crate::structures::tcb_t;
use crate::types::word_t;

//include/arch/x86/arch/64/mode/machine/registerset.h
pub type register_t=u32;

//include/machine/registerset.h
pub unsafe fn setRegister(thread:*mut tcb_t,reg:register_t,w:word_t){
    (*thread).tcbArch.tcbContext.registers[reg as usize]=w;
}
pub unsafe fn getRegister(thread:*mut tcb_t,reg:register_t)->word_t{
    (*thread).tcbArch.tcbContext.registers[reg as usize]
}