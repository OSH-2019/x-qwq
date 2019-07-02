#![allow(non_snake_case)]

use crate::types::*;
use crate::structures::*;
use crate::object::arch_structures::*;

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
    fn setUntypedCapAsFull(srcCap: cap_t, newCap: cap_t, srcSlot: *mut cte_t);
    fn preemptionPoint() -> u64;
    fn postCapDeletion(cap: cap_t);
    fn emptySlot(slot: *mut cte_t, cleanupInfo: cap_t);
    fn finaliseSlot(slot: *mut cte_t, immediate: bool_t) -> finaliseSlot_ret_t;
    fn isMDBParentOf(cte_a: *mut cte_t, cte_b: *mut cte_t) -> u64;
    fn kprintf(format: *const u8, ...) -> u64;
    fn puts(str: *const u8) -> u64;
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
    mdb_node_ptr_set_mdbNext(&mut (*srcSlot).cteMDBNode as *mut mdb_node_t, destSlot as u64);
    if mdb_node_get_mdbNext(newMDB) != 0u64 {
        mdb_node_ptr_set_mdbPrev(&mut (*(mdb_node_get_mdbNext(newMDB) as *mut cte_t)).cteMDBNode as *mut mdb_node_t, destSlot as u64);
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
        mdb_node_ptr_set_mdbNext(&mut (*(prev_ptr as *mut cte_t)).cteMDBNode as *mut mdb_node_t, destSlot as u64);
    }
    let next_ptr: u64 = mdb_node_get_mdbNext(mdb);
    if next_ptr != 0u64 {
        mdb_node_ptr_set_mdbPrev(&mut (*(next_ptr as *mut cte_t)).cteMDBNode as *mut mdb_node_t, destSlot as u64);
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
        mdb_node_ptr_set_mdbNext(&mut (*(prev_ptr as *mut cte_t)).cteMDBNode as *mut mdb_node_t, slot2 as u64);
    }
    let mut next_ptr: u64 = mdb_node_get_mdbNext(mdb1);
    if next_ptr != 0u64 {
        mdb_node_ptr_set_mdbPrev(&mut (*(next_ptr as *mut cte_t)).cteMDBNode as *mut mdb_node_t, slot2 as u64);
    }
    let mdb2: mdb_node_t = (*slot2).cteMDBNode;
    (*slot1).cteMDBNode = mdb2;
    (*slot2).cteMDBNode = mdb1;
    prev_ptr = mdb_node_get_mdbPrev(mdb2);
    if prev_ptr != 0u64 {
        mdb_node_ptr_set_mdbNext(&mut (*(prev_ptr as *mut cte_t)).cteMDBNode as *mut mdb_node_t, slot1 as u64);
    }
    next_ptr = mdb_node_get_mdbNext(mdb2);
    if next_ptr != 0u64 {
        mdb_node_ptr_set_mdbPrev(&mut (*(next_ptr as *mut cte_t)).cteMDBNode as *mut mdb_node_t, slot1 as u64);
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

//#[no_mangle]
//pub unsafe extern "C" fn emptySlot(slot: *mut cte_t, cleanupInfo: cap_t) {
//    if cap_get_capType((*slot).cap) != cap_tag_t::cap_null_cap as u64 {
//        let mdbNode: mdb_node_t = (*slot).cteMDBNode;
//        let prev = mdb_node_get_mdbPrev(mdbNode) as *mut cte_t;
//        let next = mdb_node_get_mdbNext(mdbNode) as *mut cte_t;
//        if prev as u64 != 0u64 {
//            mdb_node_ptr_set_mdbNext(&mut (*prev).cteMDBNode as *mut mdb_node_t, next as u64);
//        }
//        if next as u64 != 0u64 {
//            mdb_node_ptr_set_mdbPrev(&mut (*next).cteMDBNode as *mut mdb_node_t, prev as u64);
//        }
//        if next as u64 != 0u64 {
//            mdb_node_ptr_set_mdbFirstBadged(&mut (*next).cteMDBNode as *mut mdb_node_t, 
//                mdb_node_get_mdbFirstBadged((*next).cteMDBNode) | mdb_node_get_mdbFirstBadged(mdbNode));
//        }
//        (*slot).cap = cap_null_cap_new();
//        (*slot).cteMDBNode = mdb_node_new(0, 0, 0, 0);
//        postCapDeletion(cleanupInfo);
//    }
//}