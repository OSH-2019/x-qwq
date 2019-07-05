#![allow(non_upper_case_globals)]
#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(unused_imports)]
#![allow(unused_attributes)]

use crate::cspace::*;
use crate::errors::*;
use crate::failures::*;
use crate::object::arch_structures::*;
use crate::object::cap::*;
use crate::object::cnode::*;
use crate::object::notification::*;
use crate::object::objecttype::*;
use crate::registerset::*;
use crate::structures::*;
use crate::syscall::*;
use crate::thread::*;
use crate::types::*;

extern "C" {
    static mut ksCurThread: *mut tcb_t;
    static mut ksReadyQueues: [tcb_queue_t; 256];
    static mut ksReadyQueuesL1Bitmap: [u64; 1];
    static mut ksReadyQueuesL2Bitmap: [[u64; L2_BITMAP_SIZE]; 1];
    static mut current_extra_caps: extra_caps_t;
    static mut current_syscall_error: syscall_error_t;
    static mut current_fault: seL4_Fault_t;
    static mut current_lookup_fault: lookup_fault_t;
    fn getRestartPC(thread: *mut tcb_t) -> u64;
    fn setNextPC(thread: *mut tcb_t, v: u64);
    fn lookupIPCBuffer(isReceiver: bool_t, thread: *mut tcb_t) -> *mut u64;
    fn sanitiseRegister(reg: u32, v: u64, archInfo: bool_t) -> u64;
    fn Arch_setTCBIPCBuffer(thread: *mut tcb_t, bufferAddr: u64);
    fn Arch_postModifyRegisters(tptr: *mut tcb_t);
    fn Arch_performTransfer(arch: u64, tcb_src: *mut tcb_t, dest: *mut tcb_t) -> u64;
    fn kprintf(format: *const u8, ...);
}

macro_rules! MASK {
    ($x:expr) => {
        (1u64 << ($x)) - 1u64
    };
}

pub enum thread_control_flag {
    thread_control_update_priority = 0x1,
    thread_control_update_ipc_buffer = 0x2,
    thread_control_update_space = 0x4,
    thread_control_update_mcp = 0x8,
}

#[inline]
pub unsafe fn setMR(
    receiver: *mut tcb_t,
    receiveIPCBuffer: *mut u64,
    offset: u32,
    reg: u64,
) -> u32 {
    if offset >= n_msgRegisters as u32 {
        if receiveIPCBuffer as u64 != 0u64 {
            *receiveIPCBuffer.offset((offset + 1) as isize) = reg;
            return offset + 1;
        } else {
            return n_msgRegisters as u32;
        }
    } else {
        setRegister(receiver, msgRegisters[offset as usize], reg);
        return offset + 1;
    }
}

#[inline]
pub unsafe fn setMRs_lookup_failure(
    receiver: *mut tcb_t,
    receiveIPCBuffer: *mut u64,
    luf: lookup_fault_t,
    offset: u32,
) -> u32 {
    let lufType = lookup_fault_get_lufType(luf);
    let i = setMR(receiver, receiveIPCBuffer, offset, lufType + 1);
    if lufType == lookup_fault_tag_t::lookup_fault_invalid_root as u64 {
        return i;
    } else if lufType == lookup_fault_tag_t::lookup_fault_missing_capability as u64 {
        return setMR(
            receiver,
            receiveIPCBuffer,
            offset + 1,
            lookup_fault_missing_capability_get_bitsLeft(luf),
        );
    } else if lufType == lookup_fault_tag_t::lookup_fault_depth_mismatch as u64 {
        setMR(
            receiver,
            receiveIPCBuffer,
            offset + 1,
            lookup_fault_depth_mismatch_get_bitsLeft(luf),
        );
        return setMR(
            receiver,
            receiveIPCBuffer,
            offset + 2,
            lookup_fault_depth_mismatch_get_bitsFound(luf),
        );
    } else if lufType == lookup_fault_tag_t::lookup_fault_guard_mismatch as u64 {
        setMR(
            receiver,
            receiveIPCBuffer,
            offset + 1,
            lookup_fault_guard_mismatch_get_bitsLeft(luf),
        );
        setMR(
            receiver,
            receiveIPCBuffer,
            offset + 2,
            lookup_fault_guard_mismatch_get_bitsFound(luf),
        );
        return setMR(
            receiver,
            receiveIPCBuffer,
            offset + 3,
            lookup_fault_guard_mismatch_get_bitsFound(luf),
        );
    }
    panic!("Invalid lookup failure");
}

