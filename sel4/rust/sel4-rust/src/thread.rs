#![allow(non_snake_case)]
//#![allow(non_camel_case_types)]
#![allow(dead_code)]
//#![allow(non_upper_case_globals)]

use crate::types::{word_t,_bool,bool_t,extra_caps_t};
use crate::structures::{tcb_t,_thread_state,_thread_state_t};
use crate::object::arch_structures::{thread_state_get_tsType,thread_state_ptr_set_tsType};
use crate::model::statedata::*;

extern "C"{
    static mut current_extra_caps:extra_caps_t;
    //src/arch/x86/64/kernel/thread.c
    fn Arch_activateIdleThread(tcb:*mut tcb_t); //这个函数其实是空的
    //src/object/endpoint.c
    fn cancelIPC(tptr:*mut tcb_t);
    fn scheduleTCB(tptr:*mut tcb_t);
    //src/kernel/tcb.c
    fn tcbSchedDequeue(tcb:*mut tcb_t);
    //src/arch/x86/machine/hardware.c
    fn getRestartPC(thread:*mut tcb_t)->word_t;
    fn setNextPC(thread:*mut tcb_t,v:word_t);
}


//src/kernel/thread.c
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

//#[no_mangle]
//pub unsafe extern "C" fn restart()

#[no_mangle]
pub unsafe extern "C" fn setThreadState(tptr:*mut tcb_t,ts:_thread_state_t){
    let tcbState=&mut(*tptr).tcbState;
    thread_state_ptr_set_tsType(tcbState,ts);
    scheduleTCB(tptr);
}

