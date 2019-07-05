#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(dead_code)]
#![allow(non_upper_case_globals)]
#![allow(unused_attributes)]
#![allow(unused_imports)]

use crate::object::arch_structures::*;
use crate::object::cap::*;
use crate::object::cnode::*;
use crate::object::objecttype::*;
use crate::object::tcb::*;
use crate::registerset::*;
use crate::structures::{
    _thread_state, _thread_state_t, cap_t, cap_tag_t, cte_t, dschedule, tcb_queue_t, tcb_t,
};
use crate::types::*;
use crate::object::endpoint::cancelIPC;

extern "C" {
    static mut current_extra_caps: extra_caps_t;
    //src/model/statedata.c
    static mut ksReadyQueues: [tcb_queue_t; 256];
    static mut ksReadyQueuesL1Bitmap: [word_t; 1];
    static mut ksReadyQueuesL2Bitmap: [[word_t; L2_BITMAP_SIZE]; 1];
    static mut ksCurThread: *mut tcb_t;
    static mut ksIdleThread: *mut tcb_t;
    static mut ksSchedulerAction: *mut tcb_t;
    static mut ksCurDomain: dom_t;
    static mut ksDomainTime: word_t;
    static mut ksDomScheduleIdx: word_t;
    static mut ksWorkUnitsCompleted: word_t;
    //src/config/default_domain.c
    static ksDomSchedule: [dschedule; 1];
    static ksDomScheduleLength: word_t;
    //src/arch/x86/64/kernel/thread.c
    fn Arch_activateIdleThread(tcb: *mut tcb_t); //这个函数其实是空的
    fn Arch_switchToThread(tcb: *mut tcb_t);
    fn Arch_switchToIdleThread();
    fn Arch_configureIdleThread(tcb: *mut tcb_t);
    //src/arch/x86/machine/hardware.c
    fn getRestartPC(thread: *mut tcb_t) -> word_t;
    fn setNextPC(thread: *mut tcb_t, v: word_t);
    //src/arch/x86/kernel/vspace.c
    fn lookupIPCBuffer(isReceiver: bool_t, thread: *mut tcb_t) -> word_t;
    //src/api/faults.c
    fn handleFaultReply(receiver: *mut tcb_t, sender: *mut tcb_t) -> bool_t;
    fn setMRs_fault(
        sender: *mut tcb_t,
        receiver: *mut tcb_t,
        receiveIPCBuffer: *mut word_t,
    ) -> word_t;
    //src/util.c
    fn rust_clzl(x: u64) -> i64;
    fn kprintf(format: *const u8, ...);
}

#[allow(unused_variables)]
#[inline]
pub unsafe fn Arch_getSanitiseRegisterInfo(thread: *mut tcb_t) -> bool_t {
    0u64
}

//include/kernel/thread.h

#[allow(unused_variables)]
#[inline]
pub fn ready_queues_index(dom: word_t, prio: word_t) -> word_t {
    prio
}

#[inline]
pub fn l1index_to_prio(l1index: word_t) -> word_t {
    l1index << 6
}

#[inline]
pub fn prio_to_l1index(prio: u64) -> u64 {
    prio >> 6
}

#[inline]
fn isRunnable(thread: *const tcb_t) -> bool_t {
    let state = unsafe { thread_state_get_tsType(&(*thread).tcbState) };
    if state == _thread_state::ThreadState_Running as u64
        || state == _thread_state::ThreadState_Restart as u64
    {
        true as bool_t
    } else {
        false as bool_t
    }
}

pub const L2_BITMAP_SIZE: usize = (256 + (1 << 6) - 1) / (1 << 6);
#[inline]
pub fn invert_l1index(l1index: word_t) -> word_t {
    L2_BITMAP_SIZE as u64 - 1 - l1index
}

