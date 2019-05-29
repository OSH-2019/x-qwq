#![allow(non_snake_case)]

use crate::object::cnode::*;
use crate::structures::cte_t;
use crate::object::arch_structures::*;

extern "C" {
    fn getObjectSize(t: u64, userObjSize: u64) -> u64;
    fn createNewObjects(t: u64, parent: *mut cte_t, slots: slot_range_t,
                        regionBase: u64, userSize: u64, deviceMemory: u64);
    fn memzero(s: u64, n: u64);
    fn preemptionPoint() -> u64;
}

fn get_free_ref(base: u64, freeIndex: u64) -> u64 {
    base + (freeIndex << 4)
}

fn get_free_index(base: u64, free: u64) -> u64 {
    (free - base) >> 4
}

fn free_index_to_offset(freeIndex: u64) -> u64 {
    freeIndex << 4
}

fn get_offset_free_ptr(base: u64, offset: u64) -> u64 {
    base + offset
}

fn offset_to_free_index(offset: u64) -> u64 {
    offset >> 4
}

unsafe fn clearMemory(ptr: u64, bits: u32) {
    memzero(ptr, 1 << bits)
}

unsafe fn resetUntypedCap(srcSlot: *mut cte_t) -> u64 {
    let prev_cap = (*srcSlot).cap;
    let block_size = cap_untyped_cap_get_capBlockSize(prev_cap);
    let regionBase = cap_untyped_cap_get_capPtr(prev_cap);
    let chunk = 8u64;
    let offset = free_index_to_offset(cap_untyped_cap_get_capFreeIndex(prev_cap));
    let deviceMemory = cap_untyped_cap_get_capIsDevice(prev_cap);
    if offset == 0 {
        return 0;
    }
    if deviceMemory == 1 || block_size < chunk {
        if deviceMemory == 0 {
            clearMemory(regionBase, block_size as u32);
        }
        (*srcSlot).cap = cap_untyped_cap_set_capFreeIndex(prev_cap, 0);
    } else {
        let mut x: i64 = (((offset - 1) >> chunk) << chunk) as i64;
        while x != - (1 << chunk) {
            clearMemory(get_offset_free_ptr(regionBase, x as u64), chunk as u32);
            (*srcSlot).cap = cap_untyped_cap_set_capFreeIndex(prev_cap, offset_to_free_index(x as u64));
            let status = preemptionPoint();
            if status != 0 {
                return status;
            }
            x -= 1 << chunk;
        }
    }
    0
}

#[allow(unused_assignments)]
#[allow(unused_attributes)]
#[no_mangle] pub unsafe extern "C" fn
invokeUntyped_Retype(srcSlot: *mut cte_t,
                     reset: u64, retypeBase: u64,
                     newType: u64, userSize: u64,
                     destSlots: slot_range_t, deviceMemory: u64) -> u64 {
    let regionBase = cap_untyped_cap_get_capPtr((*srcSlot).cap);
    let mut freeRef = get_free_ref(regionBase, cap_untyped_cap_get_capFreeIndex((*srcSlot).cap));
    if reset == 1 {
        let status = resetUntypedCap(srcSlot);
        if status != 0 {
            return status;
        }
    }
    let totalObjectSize = destSlots.length << getObjectSize(newType, userSize);
    freeRef = retypeBase + totalObjectSize;
    (*srcSlot).cap = cap_untyped_cap_set_capFreeIndex((*srcSlot).cap, get_free_index(regionBase, freeRef));
    createNewObjects(newType, srcSlot, destSlots, retypeBase, userSize,
                     deviceMemory);
    0
}