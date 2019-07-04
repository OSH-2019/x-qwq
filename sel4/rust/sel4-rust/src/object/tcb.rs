#![allow(non_upper_case_globals)]
#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(unused_imports)]

use crate::types::*;
use crate::structures::*;
use crate::registerset::*;
use crate::thread::*;
use crate::cspace::*;
use crate::object::cap::*;
use crate::object::cnode::*;
use crate::object::notification::*;
use crate::object::arch_structures::*;

extern "C" {
    static mut ksReadyQueues:[tcb_queue_t;256];
    static mut ksReadyQueuesL1Bitmap:[u64;1];
    static mut ksReadyQueuesL2Bitmap:[[u64;L2_BITMAP_SIZE];1];
    static mut current_extra_caps: extra_caps_t;
    static mut current_fault: seL4_Fault_t;
    //fn addToBitmap(cpu: u64, dom: u64, prio: u64);
    //fn removeFromBitmap(cpu: u64, dom: u64, prio: u64);
    fn kprintf(format: *const u8, ...);
}

macro_rules! MASK {
    ($x:expr) => {
        (1u64<<($x))-1u64
    };
}

#[inline]
pub unsafe fn addToBitmap(cpu: u64, dom: u64, prio: u64) {
    let l1index = prio_to_l1index(prio);
    let l1index_inverted = invert_l1index(l1index);
    //ignore smp
    ksReadyQueuesL1Bitmap[dom as usize] |= 1u64 << l1index;
    ksReadyQueuesL2Bitmap[dom as usize][l1index_inverted as usize] |= 1u64 << (prio & MASK!(wordRadix));
}

#[inline]
pub unsafe fn removeFromBitmap(cpu: u64, dom: u64, prio: u64) {
    let l1index = prio_to_l1index(prio);
    let l1index_inverted = invert_l1index(l1index);
    //ignore smp
    ksReadyQueuesL2Bitmap[dom as usize][l1index_inverted as usize] &= !(1u64 << (prio & MASK!(wordRadix)));
    if ksReadyQueuesL2Bitmap[dom as usize][l1index_inverted as usize] == 0u64 {
        ksReadyQueuesL1Bitmap[dom as usize] &= !(1u64 << l1index);
    }
}

#[no_mangle]
pub unsafe extern "C" fn tcbSchedEnqueue(tcb: *mut tcb_t) {
    if thread_state_get_tcbQueued((*tcb).tcbState) == 0u64 {
        let dom = (*tcb).tcbDomain;
        let prio = (*tcb).tcbPriority;
        let idx = ready_queues_index(dom, prio) as usize;
        let mut queue = ksReadyQueues[idx];
        
        if queue.end as u64 == 0u64 {
            //ignore smp
            queue.end = tcb;
            addToBitmap(0, dom, prio);
        } else {
            (*queue.head).tcbSchedPrev = tcb;
        }
        (*tcb).tcbSchedPrev = 0u64 as *mut tcb_t;
        (*tcb).tcbSchedNext = queue.head;
        queue.head = tcb;
        //ignore smp
        ksReadyQueues[idx] = queue;
        thread_state_ptr_set_tcbQueued(&mut (*tcb).tcbState as *mut thread_state_t, 1u64);
    }
}

#[no_mangle]
pub unsafe extern "C" fn tcbSchedAppend(tcb: *mut tcb_t) {
    if thread_state_get_tcbQueued((*tcb).tcbState) == 0u64 {
        let dom = (*tcb).tcbDomain;
        let prio = (*tcb).tcbPriority;
        let idx = ready_queues_index(dom, prio) as usize;
        let mut queue = ksReadyQueues[idx];
        
        if queue.head as u64 == 0u64 {
            //ignore smp
            queue.head = tcb;
            addToBitmap(0, dom, prio);
        } else {
            (*queue.end).tcbSchedNext = tcb;
        }
        (*tcb).tcbSchedPrev = queue.end;
        (*tcb).tcbSchedNext = 0u64 as *mut tcb_t;
        queue.end = tcb;
        //ignore smp
        ksReadyQueues[idx] = queue;
        thread_state_ptr_set_tcbQueued(&mut (*tcb).tcbState as *mut thread_state_t, 1u64);
    }
}

#[no_mangle]
pub unsafe extern "C" fn tcbSchedDequeue(tcb: *mut tcb_t) {
    if thread_state_get_tcbQueued((*tcb).tcbState) != 0u64 {
        let dom = (*tcb).tcbDomain;
        let prio = (*tcb).tcbPriority;
        let idx = ready_queues_index(dom, prio) as usize;
        let mut queue = ksReadyQueues[idx];
        
        if (*tcb).tcbSchedPrev as u64 != 0u64 {
            (*(*tcb).tcbSchedPrev).tcbSchedNext = (*tcb).tcbSchedNext;
        } else {
            queue.head = (*tcb).tcbSchedNext;
            if (*tcb).tcbSchedNext as u64 == 0u64 {
                //ignore smp
                removeFromBitmap(0, dom, prio);
            }
        }
        
        if (*tcb).tcbSchedNext as u64 != 0u64 {
            (*(*tcb).tcbSchedNext).tcbSchedPrev = (*tcb).tcbSchedPrev;
        } else {
            queue.end = (*tcb).tcbSchedPrev;
        }
        //ignore smp
        ksReadyQueues[idx] = queue;
        thread_state_ptr_set_tcbQueued(&mut (*tcb).tcbState as *mut thread_state_t, 0u64);
    }
}

