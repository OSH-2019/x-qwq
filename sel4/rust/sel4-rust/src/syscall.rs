#![allow(non_upper_case_globals)]
#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(unused_imports)]

use crate::registerset::*;
use crate::structures::*;
use crate::types::*;

extern "C" {
    static mut ksCurThread: *mut tcb_t;
}

#[inline]
pub unsafe fn getSyscallArg(i: u64, ipc_buffer: *mut u64) -> u64 {
    if (i as usize) < n_msgRegisters {
        return getRegister(node_state!(ksCurThread), msgRegisters[i as usize]);
    }
    *ipc_buffer.offset((i + 1) as isize)
}
