#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(dead_code)]
#![allow(non_upper_case_globals)]
#![allow(unused_attributes)]
#![allow(unused_imports)]
#![allow(unused_mut)]
#![allow(unreachable_code)]

use crate::errors::*;
use crate::failures::*;
use crate::object::arch_structures::*;
use crate::structures::*;
use crate::types::*;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct lookupCap_ret_t {
    pub status: u64,
    pub cap: cap_t,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct lookupCapAndSlot_ret_t {
    pub status: u64,
    pub cap: cap_t,
    pub slot: *mut cte_t,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct lookupSlot_raw_ret_t {
    pub status: u64,
    pub slot: *mut cte_t,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct lookupSlot_ret_t {
    pub status: u64,
    pub slot: *mut cte_t,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct resolveAddressBits_ret_t {
    pub status: u64,
    pub slot: *mut cte_t,
    pub bitsRemaining: u64,
}

extern "C" {
    static mut current_syscall_error: syscall_error_t;
    static mut current_lookup_fault: lookup_fault_t;
}

#[no_mangle]
pub unsafe extern "C" fn lookupCap(thread: *mut tcb_t, cPtr: u64) -> lookupCap_ret_t {
    let mut lu_ret = lookupSlot(thread, cPtr);
    if lu_ret.status != 0u64 {
        return lookupCap_ret_t {
            status: lu_ret.status,
            cap: cap_null_cap_new(),
        };
    }
    lookupCap_ret_t {
        status: 0u64,
        cap: (*lu_ret.slot).cap,
    }
}

#[no_mangle]
pub unsafe extern "C" fn lookupCapAndSlot(thread: *mut tcb_t, cPtr: u64) -> lookupCapAndSlot_ret_t {
    let lu_ret = lookupSlot(thread, cPtr);
    if lu_ret.status != 0u64 {
        return lookupCapAndSlot_ret_t {
            status: lu_ret.status,
            cap: cap_null_cap_new(),
            slot: 0u64 as *mut cte_t,
        };
    }
    lookupCapAndSlot_ret_t {
        status: 0u64,
        cap: (*lu_ret.slot).cap,
        slot: lu_ret.slot,
    }
}

#[no_mangle]
pub unsafe extern "C" fn lookupSlot(thread: *mut tcb_t, capptr: u64) -> lookupSlot_raw_ret_t {
    let threadRoot = (*tcb_ptr_cte_ptr(thread, tcb_cnode_index::tcbCTable as u64)).cap;
    let res_ret = resolveAddressBits(threadRoot, capptr, wordBits);
    lookupSlot_raw_ret_t {
        status: res_ret.status,
        slot: res_ret.slot,
    }
}

#[no_mangle]
pub unsafe extern "C" fn lookupSlotForCNodeOp(
    isSource: bool_t,
    root: cap_t,
    capptr: u64,
    depth: u64,
) -> lookupSlot_ret_t {
    if cap_get_capType(root) != cap_tag_t::cap_cnode_cap as u64 {
        current_syscall_error.type_ = seL4_Error::seL4_FailedLookup as u64;
        current_syscall_error.failedLookupWasSource = isSource;
        current_lookup_fault = lookup_fault_invalid_root_new();
        return lookupSlot_ret_t {
            status: exception::EXCEPTION_SYSCALL_ERROR as u64,
            slot: 0u64 as *mut cte_t,
        };
    }

    if depth < 1 || depth > wordBits {
        current_syscall_error.type_ = seL4_Error::seL4_RangeError as u64;
        current_syscall_error.rangeErrorMin = 1;
        current_syscall_error.rangeErrorMax = wordBits;
        return lookupSlot_ret_t {
            status: exception::EXCEPTION_SYSCALL_ERROR as u64,
            slot: 0u64 as *mut cte_t,
        };
    }

    let res_ret = resolveAddressBits(root, capptr, depth);
    if res_ret.status != 0u64 {
        current_syscall_error.type_ = seL4_Error::seL4_FailedLookup as u64;
        current_syscall_error.failedLookupWasSource = isSource;
        return lookupSlot_ret_t {
            status: exception::EXCEPTION_SYSCALL_ERROR as u64,
            slot: 0u64 as *mut cte_t,
        };
    }
    if res_ret.bitsRemaining != 0 {
        current_syscall_error.type_ = seL4_Error::seL4_FailedLookup as u64;
        current_syscall_error.failedLookupWasSource = isSource;
        current_lookup_fault = lookup_fault_depth_mismatch_new(0, res_ret.bitsRemaining);
        return lookupSlot_ret_t {
            status: exception::EXCEPTION_SYSCALL_ERROR as u64,
            slot: 0u64 as *mut cte_t,
        };
    }
    lookupSlot_ret_t {
        status: 0u64,
        slot: res_ret.slot,
    }
}

#[no_mangle]
pub unsafe extern "C" fn lookupSourceSlot(
    root: cap_t,
    capptr: u64,
    depth: u64,
) -> lookupSlot_ret_t {
    lookupSlotForCNodeOp(1u64, root, capptr, depth)
}

#[no_mangle]
pub unsafe extern "C" fn lookupTargetSlot(
    root: cap_t,
    capptr: u64,
    depth: u64,
) -> lookupSlot_ret_t {
    lookupSlotForCNodeOp(0u64, root, capptr, depth)
}

#[no_mangle]
pub unsafe extern "C" fn lookupPivotSlot(root: cap_t, capptr: u64, depth: u64) -> lookupSlot_ret_t {
    lookupSlotForCNodeOp(1u64, root, capptr, depth)
}

macro_rules! MASK {
    ($x:expr) => {
        (1u64 << ($x)) - 1u64
    };
}

#[no_mangle]
pub unsafe extern "C" fn resolveAddressBits(
    mut nodeCap: cap_t,
    capptr: u64,
    mut n_bits: u64,
) -> resolveAddressBits_ret_t {
    let mut ret = resolveAddressBits_ret_t {
        status: 0u64,
        slot: 0u64 as *mut cte_t,
        bitsRemaining: n_bits,
    };
    if cap_get_capType(nodeCap) != cap_tag_t::cap_cnode_cap as u64 {
        current_lookup_fault = lookup_fault_invalid_root_new();
        ret.status = exception::EXCEPTION_LOOKUP_FAULT as u64;
        return ret;
    }

    loop {
        let radixBits = cap_cnode_cap_get_capCNodeRadix(nodeCap);
        let guardBits = cap_cnode_cap_get_capCNodeGuardSize(nodeCap);
        let levelBits = radixBits + guardBits;
        let capGuard = cap_cnode_cap_get_capCNodeGuard(nodeCap);
        let guard: u64 = (capptr >> ((n_bits - guardBits) & MASK!(wordRadix))) & MASK!(guardBits);
        if guardBits > n_bits || guard != capGuard {
            current_lookup_fault = lookup_fault_guard_mismatch_new(capGuard, n_bits, guardBits);
            ret.status = exception::EXCEPTION_LOOKUP_FAULT as u64;
            return ret;
        }
        if levelBits > n_bits {
            current_lookup_fault = lookup_fault_depth_mismatch_new(levelBits, n_bits);
            ret.status = exception::EXCEPTION_LOOKUP_FAULT as u64;
            return ret;
        }
        let offset: u64 = (capptr >> (n_bits - levelBits)) & MASK!(radixBits);
        let slot = (cap_cnode_cap_get_capCNodePtr(nodeCap) as *mut cte_t).offset(offset as isize);
        if n_bits <= levelBits {
            ret.status = 0u64;
            ret.slot = slot;
            ret.bitsRemaining = 0;
            return ret;
        }
        n_bits -= levelBits;
        nodeCap = (*slot).cap;
        if cap_get_capType(nodeCap) != cap_tag_t::cap_cnode_cap as u64 {
            ret.status = exception::EXCEPTION_NONE as u64;
            ret.slot = slot;
            ret.bitsRemaining = n_bits;
            return ret;
        }
    }
    ret.status = 0u64;
    ret
}
