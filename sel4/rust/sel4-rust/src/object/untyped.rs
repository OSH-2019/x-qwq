#![allow(non_snake_case)]

use crate::object::cnode::*;
use crate::structures::*;

fn get_free_ref(base: u64, freeIndex: u64) -> u64 {
    base + (freeIndex << 4)
}

fn get_free_index(base: u64, free: u64) -> u64 {
    (free - base) >> 4
}

extern "C" {
    fn resetUntypedCap(srcSlot: *mut cte_t) -> u64;
    fn getObjectSize(t: u64, userObjSize: u64) -> u64;
    fn createNewObjects(t: u64, parent: *mut cte_t, slots: slot_range_t,
                        regionBase: u64, userSize: u64, deviceMemory: u64);
}

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