#[inline]
unsafe fn getHighestPrio(dom: word_t) -> prio_t {
    let l1index: word_t =
        (wordBits as i64 - 1 - rust_clzl(node_state!(ksReadyQueuesL1Bitmap)[dom as usize])) as u64;
    let l1index_inverted: word_t = invert_l1index(l1index);
    let l2index: word_t = (wordBits as i64
        - 1
        - rust_clzl(node_state!(ksReadyQueuesL2Bitmap)[dom as usize][l1index_inverted as usize]))
        as u64;
    l1index_to_prio(l1index) | l2index
}

#[inline]
unsafe fn isHighestPrio(dom: word_t, prio: prio_t) -> bool_t {
    (node_state!(ksReadyQueuesL1Bitmap)[dom as usize] == 0 || prio >= getHighestPrio(dom)) as u64
}

//src/kernel/thread.c

//线程控制相关函数

#[no_mangle]
pub unsafe extern "C" fn isBlocked(thread: *const tcb_t) -> bool_t {
    let tcbState = &(*thread).tcbState;
    let tsType = thread_state_get_tsType(tcbState);
    if tsType == (_thread_state::ThreadState_Inactive as u64)
        || tsType == (_thread_state::ThreadState_BlockedOnReceive as u64)
        || tsType == (_thread_state::ThreadState_BlockedOnSend as u64)
        || tsType == (_thread_state::ThreadState_BlockedOnReply as u64)
    {
        _bool::r#true as u64
    } else {
        _bool::r#false as u64
    }
}

#[no_mangle]
#[link_section = ".boot.text"]
pub unsafe extern "C" fn configureIdleThread(tcb: *mut tcb_t) {
    Arch_configureIdleThread(tcb);
    setThreadState(tcb, _thread_state::ThreadState_IdleThreadState as u64);
}

#[no_mangle]
pub unsafe extern "C" fn activateThread() {
    let tsType = thread_state_get_tsType(&(*(node_state!(ksCurThread))).tcbState);
    if tsType == (_thread_state::ThreadState_Running as u64) {
        return;
    } else if tsType == (_thread_state::ThreadState_Restart as u64) {
        let pc: word_t = getRestartPC(node_state!(ksCurThread));
        setNextPC(node_state!(ksCurThread), pc);
        setThreadState(
            node_state!(ksCurThread),
            _thread_state::ThreadState_Running as u64,
        );
    } else if tsType == (_thread_state::ThreadState_IdleThreadState as u64) {
        Arch_activateIdleThread(node_state!(ksCurThread));
    } else {
        panic!("Current thread is blocked"); //原文是fail()
    }
}

#[no_mangle]
pub unsafe extern "C" fn suspend(target: *mut tcb_t) {
    cancelIPC(target);
    setThreadState(target, _thread_state::ThreadState_Inactive as u64);
    tcbSchedDequeue(target);
}

#[no_mangle]
pub unsafe extern "C" fn restart(target: *mut tcb_t) {
    if isBlocked(target) != 0 {
        cancelIPC(target);
        setupReplyMaster(target);
        setThreadState(target, _thread_state::ThreadState_Restart as u64);
        tcbSchedEnqueue(target);
        possibleSwitchTo(target);
    }
}

//消息传递相关函数

