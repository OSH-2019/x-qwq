#![allow(non_snake_case)]
#![allow(unused_assignments)]
#![allow(unused_attributes)]
#![allow(unused_variables)]
#![allow(non_upper_case_globals)]
#![allow(unused_imports)]

use crate::cspace::*;
use crate::errors::*;
use crate::failures::*;
use crate::invocation::*;
use crate::object::arch_structures::*;
use crate::object::cnode::*;
use crate::object::objecttype::*;
use crate::structures::*;
use crate::syscall::*;
use crate::thread::*;
use crate::types::*;

extern "C" {
    static mut current_syscall_error: syscall_error_t;
    static mut current_lookup_fault: lookup_fault_t;
    static mut ksCurThread: *mut tcb_t;
    fn createNewObjects(
        t: u64,
        parent: *mut cte_t,
        slots: slot_range_t,
        regionBase: u64,
        userSize: u64,
        deviceMemory: u64,
    );
    fn Arch_isFrameType(type_: u64) -> bool_t;
    fn memzero(s: u64, n: u64);
    fn preemptionPoint() -> u64;
}

#[inline]
fn get_free_ref(base: u64, freeIndex: u64) -> u64 {
    base + (freeIndex << 4)
}

#[inline]
fn get_free_index(base: u64, free: u64) -> u64 {
    (free - base) >> 4
}

#[inline]
fn free_index_to_offset(freeIndex: u64) -> u64 {
    freeIndex << 4
}

#[inline]
fn get_offset_free_ptr(base: u64, offset: u64) -> u64 {
    base + offset
}

#[inline]
fn offset_to_free_index(offset: u64) -> u64 {
    offset >> 4
}

#[inline]
unsafe fn clearMemory(ptr: u64, bits: u32) {
    memzero(ptr, 1 << bits)
}

//CONFIG_RETYPE_FAN_OUT_LIMIT
const CONFIG_RETYPE_FAN_OUT_LIMIT: u64 = 256;

macro_rules! MASK {
    ($x:expr) => {
        (1u64 << ($x)) - 1u64
    };
}

#[inline]
fn alignUp(baseValue: u64, alignment: u64) -> u64 {
    (baseValue + (1u64 << alignment) - 1) & !MASK!(alignment)
}

pub const seL4_MinUntypedBits: u64 = 4;
pub const seL4_MaxUntypedBits: u64 = 47;