#[no_mangle]
pub unsafe extern "C" fn tcbEPAppend(tcb: *mut tcb_t, mut queue: tcb_queue_t) -> tcb_queue_t {
    if queue.head as u64 == 0u64 {
        queue.head = tcb;
    } else {
        (*queue.end).tcbEPNext = tcb;
    }
    (*tcb).tcbEPPrev = queue.end;
    (*tcb).tcbEPNext = 0 as *mut tcb_t;
    queue.end = tcb;
    queue
}

#[no_mangle]
pub unsafe extern "C" fn tcbEPDequeue(tcb: *mut tcb_t, mut queue: tcb_queue_t) -> tcb_queue_t {
    if (*tcb).tcbEPPrev as u64 != 0u64 {
        (*(*tcb).tcbEPPrev).tcbEPNext = (*tcb).tcbEPNext;
    } else {
        queue.head = (*tcb).tcbEPNext;
    }
    if (*tcb).tcbEPNext as u64 != 0u64 {
        (*(*tcb).tcbEPNext).tcbEPPrev = (*tcb).tcbEPPrev;
    } else {
        queue.end = (*tcb).tcbEPPrev;
    }
    queue
}

#[no_mangle]
pub unsafe extern "C" fn getExtraCPtr(bufferPtr: *mut u64, i: u64) -> u64 {
    *bufferPtr.offset((seL4_MsgMaxLength + 2 + i) as isize)
}

#[no_mangle]
pub unsafe extern "C" fn setExtraBadge(bufferPtr: *mut u64, badge: u64, i: u64) {
    *bufferPtr.offset((seL4_MsgMaxLength + 2 + i) as isize) = badge;
}

#[no_mangle]
pub unsafe extern "C" fn setupCallerCap(sender: *mut tcb_t, receiver: *mut tcb_t) {
    setThreadState(sender, _thread_state::ThreadState_BlockedOnReply as u64);
    let replySlot = tcb_ptr_cte_ptr(sender, tcb_cnode_index::tcbReply as u64);
    let callerSlot = tcb_ptr_cte_ptr(receiver, tcb_cnode_index::tcbCaller as u64);
    cteInsert(cap_reply_cap_new(0u64, sender as u64), replySlot, callerSlot);
}

#[no_mangle]
pub unsafe extern "C" fn deleteCallerCap(receiver: *mut tcb_t) {
    let callerSlot = tcb_ptr_cte_ptr(receiver, tcb_cnode_index::tcbCaller as u64);
    cteDeleteOne(callerSlot);
}

#[no_mangle]
pub unsafe extern "C" fn lookupExtraCaps(thread: *mut tcb_t, bufferPtr: *mut u64, info: seL4_MessageInfo_t) -> u64 {
    if bufferPtr as u64 == 0u64 {
        current_extra_caps.excaprefs[0] = 0u64 as *mut cte_t;
        return 0u64;
    }
    let length = seL4_MessageInfo_get_extraCaps(info);
    let mut i: usize = 0;
    while i < length as usize {
        let cptr = getExtraCPtr(bufferPtr, i as u64);
        let lu_ret = lookupSlot(thread, cptr);
        if lu_ret.status != 0u64 {
            current_fault = seL4_Fault_CapFault_new(cptr, 0u64);
            return lu_ret.status;
        }
        current_extra_caps.excaprefs[i] = lu_ret.slot;
        i += 1;
    }
    if i < seL4_MsgMaxExtraCaps {
        current_extra_caps.excaprefs[i] = 0u64 as *mut cte_t;
    }
    0u64
}

//#[no_mangle]
//pub unsafe extern "C" fn copyMRs(sender: *mut tcb_t, sendBuf: *mut u64, receiver: *mut tcb_t, recvBuf: *mut u64, n: u64) -> u64 {
//    let mut i: usize = 0;
//    while i < n as usize && i < n_msgRegisters {
//        setRegister(receiver, msgRegisters[i], getRegister(sender, msgRegisters[i]));
//        i += 1;
//    }
//    if recvBuf as u64 == 0u64 || sendBuf as u64 == 0u64 {
//        return i as u64;
//    }
//    while i < n as usize {
//        *recvBuf.offset((i + 1) as isize) = *sendBuf.offset((i + 1) as isize);
//        i += 1;
//    }
//    i as u64
//}