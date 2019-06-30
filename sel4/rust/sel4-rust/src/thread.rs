#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(dead_code)]
#![allow(non_upper_case_globals)]
#![allow(unused_attributes)]

use crate::types::*;
use crate::structures::{tcb_t,_thread_state,_thread_state_t,cte_t,cap_t,cap_tag_t};
use crate::object::arch_structures::*;
use crate::model::statedata::*;
use crate::registerset::*;

//src/kernel/thread.c

extern "C"{
    static mut current_extra_caps:extra_caps_t;
    fn possibleSwitchTo(target:*mut tcb_t);
    //src/model/statedata.c
    static mut ksCurDomain:dom_t;
    //src/arch/x86/64/kernel/thread.c
    fn Arch_activateIdleThread(tcb:*mut tcb_t); //这个函数其实是空的
    //src/object/cnode.c
    fn setupReplyMaster(thread:*mut tcb_t);
    fn cteDeleteOne(slot:*mut cte_t);
    fn getReceiveSlots(thread:*mut tcb_t,buffer:*mut word_t)->*mut cte_t;
    fn cteInsert(newCap:cap_t,srcSlot:*mut cte_t,destSlot:*mut cte_t);
    //src/object/endpoint.c
    fn cancelIPC(tptr:*mut tcb_t);
    fn scheduleTCB(tptr:*mut tcb_t);
    //src/kernel/tcb.c
    fn tcbSchedDequeue(tcb:*mut tcb_t);
    fn tcbSchedEnqueue(tcb:*mut tcb_t);
    fn lookupExtraCaps(thread:*mut tcb_t,bufferPtr:*mut word_t,info:seL4_MessageInfo_t)->exception_t;
    fn copyMRs(sender:*mut tcb_t,sendBuf:*mut word_t,receiver:*mut tcb_t,recvBuf:*mut word_t,n:word_t)->word_t;
    fn setExtraBadge(bufferPtr:*mut word_t,badge:word_t,i:word_t);
    //src/arch/x86/machine/hardware.c
    fn getRestartPC(thread:*mut tcb_t)->word_t;
    fn setNextPC(thread:*mut tcb_t,v:word_t);
    //src/arch/x86/kernel/vspace.c
    fn lookupIPCBuffer(isReceiver:bool_t,thread:*mut tcb_t)->word_t;
    //src/api/faults.c
    fn handleFaultReply(receiver:*mut tcb_t,sender:*mut tcb_t)->bool_t;
    fn setMRs_fault(sender:*mut tcb_t,receiver:*mut tcb_t,receiveIPCBuffer:*mut word_t)->word_t;
    //src/object/objecttype.c
    fn deriveCap(slot:*mut cte_t,cap:cap_t)->deriveCap_ret_t;
}

//线程控制相关函数

#[no_mangle]
pub extern "C" fn isBlocked(thread:*const tcb_t)->bool_t{
    let tcbState=unsafe{
        &(*thread).tcbState
    };
    let tsType=thread_state_get_tsType(tcbState);
    if tsType==(_thread_state::ThreadState_Inactive as u64)
        || tsType==(_thread_state::ThreadState_BlockedOnReceive as u64)
        || tsType==(_thread_state::ThreadState_BlockedOnSend as u64)
        || tsType==(_thread_state::ThreadState_BlockedOnReply as u64) {
        _bool::r#true as u64
    }else{
        _bool::r#false as u64
    }
}

#[no_mangle]
pub unsafe extern "C" fn activateThread(){
    let tsType=thread_state_get_tsType(&(*(node_state!(ksCurThread))).tcbState);
    if tsType==(_thread_state::ThreadState_Running as u64) {
        return;
    }else if tsType==(_thread_state::ThreadState_Restart as u64) {
        let pc:word_t=getRestartPC(node_state!(ksCurThread));
        setNextPC(node_state!(ksCurThread),pc);
        setThreadState(node_state!(ksCurThread),_thread_state::ThreadState_Running as u64);
    }else if tsType==(_thread_state::ThreadState_IdleThreadState as u64) {
        Arch_activateIdleThread(node_state!(ksCurThread));
    }else{
        panic!("Current thread is blocked"); //原文是fail()
    }
}

#[no_mangle]
pub unsafe extern "C" fn suspend(target:*mut tcb_t){
    cancelIPC(target);
    setThreadState(target,_thread_state::ThreadState_Inactive as u64);
    tcbSchedDequeue(target);
}

#[no_mangle]
pub unsafe extern "C" fn restart(target:*mut tcb_t){
    if isBlocked(target)!=0 {
        cancelIPC(target);
        setupReplyMaster(target);
        setThreadState(target,_thread_state::ThreadState_Restart as u64);
        tcbSchedEnqueue(target);
        possibleSwitchTo(target);
    }
}

//消息传递相关函数