#[no_mangle]
pub unsafe extern "C" fn decodeUntypedInvocation(
    invLabel: u64,
    length: u64,
    slot: *mut cte_t,
    cap: cap_t,
    excaps: extra_caps_t,
    call: bool_t,
    buffer: *mut u64,
) -> u64 {
    if invLabel != invocation_label::UntypedRetype as u64 {
        userError!("Untyped cap: Illegal operation attempted.");
        current_syscall_error.type_ = seL4_Error::seL4_IllegalOperation as u64;
        return exception::EXCEPTION_SYSCALL_ERROR as u64;
    }
    if length < 6 || excaps.excaprefs[0] as u64 == 0u64 {
        userError!("Untyped invocation: Truncated messaage.");
        current_syscall_error.type_ = seL4_Error::seL4_TruncatedMessage as u64;
        return exception::EXCEPTION_SYSCALL_ERROR as u64;
    }
    let newType = getSyscallArg(0, buffer);
    let userObjSize = getSyscallArg(1, buffer);
    let nodeIndex = getSyscallArg(2, buffer);
    let nodeDepth = getSyscallArg(3, buffer);
    let nodeOffset = getSyscallArg(4, buffer);
    let nodeWindow = getSyscallArg(5, buffer);
    let rootSlot = excaps.excaprefs[0];
    if newType >= seL4_ArchObjectType::seL4_ObjectTypeCount as u64 {
        userError!("Untyped Retype: Invalid object type.");
        current_syscall_error.type_ = seL4_Error::seL4_InvalidArgument as u64;
        current_syscall_error.invalidArgumentNumber = 0;
        return exception::EXCEPTION_SYSCALL_ERROR as u64;
    }
    let objectSize = getObjectSize(newType, userObjSize);
    if userObjSize >= wordBits || objectSize > seL4_MaxUntypedBits {
        userError!("Untyped Retype: Invalid object size.");
        current_syscall_error.type_ = seL4_Error::seL4_RangeError as u64;
        current_syscall_error.rangeErrorMin = 0;
        current_syscall_error.rangeErrorMax = seL4_MaxUntypedBits;
        return exception::EXCEPTION_SYSCALL_ERROR as u64;
    }
    if newType == seL4_ObjectType::seL4_CapTableObject as u64 && userObjSize == 0 {
        userError!("Untyped Retype: Requested CapTable size too small.");
        current_syscall_error.type_ = seL4_Error::seL4_InvalidArgument as u64;
        current_syscall_error.invalidArgumentNumber = 1;
        return exception::EXCEPTION_SYSCALL_ERROR as u64;
    }
    if newType == seL4_ObjectType::seL4_UntypedObject as u64 && userObjSize < seL4_MinUntypedBits {
        userError!("Untyped Retype: Requested UntypedItem size too small.");
        current_syscall_error.type_ = seL4_Error::seL4_InvalidArgument as u64;
        current_syscall_error.invalidArgumentNumber = 1;
        return exception::EXCEPTION_SYSCALL_ERROR as u64;
    }
    let nodeCap: cap_t;
    if nodeDepth == 0 {
        nodeCap = (*excaps.excaprefs[0]).cap;
    } else {
        let rootCap = (*excaps.excaprefs[0]).cap;
        let lu_ret = lookupTargetSlot(rootCap, nodeIndex, nodeDepth);
        if lu_ret.status != 0u64 {
            userError!("Untyped Retype: Invalid destination address.");
            return lu_ret.status;
        }
        nodeCap = (*lu_ret.slot).cap
    }
    if cap_get_capType(nodeCap) != cap_tag_t::cap_cnode_cap as u64 {
        userError!("Untyped Retype: Destination cap invalid or read-only.");
        current_syscall_error.type_ = seL4_Error::seL4_FailedLookup as u64;
        current_syscall_error.failedLookupWasSource = 0u64;
        current_lookup_fault = lookup_fault_missing_capability_new(nodeDepth);
        return exception::EXCEPTION_SYSCALL_ERROR as u64;
    }
    let nodeSize = 1u64 << cap_cnode_cap_get_capCNodeRadix(nodeCap);
    if nodeOffset > nodeSize - 1 {
        userError!(
            "Untyped Retype: Destination node offset #%d too large.",
            nodeOffset as i32
        );
        current_syscall_error.type_ = seL4_Error::seL4_RangeError as u64;
        current_syscall_error.rangeErrorMin = 0;
        current_syscall_error.rangeErrorMax = nodeSize - 1;
        return exception::EXCEPTION_SYSCALL_ERROR as u64;
    }
    if nodeWindow < 1 || nodeWindow > CONFIG_RETYPE_FAN_OUT_LIMIT {
        userError!(
            "Untyped Retype: Number of requested objects (%d) too small or large.",
            nodeWindow as i32
        );
        current_syscall_error.type_ = seL4_Error::seL4_RangeError as u64;
        current_syscall_error.rangeErrorMin = 1;
        current_syscall_error.rangeErrorMax = CONFIG_RETYPE_FAN_OUT_LIMIT;
        return exception::EXCEPTION_SYSCALL_ERROR as u64;
    }
    if nodeWindow > nodeSize - nodeOffset {
        userError!("Untyped Retype: Requested destination window overruns size of node.");
        current_syscall_error.type_ = seL4_Error::seL4_RangeError as u64;
        current_syscall_error.rangeErrorMin = 1;
        current_syscall_error.rangeErrorMax = nodeSize - nodeOffset;
        return exception::EXCEPTION_SYSCALL_ERROR as u64;
    }
    let slots = slot_range_t {
        cnode: cap_cnode_cap_get_capCNodePtr(nodeCap) as *mut cte_t,
        offset: nodeOffset,
        length: nodeWindow,
    };
    let mut i: u64 = nodeOffset;
    while i < nodeOffset + nodeWindow {
        let status = ensureEmptySlot(slots.cnode.offset(i as isize));
        if status != 0u64 {
            userError!(
                "Untyped Retype: Slot #%d in destination window non-empty",
                i as i32
            );
            return status;
        }
        i += 1;
    }
    let status = ensureNoChildren(slot);
    let freeIndex: u64;
    let reset: bool;
    if status != 0u64 {
        freeIndex = cap_untyped_cap_get_capFreeIndex(cap);
        reset = false;
    } else {
        freeIndex = 0;
        reset = true;
    }
    let freeRef = get_free_ref(cap_untyped_cap_get_capPtr(cap), freeIndex);
    let untypedFreeBytes: u64 =
        (1u64 << cap_untyped_cap_get_capBlockSize(cap)) - free_index_to_offset(freeIndex);
    if (untypedFreeBytes >> objectSize) < nodeWindow {
        //ignore userError
        current_syscall_error.type_ = seL4_Error::seL4_NotEnoughMemory as u64;
        current_syscall_error.memoryLeft = untypedFreeBytes;
        return exception::EXCEPTION_SYSCALL_ERROR as u64;
    }
    let deviceMemory = cap_untyped_cap_get_capIsDevice(cap);
    if deviceMemory != 0u64
        && Arch_isFrameType(newType) == 0u64
        && newType != seL4_ObjectType::seL4_UntypedObject as u64
    {
        userError!("Untyped Retype: Creating kernel objects with device untyped");
        current_syscall_error.type_ = seL4_Error::seL4_InvalidArgument as u64;
        current_syscall_error.invalidArgumentNumber = 1;
        return exception::EXCEPTION_SYSCALL_ERROR as u64;
    }
    let alignedFreeRef = alignUp(freeRef, objectSize);
    setThreadState(
        node_state!(ksCurThread),
        _thread_state::ThreadState_Restart as u64,
    );
    invokeUntyped_Retype(
        slot,
        reset as u64,
        alignedFreeRef,
        newType,
        userObjSize,
        slots,
        deviceMemory,
    )
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
        while x != -(1 << chunk) {
            clearMemory(get_offset_free_ptr(regionBase, x as u64), chunk as u32);
            (*srcSlot).cap =
                cap_untyped_cap_set_capFreeIndex(prev_cap, offset_to_free_index(x as u64));
            let status = preemptionPoint();
            if status != 0 {
                return status;
            }
            x -= 1 << chunk;
        }
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn invokeUntyped_Retype(
    srcSlot: *mut cte_t,
    reset: u64,
    retypeBase: u64,
    newType: u64,
    userSize: u64,
    destSlots: slot_range_t,
    deviceMemory: u64,
) -> u64 {
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
    (*srcSlot).cap =
        cap_untyped_cap_set_capFreeIndex((*srcSlot).cap, get_free_index(regionBase, freeRef));
    createNewObjects(
        newType,
        srcSlot,
        destSlots,
        retypeBase,
        userSize,
        deviceMemory,
    );
    0
}
