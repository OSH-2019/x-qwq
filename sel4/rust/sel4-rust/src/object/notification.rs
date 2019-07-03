#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(non_upper_case_globals)]

use crate::types::*;
use crate::structures::*;
use crate::failures::*;
use crate::errors::*;
use crate::thread::*;
use crate::object::arch_structures::*;
use crate::object::objecttype::*;
use crate::object::cap::*;
use crate::registerset::*;
use crate::model::statedata::*;

extern "C" {
    fn tcbSchedEnqueue(tcb:*mut tcb_t);
    fn cancelIPC(tptr: *mut tcb_t);
    fn possibleSwitchTo(tptr: *mut tcb_t);
    fn tcbEPDequeue(tcb: *mut tcb_t, queue: tcb_queue_t) -> tcb_queue_t;
    fn tcbEPAppend(tcb: *mut tcb_t, queue: tcb_queue_t) -> tcb_queue_t;
}

#[inline]
pub unsafe fn ntfn_ptr_get_queue(ntfnPtr: *mut notification_t) -> tcb_queue_t {
    tcb_queue_t {
        head: notification_ptr_get_ntfnQueue_head(ntfnPtr) as *mut tcb_t,
        end: notification_ptr_get_ntfnQueue_tail(ntfnPtr) as *mut tcb_t,
    }
}

#[inline]
pub unsafe fn ntfn_ptr_set_queue(ntfnPtr: *mut notification_t, ntfn_queue: tcb_queue_t) {
    notification_ptr_set_ntfnQueue_head(ntfnPtr, ntfn_queue.head as u64);
    notification_ptr_set_ntfnQueue_tail(ntfnPtr, ntfn_queue.end as u64);
}

#[inline]
pub unsafe fn ntfn_set_active(ntfnPtr: *mut notification_t, badge: u64) {
    notification_ptr_set_state(ntfnPtr, notification_state::NtfnState_Active as u64);
    notification_ptr_set_ntfnMsgIdentifier(ntfnPtr, badge);
}

const badgeRegister: u32 = 0;

#[no_mangle]
pub unsafe extern "C" fn sendSignal(ntfnPtr: *mut notification_t, badge: u64) {
    let state = notification_ptr_get_state(ntfnPtr);
    if state == notification_state::NtfnState_Idle as u64 {
        let tcb = notification_ptr_get_ntfnBoundTCB(ntfnPtr) as *mut tcb_t;
        if tcb as u64 != 0u64 {
            if thread_state_ptr_get_tsType(&mut (*tcb).tcbState as *mut thread_state_t) == _thread_state::ThreadState_BlockedOnReceive as u64 {
                cancelIPC(tcb);
                setThreadState(tcb, _thread_state::ThreadState_Running as u64);
                setRegister(tcb, badgeRegister, badge);
                possibleSwitchTo(tcb);
            } else {
                ntfn_set_active(ntfnPtr, badge);
            }
        } else {
            ntfn_set_active(ntfnPtr, badge);
        }
    } else if state == notification_state::NtfnState_Waiting as u64 {
        let mut ntfn_queue = ntfn_ptr_get_queue(ntfnPtr);
        let dest = ntfn_queue.head;
        ntfn_queue = tcbEPDequeue(dest, ntfn_queue);
        ntfn_ptr_set_queue(ntfnPtr, ntfn_queue);
        if ntfn_queue.head as u64 == 0u64 {
            notification_ptr_set_state(ntfnPtr, notification_state::NtfnState_Idle as u64);
        }
        setThreadState(dest, _thread_state::ThreadState_Running as u64);
        setRegister(dest, badgeRegister, badge);
        possibleSwitchTo(dest);
    } else if state == notification_state::NtfnState_Active as u64 {
        let mut badge2 = notification_ptr_get_ntfnMsgIdentifier(ntfnPtr);
        badge2 |= badge;
        notification_ptr_set_ntfnMsgIdentifier(ntfnPtr, badge2);
    }
}