const seL4_Fault_NullFault:u64=0;
#[no_mangle]
pub unsafe extern "C" fn doIPCTransfer(sender:*mut tcb_t,endpoint:*mut endpoint_t,
    badge:word_t,grant:bool_t,receiver:*mut tcb_t){
    let receiveBuffer=lookupIPCBuffer(_bool::r#true as u64,receiver) as *mut word_t;
    if seL4_Fault_get_seL4_FaultType(&(*sender).tcbFault) == seL4_Fault_NullFault {
        let sendBuffer=lookupIPCBuffer(_bool::r#false as u64,sender) as *mut word_t;
        doNormalTransfer(sender, sendBuffer, endpoint, badge, grant,
                         receiver, receiveBuffer);
    }else{
        doFaultTransfer(badge, sender, receiver, receiveBuffer);
    }
}

#[no_mangle]
pub unsafe extern "C" fn doReplyTransfer(sender:*mut tcb_t,receiver:*mut tcb_t,slot:*mut cte_t){
    if seL4_Fault_get_seL4_FaultType(&(*receiver).tcbFault) == seL4_Fault_NullFault{
        doIPCTransfer(sender,0 as *mut endpoint_t,0,_bool::r#true as u64,receiver);
        cteDeleteOne(slot);
        setThreadState(receiver,_thread_state::ThreadState_Running as u64);
        possibleSwitchTo(receiver);
    }else{
        cteDeleteOne(slot);
        let restart:bool_t=handleFaultReply(receiver,sender);
        (*receiver).tcbFault=seL4_Fault_NullFault_new();
        if restart!=0 {
            setThreadState(receiver,_thread_state::ThreadState_Restart as u64);
            possibleSwitchTo(receiver);
        }else{
            setThreadState(receiver,_thread_state::ThreadState_Inactive as u64);
        }
    }
}

const msgInfoRegister:u32=1;
const badgeRegister:u32=0;
const EXCEPTION_NONE:u64=0;
type exception_t=word_t;
#[no_mangle]
pub unsafe extern "C" fn doNormalTransfer(sender:*mut tcb_t,sendBuffer:*mut word_t,
    endpoint:*mut endpoint_t,badge:word_t,canGrant:bool_t,receiver:*mut tcb_t,receiveBuffer:*mut word_t){
    let mut tag:seL4_MessageInfo_t=messageInfoFromWord(getRegister(sender,msgInfoRegister));
    let mut caps:extra_caps_t;
    if canGrant!=0 {
        let status:exception_t=lookupExtraCaps(sender,sendBuffer,tag);
        caps=current_extra_caps;
        if status!=EXCEPTION_NONE {
            caps.excaprefs[0] = 0 as cte_ptr_t;
        }
    }else{
        caps=current_extra_caps;
        caps.excaprefs[0]=0 as cte_ptr_t;
    }
    
    let msgTransferred:word_t=copyMRs(sender,sendBuffer,receiver,receiveBuffer,
        seL4_MessageInfo_get_length(tag));
    tag = transferCaps(tag, caps, endpoint, receiver, receiveBuffer);
    tag = seL4_MessageInfo_set_length(tag, msgTransferred);
    setRegister(receiver, msgInfoRegister, wordFromMessageInfo(tag));
    setRegister(receiver, badgeRegister, badge);
}

#[no_mangle]
pub unsafe extern "C" fn doFaultTransfer(badge:word_t,sender:*mut tcb_t,receiver:*mut tcb_t,receiverIPCBuffer:*mut word_t){
    let sent:word_t=setMRs_fault(sender,receiver,receiverIPCBuffer);
    let msgInfo:seL4_MessageInfo_t=seL4_MessageInfo_new(
        seL4_Fault_get_seL4_FaultType(&(*sender).tcbFault), 0, 0, sent);
    setRegister(receiver, msgInfoRegister, wordFromMessageInfo(msgInfo));
    setRegister(receiver, badgeRegister, badge);
}

#[repr(C)]
pub struct deriveCap_ret{
    status:exception_t,
    cap:cap_t
}
pub type deriveCap_ret_t=deriveCap_ret;

const cap_endpoint_cap:u64=4;
unsafe fn transferCaps(mut info:seL4_MessageInfo_t,caps:extra_caps_t,endpoint:*mut endpoint_t,
    receiver:*mut tcb_t,receiveBuffer:*mut word_t)->seL4_MessageInfo_t{
    info = seL4_MessageInfo_set_extraCaps(info, 0);
    info = seL4_MessageInfo_set_capsUnwrapped(info, 0);
    if (caps.excaprefs[0] == 0 as cte_ptr_t) || (receiveBuffer == 0 as *mut word_t) {
        return info;
    }
    let mut destSlot:*mut cte_t=getReceiveSlots(receiver, receiveBuffer);
    let mut i:usize=0;
    while i < seL4_MsgMaxExtraCaps && caps.excaprefs[i] != 0 as cte_ptr_t {
        let slot:*mut cte_t=caps.excaprefs[i];
        let cap:cap_t=(*slot).cap;
        
        if cap_get_capType(cap) == cap_endpoint_cap && cap_endpoint_cap_get_capEPPtr(cap) == endpoint as u64 {
            setExtraBadge(receiveBuffer,cap_endpoint_cap_get_capEPBadge(cap),i as u64);
            info = seL4_MessageInfo_set_capsUnwrapped(info,
                seL4_MessageInfo_get_capsUnwrapped(info) | (1 << i));

        }else{
            if destSlot==0 as cte_ptr_t {
                break;
            }
            
            let dc_ret:deriveCap_ret_t=deriveCap(slot, cap);
            if dc_ret.status!=EXCEPTION_NONE {
                break;
            }
            if cap_get_capType(dc_ret.cap)==cap_tag_t::cap_null_cap as u64 {
                break;
            }
            
            cteInsert(dc_ret.cap,slot,destSlot);
            destSlot=0 as cte_ptr_t;
        }
        
        i+=1;
    }
    
    seL4_MessageInfo_set_extraCaps(info,i as u64)
}

#[no_mangle]
pub unsafe extern "C" fn doNBRecvFailedTransfer(thread:*mut tcb_t){
    setRegister(thread, badgeRegister, 0);
}

//

#[no_mangle]
pub unsafe extern "C" fn setThreadState(tptr:*mut tcb_t,ts:_thread_state_t){
    let tcbState=&mut(*tptr).tcbState;
    thread_state_ptr_set_tsType(tcbState,ts);
    scheduleTCB(tptr);
}

