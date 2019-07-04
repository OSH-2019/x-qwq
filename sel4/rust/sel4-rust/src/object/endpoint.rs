#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(dead_code)]
#![allow(non_upper_case_globals)]

use crate::structures::*;
use crate::object::arch_structures::*;
use crate::types::*;
use crate::thread::{scheduleTCB,doIPCTransfer,setThreadState,possibleSwitchTo};
use crate::object::tcb::{tcbEPDequeue,tcbEPAppend,setupCallerCap};

#[inline]
pub fn ep_ptr_get_queue(epptr:&endpoint_t)->tcb_queue_t{
    tcb_queue_t{
        head:endpoint_ptr_get_epQueue_head(epptr) as *mut tcb_t,
        end:endpoint_ptr_get_epQueue_tail(epptr) as *mut tcb_t
    }
}

#[inline]
pub fn ep_ptr_set_queue(epptr:&mut endpoint_t,queue:tcb_queue_t){
    endpoint_ptr_set_epQueue_head(epptr,queue.head as word_t);
    endpoint_ptr_set_epQueue_tail(epptr,queue.end as word_t);
}

const seL4_Fault_NullFault:u64=0;
pub unsafe fn sendIPC(blocking:bool_t,do_call:bool_t,badge:word_t,
    canGrant:bool_t,thread:*mut tcb_t,epptr:*mut endpoint_t){
    let state=endpoint_ptr_get_state(&*epptr);
    match state {
        state if (state == endpoint_state::EPState_Idle as u64)
            || (state == endpoint_state::EPState_Send as u64) =>
            if blocking!=0 {
                let mut queue:tcb_queue_t;
                let tcbstate=&mut (*thread).tcbState;
                thread_state_ptr_set_tsType(tcbstate,_thread_state::ThreadState_BlockedOnSend as u64);
                thread_state_ptr_set_blockingObject(tcbstate,epptr as u64);
                thread_state_ptr_set_blockingIPCBadge(tcbstate,badge);
                thread_state_ptr_set_blockingIPCCanGrant(tcbstate,canGrant);
                thread_state_ptr_set_blockingIPCIsCall(tcbstate,do_call);
                scheduleTCB(thread);
                queue=ep_ptr_get_queue(&*epptr);
                queue=tcbEPAppend(thread,queue);
                endpoint_ptr_set_state(&mut *epptr,endpoint_state::EPState_Send as u64);
                ep_ptr_set_queue(&mut *epptr,queue);
            }
        state if state == endpoint_state::EPState_Recv as u64 =>{
            let mut queue:tcb_queue_t=ep_ptr_get_queue(&*epptr);
            let dest:*mut tcb_t=queue.head;
            queue=tcbEPDequeue(dest,queue);
            ep_ptr_set_queue(&mut *epptr,queue);
            if queue.head as u64 == 0 {
                endpoint_ptr_set_state(&mut *epptr,endpoint_state::EPState_Idle as u64);
            }
            doIPCTransfer(thread,epptr,badge,canGrant,dest);
            setThreadState(dest,_thread_state::ThreadState_Running as u64);
            possibleSwitchTo(dest);
            
            if do_call !=0 || seL4_Fault_ptr_get_seL4_FaultType(&(*thread).tcbFault) != seL4_Fault_NullFault {
                if canGrant !=0 {
                    setupCallerCap(thread,dest);
                }else{
                    setThreadState(thread,_thread_state::ThreadState_Inactive as u64);
                }
            }
        }
        _=>{panic!("");}
    }
}