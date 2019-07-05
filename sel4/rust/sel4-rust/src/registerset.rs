#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
use crate::structures::tcb_t;
use crate::types::word_t;

//include/arch/x86/arch/64/mode/machine/registerset.h
pub type register_t = u32;

//include/machine/registerset.h
pub unsafe fn setRegister(thread: *mut tcb_t, reg: register_t, w: word_t) {
    (*thread).tcbArch.tcbContext.registers[reg as usize] = w;
}
pub unsafe fn getRegister(thread: *mut tcb_t, reg: register_t) -> word_t {
    (*thread).tcbArch.tcbContext.registers[reg as usize]
}

pub const seL4_FastMessageRegisters: usize = 4;
pub const n_msgRegisters: usize = 4;
pub const n_frameRegisters: usize = 18;
pub const n_gpRegisters: usize = 1;
pub const n_exceptionMessage: usize = 3;
pub const n_syscallMessage: usize = 18;

pub const RDI: u32 = 0;
pub const capRegister: u32 = 0;
pub const badgeRegister: u32 = 0;
pub const RSI: u32 = 0;
pub const msgInfoRegister: u32 = 1;
pub const RAX: u32 = 2;
pub const RBX: u32 = 3;
pub const RBP: u32 = 4;
pub const R12: u32 = 5;
pub const R13: u32 = 6;
pub const R14: u32 = 7;
pub const RDX: u32 = 8;
pub const R10: u32 = 9;
pub const R8: u32 = 10;
pub const R9: u32 = 11;
pub const R15: u32 = 12;
pub const FLAGS: u32 = 13;
pub const NextIP: u32 = 14;
pub const Error: u32 = 15;
pub const RSP: u32 = 16;
pub const TLS_BASE: u32 = 17;
pub const FaultIP: u32 = 18;
pub const R11: u32 = 19;
pub const RCX: u32 = 20;
pub const CS: u32 = 21;
pub const SS: u32 = 22;
pub const n_contextRegisters: u32 = 23;

pub const msgRegisters: [u32; 4] = [R10, R8, R9, R15];
pub const frameRegisters: [u32; 18] = [
    FaultIP, RSP, FLAGS, RAX, RBX, RCX, RDX, RSI, RDI, RBP, R8, R9, R10, R11, R12, R13, R14, R15,
];
pub const gpRegisters: [u32; 1] = [TLS_BASE];
