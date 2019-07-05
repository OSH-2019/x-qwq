#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(non_upper_case_globals)]
#![allow(unused_attributes)]

use crate::cspace::*;
use crate::errors::*;
use crate::failures::*;
use crate::invocation::*;
use crate::model::statedata::*;
use crate::object::arch_structures::*;
use crate::object::cap::*;
use crate::object::objecttype::*;
use crate::registerset::*;
use crate::structures::*;
use crate::syscall::*;
use crate::thread::*;
use crate::types::*;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct slot_range_t {
    pub cnode: *mut cte_t,
    pub offset: u64,
    pub length: u64,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct finaliseSlot_ret_t {
    pub status: u64,
    pub success: bool_t,
    pub cleanupInfo: cap_t,
}

extern "C" {
    static mut current_syscall_error: syscall_error_t;
    static mut current_lookup_fault: lookup_fault_t;
    static mut ksCurThread: *mut tcb_t;
    fn preemptionPoint() -> u64;
    fn finaliseCap(cap: cap_t, final_: bool_t, exposed: bool_t) -> finaliseCap_ret_t;
    fn sameRegionAs(cap_a: cap_t, cap_b: cap_t) -> bool_t;
    fn sameObjectAs(cap_a: cap_t, cap_b: cap_t) -> bool_t;
    fn cancelBadgedSends(epptr: *mut endpoint_t, badge: u64);
    fn maskCapRights(cap_rights: seL4_CapRights_t, cap: cap_t) -> cap_t;
    fn kprintf(format: *const u8, ...) -> u64;
    fn puts(str: *const u8) -> u64;
}

//#[no_mangle]
//pub unsafe extern "C" fn decodeCNodeInvocation(
//    invLabel: u64,
//    length: u64,
//    cap: cap_t,
//    excaps: extra_caps_t,
//    buffer: *mut u64,
//) -> u64 {
//    if invLabel < invocation_label::CNodeRevoke as u64
//        || invLabel > invocation_label::CNodeSaveCaller as u64
//    {
//        userError!("CNodeCap: IllegalOperation attemped.");
//        current_syscall_error.type_ = seL4_Error::seL4_IllegalOperation as u64;
//        return exception::EXCEPTION_SYSCALL_ERROR as u64;
//    }
//    if length < 2u64 {
//        userError!("CNode operation: Truncated message.");
//        current_syscall_error.type_ = seL4_Error::seL4_TruncatedMessage as u64;
//        return exception::EXCEPTION_SYSCALL_ERROR as u64;
//    }
//    let index = getSyscallArg(0, buffer);
//    let w_bits = getSyscallArg(1, buffer);
//    let mut lu_ret = lookupTargetSlot(cap, index, w_bits);
//    if lu_ret.status != 0u64 {
//        userError!("CNode operation: Target slot invalid.");
//        return lu_ret.status;
//    }
//    let destSlot = lu_ret.slot;
//    if invLabel >= invocation_label::CNodeCopy as u64
//        && invLabel <= invocation_label::CNodeMutate as u64
//    {
//        if length < 4 || excaps.excaprefs[0] as u64 == 0u64 {
//            userError!("CNode Copy/Mint/Move/Mutate: Truncated message.");
//            current_syscall_error.type_ = seL4_Error::seL4_TruncatedMessage as u64;
//            return exception::EXCEPTION_SYSCALL_ERROR as u64;
//        }
//        let srcIndex = getSyscallArg(2, buffer);
//        let srcDepth = getSyscallArg(3, buffer);
//        let srcRoot = (*excaps.excaprefs[0]).cap;
//        let status = ensureEmptySlot(destSlot);
//        let newCap: cap_t;
//        let isMove: bool;
//        if status != 0u64 {
//            userError!("CNode Copy/Mint/Move/Mutate: Destination not empty.");
//            return status;
//        }
//        lu_ret = lookupSourceSlot(srcRoot, srcIndex, srcDepth);
//        if lu_ret.status != 0u64 {
//            userError!("CNode Copy/Mint/Move/Mutate: Invalid source slot.");
//            return lu_ret.status;
//        }
//        let srcSlot = lu_ret.slot;
//        if cap_get_capType((*srcSlot).cap) == cap_tag_t::cap_null_cap as u64 {
//            userError!("CNode Copy/Mint/Move/Mutate: Source slot invalid or empty.");
//            current_syscall_error.type_ = seL4_Error::seL4_FailedLookup as u64;
//            current_syscall_error.failedLookupWasSource = 1u64;
//            current_lookup_fault = lookup_fault_missing_capability_new(srcDepth);
//            return exception::EXCEPTION_SYSCALL_ERROR as u64;
//        }
//        if invLabel == invocation_label::CNodeCopy as u64 {
//            if length < 5 {
//                userError!("Truncated message for CNode Copy operation.");
//                current_syscall_error.type_ = seL4_Error::seL4_TruncatedMessage as u64;
//                return exception::EXCEPTION_SYSCALL_ERROR as u64;
//            }
//            let cap_rights = rightsFromWord(getSyscallArg(4, buffer));
//            let srcCap = maskCapRights(cap_rights, (*srcSlot).cap);
//            let dc_ret = deriveCap(srcSlot, srcCap);
//            if dc_ret.status != 0u64 {
//                userError!("Error deriving cap for CNode Copy operation.");
//                return dc_ret.status;
//            }
//            newCap = dc_ret.cap;
//            isMove = false;
//        } else if invLabel == invocation_label::CNodeMint as u64 {
//            if length < 6 {
//                userError!("CNode Mint: Truncated message.");
//                current_syscall_error.type_ = seL4_Error::seL4_TruncatedMessage as u64;
//                return exception::EXCEPTION_SYSCALL_ERROR as u64;
//            }
//            let cap_rights = rightsFromWord(getSyscallArg(4, buffer));
//            let capData = getSyscallArg(5, buffer);
//            let srcCap = maskCapRights(cap_rights, (*srcSlot).cap);
//            let dc_ret = deriveCap(srcSlot, updateCapData(0u64, capData, srcCap));
//            if dc_ret.status != 0u64 {
//                userError!("Erro deriving cap for CNode Mint operation.");
//                return dc_ret.status;
//            }
//            newCap = dc_ret.cap;
//            isMove = false;
//        } else if invLabel == invocation_label::CNodeMove as u64 {
//            newCap = (*srcSlot).cap;
//            isMove = true;
//        } else if invLabel == invocation_label::CNodeMutate as u64 {
//            if length < 5 {
//                userError!("CNode Mutate: Truncated message.");
//                current_syscall_error.type_ = seL4_Error::seL4_TruncatedMessage as u64;
//                return exception::EXCEPTION_SYSCALL_ERROR as u64;
//            }
//            let capData = getSyscallArg(4, buffer);
//            newCap = updateCapData(1u64, capData, (*srcSlot).cap);
//            isMove = true;
//        } else {
//            panic!();
//        }
//        if cap_get_capType(newCap) == cap_tag_t::cap_null_cap as u64 {
//            userError!("CNode Copy/Mint/Move/Mutate: Mutated cap would be invalid.");
//            current_syscall_error.type_ = seL4_Error::seL4_IllegalOperation as u64;
//            return exception::EXCEPTION_SYSCALL_ERROR as u64;
//        }
//
//        setThreadState(
//            node_state!(ksCurThread),
//            _thread_state::ThreadState_Restart as u64,
//        );
//        if isMove {
//            invokeCNodeMove(newCap, srcSlot, destSlot);
//        } else {
//            invokeCNodeInsert(newCap, srcSlot, destSlot);
//        }
//    }
//    if invLabel == invocation_label::CNodeRevoke as u64 {
//        setThreadState(
//            node_state!(ksCurThread),
//            _thread_state::ThreadState_Restart as u64,
//        );
//        return invokeCNodeRevoke(destSlot);
//    } else if invLabel == invocation_label::CNodeDelete as u64 {
//        setThreadState(
//            node_state!(ksCurThread),
//            _thread_state::ThreadState_Restart as u64,
//        );
//        return invokeCNodeDelete(destSlot);
//    } else if invLabel == invocation_label::CNodeSaveCaller as u64 {
//        let status = ensureEmptySlot(destSlot);
//        if status != 0u64 {
//            userError!("CNode SaveCaller: Destination slot not empty.");
//            return status;
//        }
//        setThreadState(
//            node_state!(ksCurThread),
//            _thread_state::ThreadState_Restart as u64,
//        );
//        return invokeCNodeSaveCaller(destSlot);
//    } else if invLabel == invocation_label::CNodeCancelBadgedSends as u64 {
//        let destCap = (*destSlot).cap;
//        if hasCancelSendRights(destCap) == 0u64 {
//            userError!("CNode CancelBadgedSends: Target cap invalid.");
//            current_syscall_error.type_ = seL4_Error::seL4_IllegalOperation as u64;
//            return exception::EXCEPTION_SYSCALL_ERROR as u64;
//        }
//        setThreadState(
//            node_state!(ksCurThread),
//            _thread_state::ThreadState_Restart as u64,
//        );
//        return invokeCNodeCancelBadgedSends(destCap);
//    } else if invLabel == invocation_label::CNodeRotate as u64 {
//        if length < 8 || excaps.excaprefs[0] as u64 == 0u64 || excaps.excaprefs[1] as u64 == 0u64 {
//            current_syscall_error.type_ = seL4_Error::seL4_TruncatedMessage as u64;
//            return exception::EXCEPTION_SYSCALL_ERROR as u64;
//        }
//        let pivotNewData = getSyscallArg(2, buffer);
//        let pivotIndex = getSyscallArg(3, buffer);
//        let pivotDepth = getSyscallArg(4, buffer);
//        let srcNewData = getSyscallArg(5, buffer);
//        let srcIndex = getSyscallArg(6, buffer);
//        let srcDepth = getSyscallArg(7, buffer);
//        let pivotRoot = (*excaps.excaprefs[0]).cap;
//        let srcRoot = (*excaps.excaprefs[1]).cap;
//        let mut lu_ret = lookupSourceSlot(srcRoot, srcIndex, srcDepth);
//        if lu_ret.status != 0u64 {
//            return lu_ret.status;
//        }
//        let srcSlot = lu_ret.slot;
//        lu_ret = lookupPivotSlot(pivotRoot, pivotIndex, pivotDepth);
//        if lu_ret.status != 0u64 {
//            return lu_ret.status;
//        }
//        let pivotSlot = lu_ret.slot;
//        if pivotSlot == srcSlot || pivotSlot == destSlot {
//            userError!("CNode Rotate: Pivot slot the same as source or dest slot");
//            current_syscall_error.type_ = seL4_Error::seL4_IllegalOperation as u64;
//            return exception::EXCEPTION_SYSCALL_ERROR as u64;
//        }
//        if srcSlot != destSlot {
//            let status = ensureEmptySlot(destSlot);
//            if status != 0u64 {
//                return status;
//            }
//        }
//        if cap_get_capType((*srcSlot).cap) == cap_tag_t::cap_null_cap as u64 {
//            current_syscall_error.type_ = seL4_Error::seL4_FailedLookup as u64;
//            current_syscall_error.failedLookupWasSource = 1;
//            current_lookup_fault = lookup_fault_missing_capability_new(srcDepth);
//            return exception::EXCEPTION_SYSCALL_ERROR as u64;
//        }
//        if cap_get_capType((*pivotSlot).cap) == cap_tag_t::cap_null_cap as u64 {
//            current_syscall_error.type_ = seL4_Error::seL4_FailedLookup as u64;
//            current_syscall_error.failedLookupWasSource = 1;
//            current_lookup_fault = lookup_fault_missing_capability_new(pivotDepth);
//            return exception::EXCEPTION_SYSCALL_ERROR as u64;
//        }
//        let newSrcCap = updateCapData(1u64, srcNewData, (*srcSlot).cap);
//        let newPivotCap = updateCapData(1u64, pivotNewData, (*pivotSlot).cap);
//        if cap_get_capType(newSrcCap) == cap_tag_t::cap_null_cap as u64 {
//            userError!("CNode Rotate: Source cap invalid.");
//            current_syscall_error.type_ = seL4_Error::seL4_IllegalOperation as u64;
//            return exception::EXCEPTION_SYSCALL_ERROR as u64;
//        }
//        if cap_get_capType(newPivotCap) == cap_tag_t::cap_null_cap as u64 {
//            userError!("CNode Rotate: Pivot cap invalid.");
//            current_syscall_error.type_ = seL4_Error::seL4_IllegalOperation as u64;
//            return exception::EXCEPTION_SYSCALL_ERROR as u64;
//        }
//        setThreadState(
//            node_state!(ksCurThread),
//            _thread_state::ThreadState_Restart as u64,
//        );
//        return invokeCNodeRotate(newSrcCap, newPivotCap, srcSlot, pivotSlot, destSlot);
//    }
//    0u64
//}

#[no_mangle]
pub unsafe extern "C" fn invokeCNodeRevoke(destSlot: *mut cte_t) -> u64 {
    cteRevoke(destSlot)
}

#[no_mangle]
pub unsafe extern "C" fn invokeCNodeDelete(destSlot: *mut cte_t) -> u64 {
    cteDelete(destSlot, 1u64)
}

#[no_mangle]
pub unsafe extern "C" fn invokeCNodeCancelBadgedSends(cap: cap_t) -> u64 {
    let badge = cap_endpoint_cap_get_capEPBadge(cap);
    if badge != 0u64 {
        let ep = cap_endpoint_cap_get_capEPPtr(cap) as *mut endpoint_t;
        cancelBadgedSends(ep, badge);
    }
    0u64
}

#[no_mangle]
pub unsafe extern "C" fn invokeCNodeInsert(
    cap: cap_t,
    srcSlot: *mut cte_t,
    destSlot: *mut cte_t,
) -> u64 {
    cteInsert(cap, srcSlot, destSlot);
    0u64
}

#[no_mangle]
pub unsafe extern "C" fn invokeCNodeMove(
    cap: cap_t,
    srcSlot: *mut cte_t,
    destSlot: *mut cte_t,
) -> u64 {
    cteMove(cap, srcSlot, destSlot);
    0u64
}

#[no_mangle]
pub unsafe extern "C" fn invokeCNodeRotate(
    cap1: cap_t,
    cap2: cap_t,
    slot1: *mut cte_t,
    slot2: *mut cte_t,
    slot3: *mut cte_t,
) -> u64 {
    if slot1 == slot3 {
        cteSwap(cap1, slot1, cap2, slot2);
    } else {
        cteMove(cap2, slot2, slot3);
        cteMove(cap1, slot1, slot2);
    }
    0u64
}

#[no_mangle]
pub unsafe extern "C" fn invokeCNodeSaveCaller(destSlot: *mut cte_t) -> u64 {
    let srcSlot = tcb_ptr_cte_ptr(node_state!(ksCurThread), tcb_cnode_index::tcbCaller as u64);
    let cap = (*srcSlot).cap;
    let cap_type = cap_get_capType(cap);
    if cap_type == cap_tag_t::cap_null_cap as u64 {
        //userError!("CNode SaveCaller: Reply cap not present.")
    } else if cap_type == cap_tag_t::cap_reply_cap as u64 {
        if cap_reply_cap_get_capReplyMaster(cap) == 0u64 {
            cteMove(cap, srcSlot, destSlot);
        }
    } else {
        panic!("caller capability must be null or reply");
    }
    0u64
}

unsafe fn setUntypedCapAsFull(srcCap: cap_t, newCap: cap_t, srcSlot: *mut cte_t) {
    if cap_get_capType(srcCap) == cap_tag_t::cap_untyped_cap as u64
        && cap_get_capType(newCap) == cap_tag_t::cap_untyped_cap as u64
    {
        if cap_untyped_cap_get_capPtr(srcCap) == cap_untyped_cap_get_capPtr(newCap)
            && cap_untyped_cap_get_capBlockSize(newCap) == cap_untyped_cap_get_capBlockSize(srcCap)
        {
            cap_untyped_cap_ptr_set_capFreeIndex(
                &mut (*srcSlot).cap,
                (1 << cap_untyped_cap_get_capBlockSize(srcCap)) - 4,
            );
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn cteInsert(newCap: cap_t, srcSlot: *mut cte_t, destSlot: *mut cte_t) {
    let srcMDB: mdb_node_t = (*srcSlot).cteMDBNode;
    let srcCap: cap_t = (*srcSlot).cap;
    let newCapIsRevocable: u64 = isCapRevocable(newCap, srcCap);
    let mut newMDB = mdb_node_set_mdbPrev(srcMDB, srcSlot as u64);
    newMDB = mdb_node_set_mdbRevocable(newMDB, newCapIsRevocable);
    newMDB = mdb_node_set_mdbFirstBadged(newMDB, newCapIsRevocable);
    setUntypedCapAsFull(srcCap, newCap, srcSlot);
    (*destSlot).cap = newCap;
    (*destSlot).cteMDBNode = newMDB;
    mdb_node_ptr_set_mdbNext(&mut (*srcSlot).cteMDBNode, destSlot as u64);
    if mdb_node_get_mdbNext(newMDB) != 0u64 {
        mdb_node_ptr_set_mdbPrev(
            &mut (*(mdb_node_get_mdbNext(newMDB) as *mut cte_t)).cteMDBNode,
            destSlot as u64,
        );
    }
}

#[no_mangle]
pub unsafe extern "C" fn cteMove(newCap: cap_t, srcSlot: *mut cte_t, destSlot: *mut cte_t) {
    let mdb: mdb_node_t = (*srcSlot).cteMDBNode;
    (*destSlot).cap = newCap;
    (*srcSlot).cap = cap_null_cap_new();
    (*destSlot).cteMDBNode = mdb;
    (*srcSlot).cteMDBNode = mdb_node_new(0, 0, 0, 0);
    let prev_ptr: u64 = mdb_node_get_mdbPrev(mdb);
    if prev_ptr != 0u64 {
        mdb_node_ptr_set_mdbNext(&mut (*(prev_ptr as *mut cte_t)).cteMDBNode, destSlot as u64);
    }
    let next_ptr: u64 = mdb_node_get_mdbNext(mdb);
    if next_ptr != 0u64 {
        mdb_node_ptr_set_mdbPrev(&mut (*(next_ptr as *mut cte_t)).cteMDBNode, destSlot as u64);
    }
}

#[no_mangle]
pub unsafe extern "C" fn capSwapForDelete(slot1: *mut cte_t, slot2: *mut cte_t) {
    if slot1 == slot2 {
        return;
    }
    let cap1 = (*slot1).cap;
    let cap2 = (*slot2).cap;
    cteSwap(cap1, slot1, cap2, slot2);
}

#[no_mangle]
pub unsafe extern "C" fn cteSwap(cap1: cap_t, slot1: *mut cte_t, cap2: cap_t, slot2: *mut cte_t) {
    (*slot1).cap = cap2;
    (*slot2).cap = cap1;
    let mdb1: mdb_node_t = (*slot1).cteMDBNode;
    let mut prev_ptr: u64 = mdb_node_get_mdbPrev(mdb1);
    if prev_ptr != 0u64 {
        mdb_node_ptr_set_mdbNext(&mut (*(prev_ptr as *mut cte_t)).cteMDBNode, slot2 as u64);
    }
    let mut next_ptr: u64 = mdb_node_get_mdbNext(mdb1);
    if next_ptr != 0u64 {
        mdb_node_ptr_set_mdbPrev(&mut (*(next_ptr as *mut cte_t)).cteMDBNode, slot2 as u64);
    }
    let mdb2: mdb_node_t = (*slot2).cteMDBNode;
    (*slot1).cteMDBNode = mdb2;
    (*slot2).cteMDBNode = mdb1;
    prev_ptr = mdb_node_get_mdbPrev(mdb2);
    if prev_ptr != 0u64 {
        mdb_node_ptr_set_mdbNext(&mut (*(prev_ptr as *mut cte_t)).cteMDBNode, slot1 as u64);
    }
    next_ptr = mdb_node_get_mdbNext(mdb2);
    if next_ptr != 0u64 {
        mdb_node_ptr_set_mdbPrev(&mut (*(next_ptr as *mut cte_t)).cteMDBNode, slot1 as u64);
    }
}

#[no_mangle]
pub unsafe extern "C" fn cteRevoke(slot: *mut cte_t) -> u64 {
    let mut nextPtr: *mut cte_t = mdb_node_get_mdbNext((*slot).cteMDBNode) as *mut cte_t;
    while nextPtr as u64 != 0u64 && isMDBParentOf(slot, nextPtr) != 0u64 {
        let mut status: u64 = cteDelete(nextPtr, true as u64);
        if status != 0u64 {
            return status;
        }
        status = preemptionPoint();
        if status != 0u64 {
            return status;
        }
        nextPtr = mdb_node_get_mdbNext((*slot).cteMDBNode) as *mut cte_t;
    }
    0u64
}

#[no_mangle]
pub unsafe extern "C" fn cteDelete(slot: *mut cte_t, exposed: bool_t) -> u64 {
    let fs_ret: finaliseSlot_ret_t = finaliseSlot(slot, exposed);
    if fs_ret.status != 0u64 {
        return fs_ret.status;
    }
    if exposed != 0u64 || fs_ret.success != 0u64 {
        emptySlot(slot, fs_ret.cleanupInfo);
    }
    0u64
}

#[no_mangle]
pub unsafe extern "C" fn emptySlot(slot: *mut cte_t, cleanupInfo: cap_t) {
    if cap_get_capType((*slot).cap) != cap_tag_t::cap_null_cap as u64 {
        let mdbNode: mdb_node_t = (*slot).cteMDBNode;
        let prev = mdb_node_get_mdbPrev(mdbNode) as *mut cte_t;
        let next = mdb_node_get_mdbNext(mdbNode) as *mut cte_t;
        if prev as u64 != 0u64 {
            mdb_node_ptr_set_mdbNext(&mut (*prev).cteMDBNode, next as u64);
        }
        if next as u64 != 0u64 {
            mdb_node_ptr_set_mdbPrev(&mut (*next).cteMDBNode, prev as u64);
        }
        if next as u64 != 0u64 {
            mdb_node_ptr_set_mdbFirstBadged(
                &mut (*next).cteMDBNode,
                mdb_node_get_mdbFirstBadged((*next).cteMDBNode)
                    | mdb_node_get_mdbFirstBadged(mdbNode),
            );
        }
        (*slot).cap = cap_null_cap_new();
        (*slot).cteMDBNode = mdb_node_new(0, 0, 0, 0);
        postCapDeletion(cleanupInfo);
    }
}

#[inline]
unsafe fn capRemovable(cap: cap_t, slot: *mut cte_t) -> bool {
    let cap_type = cap_get_capType(cap);
    if cap_type == cap_tag_t::cap_null_cap as u64 {
        return true;
    } else if cap_type == cap_tag_t::cap_zombie_cap as u64 {
        let n = cap_zombie_cap_get_capZombieNumber(cap);
        let z_slot = cap_zombie_cap_get_capZombiePtr(cap) as *mut cte_t;
        return n == 0 || (n == 1 && slot == z_slot);
    }
    panic!("finaliseCap should only return Zombie or NullCap")
}

#[inline]
unsafe fn capCyclicZombie(cap: cap_t, slot: *mut cte_t) -> bool {
    cap_get_capType(cap) == cap_tag_t::cap_zombie_cap as u64
        && cap_zombie_cap_get_capZombiePtr(cap) as *mut cte_t == slot
}

unsafe fn finaliseSlot(slot: *mut cte_t, immediate: bool_t) -> finaliseSlot_ret_t {
    while cap_get_capType((*slot).cap) != cap_tag_t::cap_null_cap as u64 {
        let final_: u64 = isFinalCapability(slot);
        let fc_ret = finaliseCap((*slot).cap, final_, 0u64);
        if capRemovable(fc_ret.remainder, slot) {
            return finaliseSlot_ret_t {
                status: 0u64,
                success: 1u64,
                cleanupInfo: fc_ret.cleanupInfo,
            };
        }
        (*slot).cap = fc_ret.remainder;
        if immediate == 0u64 && capCyclicZombie(fc_ret.remainder, slot) {
            return finaliseSlot_ret_t {
                status: 0u64,
                success: 0u64,
                cleanupInfo: fc_ret.cleanupInfo,
            };
        }
        let mut status = reduceZombie(slot, immediate);
        if status != 0u64 {
            return finaliseSlot_ret_t {
                status: status,
                success: 0u64,
                cleanupInfo: cap_null_cap_new(),
            };
        }
        status = preemptionPoint();
        if status != 0u64 {
            return finaliseSlot_ret_t {
                status: status,
                success: 0u64,
                cleanupInfo: cap_null_cap_new(),
            };
        }
    }
    finaliseSlot_ret_t {
        status: 0u64,
        success: 1u64,
        cleanupInfo: cap_null_cap_new(),
    }
}

unsafe fn reduceZombie(slot: *mut cte_t, immediate: bool_t) -> u64 {
    let ptr = cap_zombie_cap_get_capZombiePtr((*slot).cap) as *mut cte_t;
    let n = cap_zombie_cap_get_capZombieNumber((*slot).cap);
    let type_ = cap_zombie_cap_get_capZombieType((*slot).cap);
    if immediate == 1u64 {
        let endSlot = ptr.offset((n - 1) as isize);
        let status = cteDelete(endSlot, 0u64);
        if status != 0u64 {
            return status;
        }
        let cap_type = cap_get_capType((*slot).cap);
        if cap_type == cap_tag_t::cap_null_cap as u64 {
        } else if cap_type == cap_tag_t::cap_zombie_cap as u64 {
            let ptr2 = cap_zombie_cap_get_capZombiePtr((*slot).cap) as *mut cte_t;
            if ptr == ptr2
                && cap_zombie_cap_get_capZombieNumber((*slot).cap) == n
                && cap_zombie_cap_get_capZombieType((*slot).cap) == type_
            {
                (*slot).cap = cap_zombie_cap_set_capZombieNumber((*slot).cap, n - 1);
            }
        } else {
            panic!("Expected recursion to result in Zombie.");
        }
    } else {
        capSwapForDelete(ptr, slot);
    }
    0u64
}

#[allow(unused_variables)]
#[no_mangle]
pub unsafe extern "C" fn cteDeleteOne(slot: *mut cte_t) {
    let cap_type = cap_get_capType((*slot).cap);
    if cap_type != cap_tag_t::cap_null_cap as u64 {
        let final_ = isFinalCapability(slot);
        let fc_ret = finaliseCap((*slot).cap, final_, 1u64);
        emptySlot(slot, cap_null_cap_new());
    }
}

#[no_mangle]
pub unsafe extern "C" fn insertNewCap(parent: *mut cte_t, slot: *mut cte_t, cap: cap_t) {
    let next = mdb_node_get_mdbNext((*parent).cteMDBNode) as *mut cte_t;
    (*slot).cap = cap;
    (*slot).cteMDBNode = mdb_node_new(next as u64, 1u64, 1u64, parent as u64);
    if next as u64 != 0u64 {
        mdb_node_ptr_set_mdbPrev(&mut (*next).cteMDBNode, slot as u64);
    }
    mdb_node_ptr_set_mdbNext(&mut (*parent).cteMDBNode, slot as u64);
}

#[no_mangle]
pub unsafe extern "C" fn setupReplyMaster(thread: *mut tcb_t) {
    let slot = tcb_ptr_cte_ptr(thread, tcb_cnode_index::tcbReply as u64);
    if cap_get_capType((*slot).cap) == cap_tag_t::cap_null_cap as u64 {
        (*slot).cap = cap_reply_cap_new(1u64, thread as u64);
        (*slot).cteMDBNode = mdb_node_new(0, 0, 0, 0);
        mdb_node_ptr_set_mdbRevocable(&mut (*slot).cteMDBNode, 1u64);
        mdb_node_ptr_set_mdbFirstBadged(&mut (*slot).cteMDBNode, 1u64);
    }
}

#[no_mangle]
pub unsafe extern "C" fn isMDBParentOf(cte_a: *mut cte_t, cte_b: *mut cte_t) -> bool_t {
    if mdb_node_get_mdbRevocable((*cte_a).cteMDBNode) == 0u64 {
        return 0u64;
    }
    if sameRegionAs((*cte_a).cap, (*cte_b).cap) == 0u64 {
        return 0u64;
    }
    let cap_type = cap_get_capType((*cte_a).cap);
    if cap_type == cap_tag_t::cap_endpoint_cap as u64 {
        let badge = cap_endpoint_cap_get_capEPBadge((*cte_a).cap);
        if badge == 0u64 {
            return 1u64;
        }
        return ((badge == cap_endpoint_cap_get_capEPBadge((*cte_b).cap))
            && mdb_node_get_mdbFirstBadged((*cte_b).cteMDBNode) == 0u64) as u64;
    } else if cap_type == cap_tag_t::cap_notification_cap as u64 {
        let badge = cap_notification_cap_get_capNtfnBadge((*cte_a).cap);
        if badge == 0u64 {
            return 1u64;
        }
        return ((badge == cap_notification_cap_get_capNtfnBadge((*cte_b).cap))
            && mdb_node_get_mdbFirstBadged((*cte_b).cteMDBNode) == 0u64) as u64;
    }
    1u64
}

#[no_mangle]
pub unsafe extern "C" fn ensureNoChildren(slot: *mut cte_t) -> u64 {
    if mdb_node_get_mdbNext((*slot).cteMDBNode) != 0u64 {
        let next = mdb_node_get_mdbNext((*slot).cteMDBNode) as *mut cte_t;
        if isMDBParentOf(slot, next) != 0u64 {
            current_syscall_error.type_ = seL4_Error::seL4_RevokeFirst as u64;
            return exception::EXCEPTION_SYSCALL_ERROR as u64;
        }
    }
    return 0u64;
}

#[no_mangle]
pub unsafe extern "C" fn ensureEmptySlot(slot: *mut cte_t) -> u64 {
    if cap_get_capType((*slot).cap) != cap_tag_t::cap_null_cap as u64 {
        current_syscall_error.type_ = seL4_Error::seL4_DeleteFirst as u64;
        return exception::EXCEPTION_SYSCALL_ERROR as u64;
    }
    return 0u64;
}

#[no_mangle]
pub unsafe extern "C" fn isFinalCapability(cte: *mut cte_t) -> bool_t {
    let mdb = (*cte).cteMDBNode;
    let prevIsSameObject: bool = if mdb_node_get_mdbPrev(mdb) == 0u64 {
        false
    } else {
        let prev = mdb_node_get_mdbPrev(mdb) as *mut cte_t;
        sameObjectAs((*prev).cap, (*cte).cap) == 1u64
    };
    if prevIsSameObject {
        return 0u64;
    } else {
        if mdb_node_get_mdbNext(mdb) == 0u64 {
            return 1u64;
        } else {
            let next = mdb_node_get_mdbNext(mdb) as *mut cte_t;
            return sameObjectAs((*cte).cap, (*next).cap);
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn slotCapLongRunningDelete(slot: *mut cte_t) -> bool_t {
    let cap_type = cap_get_capType((*slot).cap);
    if cap_type == cap_tag_t::cap_null_cap as u64 || isFinalCapability(slot) == 0u64 {
        return 0u64;
    }
    if cap_type == cap_tag_t::cap_thread_cap as u64
        || cap_type == cap_tag_t::cap_zombie_cap as u64
        || cap_type == cap_tag_t::cap_cnode_cap as u64
    {
        return 1u64;
    }
    0u64
}

#[no_mangle]
pub unsafe extern "C" fn getReceiveSlots(thread: *mut tcb_t, buffer: *mut u64) -> *mut cte_t {
    if buffer as u64 == 0u64 {
        return 0u64 as *mut cte_t;
    }
    let ct = loadCapTransfer(buffer);
    let cptr = ct.ctReceiveRoot;
    let luc_ret = lookupCap(thread, cptr);
    if luc_ret.status != 0u64 {
        return 0u64 as *mut cte_t;
    }
    let cnode = luc_ret.cap;
    let lus_ret = lookupTargetSlot(cnode, ct.ctReceiveIndex, ct.ctReceiveDepth);
    if lus_ret.status != 0u64 {
        return 0u64 as *mut cte_t;
    }
    let slot = lus_ret.slot;
    if cap_get_capType((*slot).cap) != cap_tag_t::cap_null_cap as u64 {
        return 0u64 as *mut cte_t;
    }
    slot
}

#[no_mangle]
pub unsafe extern "C" fn loadCapTransfer(buffer: *mut u64) -> cap_transfer_t {
    const offset: isize = (seL4_MsgMaxLength + seL4_MsgMaxExtraCaps as u64 + 2) as isize;
    capTransferFromWords(buffer.offset(offset))
}