#[allow(unused_variables)]
#[inline]
pub unsafe fn addToBitmap(cpu: u64, dom: u64, prio: u64) {
    let l1index = prio_to_l1index(prio);
    let l1index_inverted = invert_l1index(l1index);
    //ignore smp
    ksReadyQueuesL1Bitmap[dom as usize] |= 1u64 << l1index;
    ksReadyQueuesL2Bitmap[dom as usize][l1index_inverted as usize] |=
        1u64 << (prio & MASK!(wordRadix));
}

#[allow(unused_variables)]
#[inline]
pub unsafe fn removeFromBitmap(cpu: u64, dom: u64, prio: u64) {
    let l1index = prio_to_l1index(prio);
    let l1index_inverted = invert_l1index(l1index);
    //ignore smp
    ksReadyQueuesL2Bitmap[dom as usize][l1index_inverted as usize] &=
        !(1u64 << (prio & MASK!(wordRadix)));
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
        thread_state_ptr_set_tcbQueued(&mut (*tcb).tcbState, 1u64);
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
        thread_state_ptr_set_tcbQueued(&mut (*tcb).tcbState, 1u64);
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
        thread_state_ptr_set_tcbQueued(&mut (*tcb).tcbState, 0u64);
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
    cteInsert(
        cap_reply_cap_new(0u64, sender as u64),
        replySlot,
        callerSlot,
    );
}

#[no_mangle]
pub unsafe extern "C" fn deleteCallerCap(receiver: *mut tcb_t) {
    let callerSlot = tcb_ptr_cte_ptr(receiver, tcb_cnode_index::tcbCaller as u64);
    cteDeleteOne(callerSlot);
}