const seL4_Fault_NullFault: u64 = 0;
#[no_mangle]
pub unsafe extern "C" fn doIPCTransfer(
    sender: *mut tcb_t,
    endpoint: *mut endpoint_t,
    badge: word_t,
    grant: bool_t,
    receiver: *mut tcb_t,
) {
    let receiveBuffer = lookupIPCBuffer(_bool::r#true as u64, receiver) as *mut word_t;
    if seL4_Fault_get_seL4_FaultType(&(*sender).tcbFault) == seL4_Fault_NullFault {
        let sendBuffer = lookupIPCBuffer(_bool::r#false as u64, sender) as *mut word_t;
        doNormalTransfer(
            sender,
            sendBuffer,
            endpoint,
            badge,
            grant,
            receiver,
            receiveBuffer,
        );
    } else {
        doFaultTransfer(badge, sender, receiver, receiveBuffer);
    }
}

#[no_mangle]
pub unsafe extern "C" fn doReplyTransfer(
    sender: *mut tcb_t,
    receiver: *mut tcb_t,
    slot: *mut cte_t,
) {
    if seL4_Fault_get_seL4_FaultType(&(*receiver).tcbFault) == seL4_Fault_NullFault {
        doIPCTransfer(
            sender,
            0 as *mut endpoint_t,
            0,
            _bool::r#true as u64,
            receiver,
        );
        cteDeleteOne(slot);
        setThreadState(receiver, _thread_state::ThreadState_Running as u64);
        possibleSwitchTo(receiver);
    } else {
        cteDeleteOne(slot);
        let restart: bool_t = handleFaultReply(receiver, sender);
        (*receiver).tcbFault = seL4_Fault_NullFault_new();
        if restart != 0 {
            setThreadState(receiver, _thread_state::ThreadState_Restart as u64);
            possibleSwitchTo(receiver);
        } else {
            setThreadState(receiver, _thread_state::ThreadState_Inactive as u64);
        }
    }
}

const msgInfoRegister: u32 = 1;
const badgeRegister: u32 = 0;
const EXCEPTION_NONE: u64 = 0;
type exception_t = word_t;
#[no_mangle]
pub unsafe extern "C" fn doNormalTransfer(
    sender: *mut tcb_t,
    sendBuffer: *mut word_t,
    endpoint: *mut endpoint_t,
    badge: word_t,
    canGrant: bool_t,
    receiver: *mut tcb_t,
    receiveBuffer: *mut word_t,
) {
    let mut tag: seL4_MessageInfo_t = messageInfoFromWord(getRegister(sender, msgInfoRegister));
    let mut caps: extra_caps_t;
    if canGrant != 0 {
        let status: exception_t = lookupExtraCaps(sender, sendBuffer, tag);
        caps = current_extra_caps;
        if status != EXCEPTION_NONE {
            caps.excaprefs[0] = 0 as cte_ptr_t;
        }
    } else {
        caps = current_extra_caps;
        caps.excaprefs[0] = 0 as cte_ptr_t;
    }

    let msgTransferred: word_t = copyMRs(
        sender,
        sendBuffer,
        receiver,
        receiveBuffer,
        seL4_MessageInfo_get_length(tag),
    );
    tag = transferCaps(tag, caps, endpoint, receiver, receiveBuffer);
    tag = seL4_MessageInfo_set_length(tag, msgTransferred);
    setRegister(receiver, msgInfoRegister, wordFromMessageInfo(tag));
    setRegister(receiver, badgeRegister, badge);
}

#[no_mangle]
pub unsafe extern "C" fn doFaultTransfer(
    badge: word_t,
    sender: *mut tcb_t,
    receiver: *mut tcb_t,
    receiverIPCBuffer: *mut word_t,
) {
    let sent: word_t = setMRs_fault(sender, receiver, receiverIPCBuffer);
    let msgInfo: seL4_MessageInfo_t = seL4_MessageInfo_new(
        seL4_Fault_get_seL4_FaultType(&(*sender).tcbFault),
        0,
        0,
        sent,
    );
    setRegister(receiver, msgInfoRegister, wordFromMessageInfo(msgInfo));
    setRegister(receiver, badgeRegister, badge);
}

//#[repr(C)]
//pub struct deriveCap_ret{
//    status:exception_t,
//    cap:cap_t
//}
//pub type deriveCap_ret_t=deriveCap_ret;

const cap_endpoint_cap: u64 = 4;
unsafe fn transferCaps(
    mut info: seL4_MessageInfo_t,
    caps: extra_caps_t,
    endpoint: *mut endpoint_t,
    receiver: *mut tcb_t,
    receiveBuffer: *mut word_t,
) -> seL4_MessageInfo_t {
    info = seL4_MessageInfo_set_extraCaps(info, 0);
    info = seL4_MessageInfo_set_capsUnwrapped(info, 0);
    if (caps.excaprefs[0] == 0 as cte_ptr_t) || (receiveBuffer == 0 as *mut word_t) {
        return info;
    }
    let mut destSlot: *mut cte_t = getReceiveSlots(receiver, receiveBuffer);
    let mut i: usize = 0;
    while i < seL4_MsgMaxExtraCaps && caps.excaprefs[i] != 0 as cte_ptr_t {
        let slot: *mut cte_t = caps.excaprefs[i];
        let cap: cap_t = (*slot).cap;

        if cap_get_capType(cap) == cap_endpoint_cap
            && cap_endpoint_cap_get_capEPPtr(cap) == endpoint as u64
        {
            setExtraBadge(
                receiveBuffer,
                cap_endpoint_cap_get_capEPBadge(cap),
                i as u64,
            );
            info = seL4_MessageInfo_set_capsUnwrapped(
                info,
                seL4_MessageInfo_get_capsUnwrapped(info) | (1 << i),
            );
        } else {
            if destSlot == 0 as cte_ptr_t {
                break;
            }

            let dc_ret: deriveCap_ret_t = deriveCap(slot, cap);
            if dc_ret.status != EXCEPTION_NONE {
                break;
            }
            if cap_get_capType(dc_ret.cap) == cap_tag_t::cap_null_cap as u64 {
                break;
            }

            cteInsert(dc_ret.cap, slot, destSlot);
            destSlot = 0 as cte_ptr_t;
        }

        i += 1;
    }

    seL4_MessageInfo_set_extraCaps(info, i as u64)
}

#[no_mangle]
pub unsafe extern "C" fn doNBRecvFailedTransfer(thread: *mut tcb_t) {
    setRegister(thread, badgeRegister, 0);
}

//调度相关

pub unsafe fn nextDomain() {
    ksDomScheduleIdx += 1;
    if ksDomScheduleIdx >= ksDomScheduleLength {
        ksDomScheduleIdx = 0;
    }
    ksWorkUnitsCompleted = 0;
    ksCurDomain = ksDomSchedule[ksDomScheduleIdx as usize].domain;
    ksDomainTime = ksDomSchedule[ksDomScheduleIdx as usize].length;
}

pub unsafe fn scheduleChooseNewThread() {
    if ksDomainTime == 0 {
        nextDomain();
    }
    chooseThread();
}

const SchedulerAction_ResumeCurrentThread: *mut tcb_t = 0 as *mut tcb_t;
const SchedulerAction_ChooseNewThread: *mut tcb_t = 1 as *mut tcb_t;
#[no_mangle]
pub unsafe extern "C" fn schedule() {
    if node_state!(ksSchedulerAction) != SchedulerAction_ResumeCurrentThread {
        let was_runnable: bool;
        if isRunnable(node_state!(ksCurThread)) != 0 {
            was_runnable = true;
            tcbSchedEnqueue(node_state!(ksCurThread));
        } else {
            was_runnable = false;
        }

        if node_state!(ksSchedulerAction) == SchedulerAction_ChooseNewThread {
            scheduleChooseNewThread();
        } else {
            let candidate: *mut tcb_t = node_state!(ksSchedulerAction);
            let fastfail = (node_state!(ksCurThread) == node_state!(ksIdleThread))
                || ((*candidate).tcbPriority < (*node_state!(ksCurThread)).tcbPriority);
            if fastfail && (isHighestPrio(ksCurDomain, (*candidate).tcbPriority) == 0) {
                tcbSchedEnqueue(candidate);
                node_state!(ksSchedulerAction) = SchedulerAction_ChooseNewThread;
                scheduleChooseNewThread();
            } else if was_runnable
                && ((*candidate).tcbPriority == (*node_state!(ksCurThread)).tcbPriority)
            {
                tcbSchedAppend(candidate);
                node_state!(ksSchedulerAction) = SchedulerAction_ChooseNewThread;
                scheduleChooseNewThread();
            } else {
                switchToThread(candidate);
            }
        }
    }
    node_state!(ksSchedulerAction) = SchedulerAction_ResumeCurrentThread;
}

#[no_mangle]
pub unsafe extern "C" fn chooseThread() {
    let dom: word_t = 0;
    if node_state!(ksReadyQueuesL1Bitmap)[dom as usize] != 0 {
        let prio: word_t = getHighestPrio(dom);
        let thread: *mut tcb_t =
            node_state!(ksReadyQueues)[ready_queues_index(dom, prio) as usize].head;
        switchToThread(thread);
    } else {
        switchToIdleThread();
    }
}

#[no_mangle]
pub unsafe extern "C" fn switchToThread(thread: *mut tcb_t) {
    Arch_switchToThread(thread);
    tcbSchedDequeue(thread);
    node_state!(ksCurThread) = thread;
}

#[no_mangle]
pub unsafe extern "C" fn switchToIdleThread() {
    Arch_switchToIdleThread();
    node_state!(ksCurThread) = node_state!(ksIdleThread);
}

//设置状态相关

#[no_mangle]
pub unsafe extern "C" fn setDomain(tptr: *mut tcb_t, dom: dom_t) {
    tcbSchedDequeue(tptr);
    (*tptr).tcbDomain = dom;
    if isRunnable(tptr) != 0 {
        tcbSchedEnqueue(tptr);
    }
    if tptr == node_state!(ksCurThread) {
        rescheduleRequired();
    }
}

#[no_mangle]
pub unsafe extern "C" fn setMCPriority(tptr: *mut tcb_t, mcp: prio_t) {
    (*tptr).tcbMCP = mcp;
}

#[no_mangle]
pub unsafe extern "C" fn setPriority(tptr: *mut tcb_t, prio: prio_t) {
    tcbSchedDequeue(tptr);
    (*tptr).tcbPriority = prio;
    if isRunnable(tptr) != 0 {
        tcbSchedEnqueue(tptr);
        rescheduleRequired();
    }
}

#[no_mangle]
pub unsafe extern "C" fn possibleSwitchTo(target: *mut tcb_t) {
    //ignore smp
    if ksCurDomain != (*target).tcbDomain {
        tcbSchedEnqueue(target);
    } else if node_state!(ksSchedulerAction) != SchedulerAction_ResumeCurrentThread {
        rescheduleRequired();
        tcbSchedEnqueue(target);
    } else {
        node_state!(ksSchedulerAction) = target;
    }
}

#[no_mangle]
pub unsafe extern "C" fn setThreadState(tptr: *mut tcb_t, ts: _thread_state_t) {
    let tcbState = &mut (*tptr).tcbState;
    thread_state_ptr_set_tsType(tcbState, ts);
    scheduleTCB(tptr);
}

#[no_mangle]
pub unsafe extern "C" fn scheduleTCB(tptr: *mut tcb_t) {
    if tptr == node_state!(ksCurThread)
        && node_state!(ksSchedulerAction) == SchedulerAction_ResumeCurrentThread
        && (isRunnable(tptr) == 0)
    {
        rescheduleRequired();
    }
}

const CONFIG_TIME_SLICE: u64 = 5;
#[no_mangle]
pub unsafe extern "C" fn timerTick() {
    if thread_state_get_tsType(&(*node_state!(ksCurThread)).tcbState)
        == _thread_state::ThreadState_Running as u64
    {
        if (*node_state!(ksCurThread)).tcbTimeSlice > 1 {
            (*node_state!(ksCurThread)).tcbTimeSlice -= 1;
        } else {
            (*node_state!(ksCurThread)).tcbTimeSlice = CONFIG_TIME_SLICE;
            tcbSchedAppend(node_state!(ksCurThread));
            rescheduleRequired();
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn rescheduleRequired() {
    if node_state!(ksSchedulerAction) != SchedulerAction_ResumeCurrentThread
        && node_state!(ksSchedulerAction) != SchedulerAction_ChooseNewThread
    {
        tcbSchedEnqueue(node_state!(ksSchedulerAction));
    }
    node_state!(ksSchedulerAction) = SchedulerAction_ChooseNewThread;
}
