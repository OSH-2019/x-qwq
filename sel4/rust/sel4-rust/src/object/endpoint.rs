#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(dead_code)]
#![allow(non_upper_case_globals)]
#![allow(unused_attributes)]

use crate::object::arch_structures::*;
use crate::object::tcb::{setupCallerCap, tcbEPAppend, tcbEPDequeue,setMRs_syscall_error,tcbSchedEnqueue};
use crate::structures::*;
use crate::thread::{doIPCTransfer, possibleSwitchTo, scheduleTCB, setThreadState,doNBRecvFailedTransfer,rescheduleRequired};
use crate::types::*;
use crate::object::notification::{cancelSignal,completeSignal};
use crate::registerset::setRegister;
use crate::failures::syscall_error_t;
use crate::object::cnode::cteDeleteOne;

extern "C" {
    //arch/x86/arch/kernel/vspace.c
    fn lookupIPCBuffer(isReceiver:bool_t,thread:*mut tcb_t)->*mut word_t;
    //src/inlines.c
    static mut current_syscall_error:syscall_error_t;
}

#[inline]
pub fn ep_ptr_get_queue(epptr: &endpoint_t) -> tcb_queue_t {
    tcb_queue_t {
        head: endpoint_ptr_get_epQueue_head(epptr) as *mut tcb_t,
        end: endpoint_ptr_get_epQueue_tail(epptr) as *mut tcb_t,
    }
}

#[inline]
pub fn ep_ptr_set_queue(epptr: &mut endpoint_t, queue: tcb_queue_t) {
    endpoint_ptr_set_epQueue_head(epptr, queue.head as word_t);
    endpoint_ptr_set_epQueue_tail(epptr, queue.end as word_t);
}