#[no_mangle]
pub unsafe extern "C" fn receiveSignal(thread: *mut tcb_t, cap: cap_t, isBlocking: bool_t) {
    let ntfnPtr = cap_notification_cap_get_capNtfnPtr(cap) as *mut notification_t;
    let state = notification_ptr_get_state(ntfnPtr);
    if state == notification_state::NtfnState_Idle as u64 ||
        state == notification_state::NtfnState_Waiting as u64 {
        if isBlocking == 1u64 {
            thread_state_ptr_set_tsType(&mut (*thread).tcbState,
                _thread_state::ThreadState_BlockedOnNotification as u64);
            thread_state_ptr_set_blockingObject(&mut (*thread).tcbState as *mut thread_state_t,
                ntfnPtr as u64);
            scheduleTCB(thread);
            let mut ntfn_queue = ntfn_ptr_get_queue(ntfnPtr);
            ntfn_queue = tcbEPAppend(thread, ntfn_queue);
            notification_ptr_set_state(ntfnPtr, notification_state::NtfnState_Waiting as u64);
            ntfn_ptr_set_queue(ntfnPtr, ntfn_queue);
        } else {
            doNBRecvFailedTransfer(thread);
        }
    } else if state == notification_state::NtfnState_Active as u64 {
        setRegister(thread, badgeRegister, notification_ptr_get_ntfnMsgIdentifier(ntfnPtr));
        notification_ptr_set_state(ntfnPtr, notification_state::NtfnState_Idle as u64);
    }
}

#[no_mangle]
pub unsafe extern "C" fn cancelAllSignals(ntfnPtr: *mut notification_t) {
    if notification_ptr_get_state(ntfnPtr) == notification_state::NtfnState_Waiting as u64 {
        let mut thread = notification_ptr_get_ntfnQueue_head(ntfnPtr) as *mut tcb_t;
        notification_ptr_set_state(ntfnPtr, notification_state::NtfnState_Idle as u64);
        notification_ptr_set_ntfnQueue_head(ntfnPtr, 0u64);
        notification_ptr_set_ntfnQueue_tail(ntfnPtr, 0u64);
        while thread as u64 != 0u64 {
            setThreadState(thread, _thread_state::ThreadState_Restart as u64);
            tcbSchedEnqueue(thread);
            thread = (*thread).tcbEPNext;
        }
        rescheduleRequired();
    }
}

#[no_mangle]
pub unsafe extern "C" fn cancelSignal(threadPtr: *mut tcb_t, ntfnPtr: *mut notification_t) {
    let mut ntfn_queue = ntfn_ptr_get_queue(ntfnPtr);
    ntfn_queue = tcbEPDequeue(threadPtr, ntfn_queue);
    ntfn_ptr_set_queue(ntfnPtr, ntfn_queue);
    if ntfn_queue.head as u64 == 0u64 {
        notification_ptr_set_state(ntfnPtr, notification_state::NtfnState_Idle as u64);
    }
    setThreadState(threadPtr, _thread_state::ThreadState_Inactive as u64);
}

#[no_mangle]
pub unsafe extern "C" fn completeSignal(ntfnPtr: *mut notification_t, tcb: *mut tcb_t) {
    if tcb as u64 != 0u64 && notification_ptr_get_state(ntfnPtr) == notification_state::NtfnState_Active as u64 {
        let badge = notification_ptr_get_ntfnMsgIdentifier(ntfnPtr);
        setRegister(tcb, badgeRegister, badge);
        notification_ptr_set_state(ntfnPtr, notification_state::NtfnState_Idle as u64);
    } else {
        panic!("tried to complete signal with inactive notification object");
    }
}

#[inline]
unsafe fn doUnbindNotification(ntfnPtr: *mut notification_t, tcbptr: *mut tcb_t) {
    notification_ptr_set_ntfnBoundTCB(ntfnPtr, 0u64);
    (*tcbptr).tcbBoundNotification = 0u64 as *mut notification_t;
}

#[no_mangle]
pub unsafe extern "C" fn unbindMaybeNotification(ntfnPtr: *mut notification_t) {
    let boundTCB = notification_ptr_get_ntfnBoundTCB(ntfnPtr) as *mut tcb_t;
    if boundTCB as u64 != 0u64 {
        doUnbindNotification(ntfnPtr, boundTCB);
    }
}

#[no_mangle]
pub unsafe extern "C" fn unbindNotification(tcb: *mut tcb_t) {
    let ntfnPtr = (*tcb).tcbBoundNotification;
    if ntfnPtr as u64 != 0u64 {
        doUnbindNotification(ntfnPtr, tcb);
    }
}

#[no_mangle]
pub unsafe extern "C" fn bindNotification(tcb: *mut tcb_t, ntfnPtr: *mut notification_t) {
    notification_ptr_set_ntfnBoundTCB(ntfnPtr, tcb as u64);
    (*tcb).tcbBoundNotification = ntfnPtr;
}