#[no_mangle]
pub unsafe extern "C" fn lookupExtraCaps(
    thread: *mut tcb_t,
    bufferPtr: *mut u64,
    info: seL4_MessageInfo_t,
) -> u64 {
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

#[no_mangle]
pub unsafe extern "C" fn copyMRs(
    sender: *mut tcb_t,
    sendBuf: *mut u64,
    receiver: *mut tcb_t,
    recvBuf: *mut u64,
    n: u64,
) -> u64 {
    let mut i: usize = 0;
    while i < n as usize && i < n_msgRegisters {
        setRegister(
            receiver,
            msgRegisters[i],
            getRegister(sender, msgRegisters[i]),
        );
        i += 1;
    }
    if recvBuf as u64 == 0u64 || sendBuf as u64 == 0u64 {
        return i as u64;
    }
    while i < n as usize {
        *recvBuf.offset((i + 1) as isize) = *sendBuf.offset((i + 1) as isize);
        i += 1;
    }
    i as u64
}

#[no_mangle]
pub unsafe extern "C" fn invokeTCB_Suspend(thread: *mut tcb_t) -> u64 {
    suspend(thread);
    0u64
}

#[no_mangle]
pub unsafe extern "C" fn invokeTCB_Resume(thread: *mut tcb_t) -> u64 {
    restart(thread);
    0u64
}

#[no_mangle]
pub unsafe extern "C" fn invokeTCB_ThreadControl(
    target: *mut tcb_t,
    slot: *mut cte_t,
    faultep: u64,
    mcp: prio_t,
    priority: prio_t,
    cRoot_newCap: cap_t,
    cRoot_srcSlot: *mut cte_t,
    vRoot_newCap: cap_t,
    vRoot_srcSlot: *mut cte_t,
    bufferAddr: u64,
    bufferCap: cap_t,
    bufferSrcSlot: *mut cte_t,
    updateFlags: u64,
) -> u64 {
    let tCap = cap_thread_cap_new(target as u64);
    if updateFlags & thread_control_flag::thread_control_update_space as u64 != 0u64 {
        (*target).tcbFaultHandler = faultep;
    }
    if updateFlags & thread_control_flag::thread_control_update_mcp as u64 != 0u64 {
        setMCPriority(target, mcp);
    }
    if updateFlags & thread_control_flag::thread_control_update_priority as u64 != 0u64 {
        setPriority(target, priority);
    }
    if updateFlags & thread_control_flag::thread_control_update_space as u64 != 0u64 {
        let rootSlot = tcb_ptr_cte_ptr(target, tcb_cnode_index::tcbCTable as u64);
        let e = cteDelete(rootSlot, 1u64);
        if e != 0u64 {
            return e;
        }
        if sameObjectAs(cRoot_newCap, (*cRoot_srcSlot).cap) != 0u64
            && sameObjectAs(tCap, (*slot).cap) != 0u64
        {
            cteInsert(cRoot_newCap, cRoot_srcSlot, rootSlot);
        }
    }
    if updateFlags & thread_control_flag::thread_control_update_space as u64 != 0u64 {
        let rootSlot = tcb_ptr_cte_ptr(target, tcb_cnode_index::tcbVTable as u64);
        let e = cteDelete(rootSlot, 1u64);
        if e != 0u64 {
            return e;
        }
        if sameObjectAs(vRoot_newCap, (*vRoot_srcSlot).cap) != 0u64
            && sameObjectAs(tCap, (*slot).cap) != 0u64
        {
            cteInsert(vRoot_newCap, vRoot_srcSlot, rootSlot);
        }
    }
    if updateFlags & thread_control_flag::thread_control_update_ipc_buffer as u64 != 0u64 {
        let bufferSlot = tcb_ptr_cte_ptr(target, tcb_cnode_index::tcbBuffer as u64);
        let e = cteDelete(bufferSlot, 1u64);
        if e != 0u64 {
            return e;
        }
        (*target).tcbIPCBuffer = bufferAddr;
        Arch_setTCBIPCBuffer(target, bufferAddr);
        if bufferSrcSlot as u64 != 0u64
            && sameObjectAs(bufferCap, (*bufferSrcSlot).cap) != 0u64
            && sameObjectAs(tCap, (*slot).cap) != 0u64
        {
            cteInsert(bufferCap, bufferSrcSlot, bufferSlot);
        }
        if target == node_state!(ksCurThread) {
            rescheduleRequired();
        }
    }
    0u64
}

#[no_mangle]
pub unsafe extern "C" fn invokeTCB_CopyRegisters(
    dest: *mut tcb_t,
    tcb_src: *mut tcb_t,
    suspendSource: bool_t,
    resumeTarget: bool_t,
    transferFrame: bool_t,
    transferInteger: bool_t,
    transferArch: u64,
) -> u64 {
    if suspendSource != 0u64 {
        suspend(tcb_src);
    }
    if resumeTarget != 0u64 {
        restart(dest);
    }
    if transferFrame != 0u64 {
        let mut i: usize = 0;
        while i < n_frameRegisters {
            let v = getRegister(tcb_src, frameRegisters[i]);
            setRegister(dest, frameRegisters[i], v);
            i += 1;
        }
        let pc = getRestartPC(dest);
        setNextPC(dest, pc);
    }
    if transferInteger != 0u64 {
        let mut i: usize = 0;
        while i < n_gpRegisters {
            let v = getRegister(tcb_src, gpRegisters[i]);
            setRegister(dest, gpRegisters[i], v);
            i += 1;
        }
    }
    Arch_postModifyRegisters(dest);
    if dest == node_state!(ksCurThread) {
        rescheduleRequired();
    }
    Arch_performTransfer(transferArch, tcb_src, dest)
}

#[no_mangle]
pub unsafe extern "C" fn invokeTCB_ReadRegisters(
    tcb_src: *mut tcb_t,
    suspendSource: bool_t,
    n: u64,
    arch: u64,
    call: bool_t,
) -> u64 {
    let thread = node_state!(ksCurThread);
    if suspendSource != 0u64 {
        suspend(tcb_src);
    }
    let e = Arch_performTransfer(arch, tcb_src, node_state!(ksCurThread));
    if e != 0u64 {
        return e;
    }
    if call != 0u64 {
        let ipcBuffer = lookupIPCBuffer(1u64, thread);
        let mut i: usize = 0;
        while i < n as usize && i < n_frameRegisters && i < n_msgRegisters {
            setRegister(
                thread,
                msgRegisters[i],
                getRegister(tcb_src, frameRegisters[i]),
            );
            i += 1;
        }
        if ipcBuffer as u64 != 0u64 && i < n as usize && i < n_frameRegisters {
            while i < n as usize && i < n_frameRegisters {
                *ipcBuffer.offset((i + 1) as isize) = getRegister(tcb_src, frameRegisters[i]);
                i += 1;
            }
        }
        let j = i;
        i = 0;
        while i < n_gpRegisters
            && i + n_frameRegisters < n as usize
            && i + n_frameRegisters < n_msgRegisters
        {
            setRegister(
                thread,
                msgRegisters[i + n_frameRegisters],
                getRegister(tcb_src, gpRegisters[i]),
            );
            i += 1;
        }
        if ipcBuffer as u64 != 0u64 && i < n_gpRegisters && i + n_frameRegisters < n as usize {
            while i < n_gpRegisters && i + n_frameRegisters < n as usize {
                *ipcBuffer.offset((i + n_frameRegisters + 1) as isize) =
                    getRegister(tcb_src, gpRegisters[i]);
                i += 1;
            }
        }
        setRegister(
            thread,
            msgInfoRegister,
            wordFromMessageInfo(seL4_MessageInfo_new(0, 0, 0, (i + j) as u64)),
        );
    }
    setThreadState(thread, _thread_state::ThreadState_Running as u64);
    0u64
}

#[no_mangle]
pub unsafe extern "C" fn invokeTCB_WriteRegisters(
    dest: *mut tcb_t,
    resumeTarget: bool_t,
    mut n: u64,
    arch: u64,
    buffer: *mut u64,
) -> u64 {
    let e = Arch_performTransfer(arch, node_state!(ksCurThread), dest);
    if e != 0u64 {
        return e;
    }
    if n as usize > n_frameRegisters + n_gpRegisters {
        n = (n_frameRegisters + n_gpRegisters) as u64;
    }
    let archInfo = Arch_getSanitiseRegisterInfo(dest);
    let mut i: usize = 0;
    while i < n_frameRegisters && i < n as usize {
        setRegister(
            dest,
            frameRegisters[i],
            sanitiseRegister(
                frameRegisters[i],
                getSyscallArg((i + 2) as u64, buffer),
                archInfo,
            ),
        );
        i += 1;
    }
    i = 0;
    while i < n_gpRegisters && i + n_frameRegisters < n as usize {
        setRegister(
            dest,
            gpRegisters[i],
            sanitiseRegister(
                gpRegisters[i],
                getSyscallArg((i + n_frameRegisters + 2) as u64, buffer),
                archInfo,
            ),
        );
        i += 1;
    }
    let pc = getRestartPC(dest);
    setNextPC(dest, pc);
    Arch_postModifyRegisters(dest);
    if resumeTarget != 0u64 {
        restart(dest);
    }
    if dest == node_state!(ksCurThread) {
        rescheduleRequired();
    }
    0u64
}

#[no_mangle]
pub unsafe extern "C" fn invokeTCB_NotificationControl(
    tcb: *mut tcb_t,
    ntfnPtr: *mut notification_t,
) -> u64 {
    if ntfnPtr as u64 != 0u64 {
        bindNotification(tcb, ntfnPtr);
    } else {
        unbindNotification(tcb);
    }
    0u64
}

#[no_mangle]
pub unsafe extern "C" fn setMRs_syscall_error(
    thread: *mut tcb_t,
    receiveIPCBuffer: *mut u64,
) -> u64 {
    if current_syscall_error.type_ == seL4_Error::seL4_InvalidArgument as u64 {
        return setMR(
            thread,
            receiveIPCBuffer,
            0,
            current_syscall_error.invalidArgumentNumber,
        ) as u64;
    } else if current_syscall_error.type_ == seL4_Error::seL4_InvalidCapability as u64 {
        return setMR(
            thread,
            receiveIPCBuffer,
            0,
            current_syscall_error.invalidCapNumber,
        ) as u64;
    } else if current_syscall_error.type_ == seL4_Error::seL4_IllegalOperation as u64 {
        return 0u64;
    } else if current_syscall_error.type_ == seL4_Error::seL4_RangeError as u64 {
        setMR(
            thread,
            receiveIPCBuffer,
            0,
            current_syscall_error.rangeErrorMin,
        );
        return setMR(
            thread,
            receiveIPCBuffer,
            1,
            current_syscall_error.rangeErrorMax,
        ) as u64;
    } else if current_syscall_error.type_ == seL4_Error::seL4_AlignmentError as u64 {
        return 0u64;
    } else if current_syscall_error.type_ == seL4_Error::seL4_FailedLookup as u64 {
        setMR(
            thread,
            receiveIPCBuffer,
            0,
            (current_syscall_error.failedLookupWasSource != 0u64) as u64,
        );
        return setMRs_lookup_failure(thread, receiveIPCBuffer, current_lookup_fault, 1) as u64;
    } else if current_syscall_error.type_ == seL4_Error::seL4_TruncatedMessage as u64
        || current_syscall_error.type_ == seL4_Error::seL4_DeleteFirst as u64
        || current_syscall_error.type_ == seL4_Error::seL4_RevokeFirst as u64
    {
        return 0u64;
    } else if current_syscall_error.type_ == seL4_Error::seL4_NotEnoughMemory as u64 {
        return setMR(
            thread,
            receiveIPCBuffer,
            0,
            current_syscall_error.memoryLeft,
        ) as u64;
    }
    panic!("Invalid syscall error");
}