const seL4_Fault_NullFault: u64 = 0;
#[no_mangle]
pub unsafe extern "C" fn sendIPC(
    blocking: bool_t,
    do_call: bool_t,
    badge: word_t,
    canGrant: bool_t,
    thread: *mut tcb_t,
    epptr: *mut endpoint_t,
) {
    let state = endpoint_ptr_get_state(&*epptr);
    match state {
        state
            if (state == endpoint_state::EPState_Idle as u64)
                || (state == endpoint_state::EPState_Send as u64) =>
        {
            if blocking != 0 {
                let mut queue: tcb_queue_t;
                let tcbstate = &mut (*thread).tcbState;
                thread_state_ptr_set_tsType(
                    tcbstate,
                    _thread_state::ThreadState_BlockedOnSend as u64,
                );
                thread_state_ptr_set_blockingObject(tcbstate, epptr as u64);
                thread_state_ptr_set_blockingIPCBadge(tcbstate, badge);
                thread_state_ptr_set_blockingIPCCanGrant(tcbstate, canGrant);
                thread_state_ptr_set_blockingIPCIsCall(tcbstate, do_call);
                scheduleTCB(thread);
                queue = ep_ptr_get_queue(&*epptr);
                queue = tcbEPAppend(thread, queue);
                endpoint_ptr_set_state(&mut *epptr, endpoint_state::EPState_Send as u64);
                ep_ptr_set_queue(&mut *epptr, queue);
            }
        }
        state if state == endpoint_state::EPState_Recv as u64 => {
            let mut queue: tcb_queue_t = ep_ptr_get_queue(&*epptr);
            let dest: *mut tcb_t = queue.head;
            queue = tcbEPDequeue(dest, queue);
            ep_ptr_set_queue(&mut *epptr, queue);
            if queue.head as u64 == 0 {
                endpoint_ptr_set_state(&mut *epptr, endpoint_state::EPState_Idle as u64);
            }
            doIPCTransfer(thread, epptr, badge, canGrant, dest);
            setThreadState(dest, _thread_state::ThreadState_Running as u64);
            possibleSwitchTo(dest);

            if do_call != 0
                || seL4_Fault_ptr_get_seL4_FaultType(&(*thread).tcbFault) != seL4_Fault_NullFault
            {
                if canGrant != 0 {
                    setupCallerCap(thread, dest);
                } else {
                    setThreadState(thread, _thread_state::ThreadState_Inactive as u64);
                }
            }
        }
        _ => {
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn receiveIPC(
    thread:*mut tcb_t,
    cap:cap_t,
    isBlocking:bool_t
){
    let epptr:*mut endpoint_t=cap_endpoint_cap_get_capEPPtr(cap) as *mut endpoint_t;
    let ntfnPtr:*mut notification_t=(*thread).tcbBoundNotification;
    if (ntfnPtr as u64 !=0) && notification_ptr_get_state(&*ntfnPtr) == notification_state::NtfnState_Active as u64 {
        completeSignal(ntfnPtr,thread);
    }else{
        if (endpoint_ptr_get_state(&*epptr)==endpoint_state::EPState_Idle as u64) || (endpoint_ptr_get_state(&*epptr)==endpoint_state::EPState_Recv as u64) {
            let mut queue:tcb_queue_t;
            if isBlocking as u64 !=0{
                thread_state_ptr_set_tsType(&mut (*thread).tcbState,_thread_state::ThreadState_BlockedOnReceive as u64);
                thread_state_ptr_set_blockingObject(&mut (*thread).tcbState,epptr as u64);
                scheduleTCB(thread);
                queue=ep_ptr_get_queue(&*epptr);
                queue=tcbEPAppend(thread,queue);
                endpoint_ptr_set_state(&mut *epptr,endpoint_state::EPState_Recv as u64);
                ep_ptr_set_queue(&mut *epptr, queue);
            }else{
                doNBRecvFailedTransfer(thread);
            }
        }else{
            let mut queue:tcb_queue_t=ep_ptr_get_queue(&*epptr);
            let sender:*mut tcb_t=queue.head;
            queue=tcbEPDequeue(sender,queue);
            ep_ptr_set_queue(&mut *epptr, queue);
            if queue.head as u64 ==0 {
                endpoint_ptr_set_state(&mut *epptr, endpoint_state::EPState_Idle as u64);
            }
            let badge:word_t=thread_state_ptr_get_blockingIPCBadge(&(*sender).tcbState);
            let canGrant:bool_t=thread_state_ptr_get_blockingIPCCanGrant(&(*sender).tcbState);
            doIPCTransfer(sender,epptr,badge,canGrant,thread);
            let do_call:bool_t=thread_state_ptr_get_blockingIPCIsCall(&(*sender).tcbState);
            if do_call!=0 || seL4_Fault_get_seL4_FaultType(&(*sender).tcbFault) != seL4_Fault_NullFault {
                if canGrant !=0 {
                    setupCallerCap(sender,thread);
                }else{
                    setThreadState(sender,_thread_state::ThreadState_Inactive as u64);
                }
            }else{
                setThreadState(sender,_thread_state::ThreadState_Running as u64);
                possibleSwitchTo(sender);
            }
        }
    }
}

const badgeRegister:u32=0;
const msgInfoRegister:u32=1;
#[no_mangle]
pub unsafe extern "C" fn replyFromKernel_error(thread:*mut tcb_t){
    let ipcBuffer:*mut word_t=lookupIPCBuffer(true as u64,thread);
    setRegister(thread,badgeRegister,0);
    let len:word_t=setMRs_syscall_error(thread,ipcBuffer);
    setRegister(thread, msgInfoRegister,
        wordFromMessageInfo(seL4_MessageInfo_new(current_syscall_error.type_,0,0,len)));
}

#[no_mangle]
pub unsafe extern "C" fn replyFromKernel_success_empty(thread:*mut tcb_t){
    setRegister(thread, badgeRegister, 0);
    setRegister(thread, msgInfoRegister,
        wordFromMessageInfo(seL4_MessageInfo_new(0, 0, 0, 0)));
}

const tcbReply:u64=2;
#[no_mangle]
pub unsafe extern "C" fn cancelIPC(tptr:*mut tcb_t){
    let state:&mut thread_state_t=&mut (*tptr).tcbState;
    let tstype=thread_state_get_tsType(state);
    match tstype {
        tstype if (tstype==_thread_state::ThreadState_BlockedOnSend as u64) || (tstype == _thread_state::ThreadState_BlockedOnReceive as u64) =>{
            let epptr:*mut endpoint_t=thread_state_ptr_get_blockingObject(state) as *mut endpoint_t;
            let mut queue:tcb_queue_t=ep_ptr_get_queue(&*epptr);
            queue=tcbEPDequeue(tptr,queue);
            ep_ptr_set_queue(&mut *epptr, queue);
            if queue.head as u64 ==0 {
                endpoint_ptr_set_state(&mut *epptr, endpoint_state::EPState_Idle as u64);
            }
            setThreadState(tptr,_thread_state::ThreadState_Inactive as u64);
        }
        tstype if tstype==_thread_state::ThreadState_BlockedOnNotification as u64 =>{
            cancelSignal(tptr,thread_state_ptr_get_blockingObject(state) as *mut notification_t);
        }
        tstype if tstype==_thread_state::ThreadState_BlockedOnReply as u64=>{
            (*tptr).tcbFault=seL4_Fault_NullFault_new();
            let slot:*mut cte_t=tcb_ptr_cte_ptr(tptr, tcbReply);
            let callerCap:*mut cte_t=mdb_node_get_mdbNext((*slot).cteMDBNode) as *mut cte_t;
            if callerCap as u64 !=0 {
                cteDeleteOne(callerCap);
            }
        }
        _=>{} //这里不能直接panic！= =
    }
}

#[no_mangle]
pub unsafe extern "C" fn cancelAllIPC(epptr:*mut endpoint_t){
    if endpoint_ptr_get_state(&*epptr) != endpoint_state::EPState_Idle as u64 {
        let mut thread:*mut tcb_t=endpoint_ptr_get_epQueue_head(&*epptr) as *mut tcb_t;
        endpoint_ptr_set_state(&mut *epptr, endpoint_state::EPState_Idle as u64);
        endpoint_ptr_set_epQueue_head(&mut *epptr, 0);
        endpoint_ptr_set_epQueue_tail(&mut *epptr, 0);
        while thread as u64 !=0 {
            setThreadState(thread,_thread_state::ThreadState_Restart as u64);
            tcbSchedEnqueue(thread);
            thread=(*thread).tcbEPNext;
        }
        rescheduleRequired();
    }
}

#[no_mangle]
pub unsafe extern "C" fn cancelBadgedSends(epptr:*mut endpoint_t,badge:word_t){
    let state=endpoint_ptr_get_state(&*epptr);
    match state {
        state if (state==endpoint_state::EPState_Idle as u64) || (state==endpoint_state::EPState_Recv as u64) =>{
        }
        state if state==endpoint_state::EPState_Send as u64 =>{
            let mut queue:tcb_queue_t=ep_ptr_get_queue(&*epptr);
            endpoint_ptr_set_state(&mut *epptr, endpoint_state::EPState_Idle as u64);
            endpoint_ptr_set_epQueue_head(&mut *epptr, 0);
            endpoint_ptr_set_epQueue_tail(&mut *epptr, 0);
            let mut thread:*mut tcb_t=queue.head;
            let mut next:*mut tcb_t;
            while thread as u64 !=0 {
                let b:word_t=thread_state_ptr_get_blockingIPCBadge(&(*thread).tcbState);
                next=(*thread).tcbEPNext;
                if b == badge {
                    setThreadState(thread,_thread_state::ThreadState_Restart as u64);
                    tcbSchedEnqueue(thread);
                    queue=tcbEPDequeue(thread,queue);
                }
                thread=next;
            }
            ep_ptr_set_queue(&mut *epptr, queue);
            if queue.head as u64 !=0 {
                endpoint_ptr_set_state(&mut *epptr, endpoint_state::EPState_Send as u64);
            }
            rescheduleRequired();
        }
        _=>{
            panic!("invalid EP state");
        }
    }
}