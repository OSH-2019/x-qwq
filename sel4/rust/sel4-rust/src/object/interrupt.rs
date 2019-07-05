#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(non_upper_case_globals)]
#![allow(unused_attributes)]

use crate::cspace::*;
use crate::errors::*;
use crate::failures::*;
use crate::invocation::*;
use crate::model::statedata::*;
use crate::object::arch_structures::*;
use crate::object::cap::*;
use crate::object::cnode::*;
use crate::object::objecttype::*;
use crate::registerset::*;
use crate::structures::*;
use crate::syscall::*;
use crate::thread::*;
use crate::types::*;

extern "C" {
    static mut intStateIRQTable: [u64; 126];
    static mut intStateIRQNode: *mut cte_t;
    static mut x86KSIRQState: [x86_irq_state_t; 126];
    static mut ksCurThread: *mut tcb_t;
    static mut current_syscall_error: syscall_error_t;
    fn Arch_decodeIRQControlInvocation(
        invLabel: u64,
        length: u64,
        srcSlot: *mut cte_t,
        excaps: extra_caps_t,
        buffer: *mut u64,
    ) -> u64;
    fn ioapic_mask(mask: bool_t, ioapic: u32, pin: u32);
    fn Arch_checkIRQ(irq_w: u64) -> u64;
}

#[no_mangle]
pub unsafe extern "C" fn decodeIRQControlInvocation(
    invLabel: u64,
    length: u64,
    srcSlot: *mut cte_t,
    excaps: extra_caps_t,
    buffer: *mut u64,
) -> u64 {
    if invLabel == invocation_label::IRQIssueIRQHandler as u64 {
        if length < 3 || excaps.excaprefs[0] as u64 == 0u64 {
            current_syscall_error.type_ = seL4_Error::seL4_TruncatedMessage as u64;
            return exception::EXCEPTION_SYSCALL_ERROR as u64;
        }
        let irq_w = getSyscallArg(0, buffer);
        let irq = irq_w as u8;
        let index = getSyscallArg(1, buffer);
        let depth = getSyscallArg(2, buffer);
        let cnodeCap = (*excaps.excaprefs[0]).cap;
        let mut status = Arch_checkIRQ(irq_w);
        if status != 0u64 {
            return status;
        }
        if isIRQActive(irq) != 0u64 {
            current_syscall_error.type_ = seL4_Error::seL4_RevokeFirst as u64;
            userError!("Rejecting request for IRQ %u. Already active.", irq as i32);
            return exception::EXCEPTION_SYSCALL_ERROR as u64;
        }
        let lu_ret = lookupTargetSlot(cnodeCap, index, depth);
        if lu_ret.status != 0u64 {
            userError!(
                "Target slot for new IRQ Handler cap invalid: cap %lu, IRQ %u.",
                getExtraCPtr(buffer, 0),
                irq as i32
            );
            return lu_ret.status;
        }
        let destSlot = lu_ret.slot;
        status = ensureEmptySlot(destSlot);
        if status != 0u64 {
            userError!(
                "Target slot for new IRQ Handler cap not empty: cap %lu, IRQ %u.",
                getExtraCPtr(buffer, 0),
                irq as i32
            );
            return status;
        }
        setThreadState(
            node_state!(ksCurThread),
            _thread_state::ThreadState_Restart as u64,
        );
        return invokeIRQControl(irq, destSlot, srcSlot);
    }
    Arch_decodeIRQControlInvocation(invLabel, length, srcSlot, excaps, buffer)
}

#[no_mangle]
pub unsafe extern "C" fn invokeIRQControl(
    irq: u8,
    handlerSlot: *mut cte_t,
    controlSlot: *mut cte_t,
) -> u64 {
    setIRQState(irq_state::IRQSignal as u64, irq);
    cteInsert(
        cap_irq_handler_cap_new(irq as u64),
        controlSlot,
        handlerSlot,
    );
    0u64
}

#[no_mangle]
#[allow(clippy::if_same_then_else)]
pub unsafe extern "C" fn decodeIRQHandlerInvocation(
    invLabel: u64,
    irq: u64,
    excaps: extra_caps_t,
) -> u64 {
    if invLabel == invocation_label::IRQAckIRQ as u64 {
        setThreadState(
            node_state!(ksCurThread),
            _thread_state::ThreadState_Restart as u64,
        );
        invokeIRQHandler_AckIRQ(irq as u8);
        return 0u64;
    } else if invLabel == invocation_label::IRQSetIRQHandler as u64 {
        if excaps.excaprefs[0] as u64 == 0u64 {
            current_syscall_error.type_ = seL4_Error::seL4_TruncatedMessage as u64;
            return exception::EXCEPTION_SYSCALL_ERROR as u64;
        }
        let ntfnCap = (*excaps.excaprefs[0]).cap;
        let slot = excaps.excaprefs[0];
        if cap_get_capType(ntfnCap) != cap_tag_t::cap_notification_cap as u64
            || cap_notification_cap_get_capNtfnCanSend(ntfnCap) == 0u64
        {
            if cap_get_capType(ntfnCap) != cap_tag_t::cap_notification_cap as u64 {
                userError!("IRQSetHandler: provided cap is not an notification capability.");
            } else {
                userError!("IRQSetHandler: caller does not have send rights on the endpoint.");
            }
            current_syscall_error.type_ = seL4_Error::seL4_InvalidCapability as u64;
            current_syscall_error.invalidCapNumber = 0;
            return exception::EXCEPTION_SYSCALL_ERROR as u64;
        }
        setThreadState(
            node_state!(ksCurThread),
            _thread_state::ThreadState_Restart as u64,
        );
        invokeIRQHandler_SetIRQHandler(irq as u8, ntfnCap, slot);
        return 0u64;
    } else if invLabel == invocation_label::IRQClearIRQHandler as u64 {
        setThreadState(
            node_state!(ksCurThread),
            _thread_state::ThreadState_Restart as u64,
        );
        invokeIRQHandler_ClearIRQHandler(irq as u8);
        return 0u64;
    }
    userError!("IRQHandler: Illegal operation.");
    current_syscall_error.type_ = seL4_Error::seL4_IllegalOperation as u64;
    return exception::EXCEPTION_SYSCALL_ERROR as u64;
}

#[no_mangle]
pub unsafe extern "C" fn invokeIRQHandler_AckIRQ(irq: u8) {
    maskInterrupt(0u64, irq);
}

#[no_mangle]
pub unsafe extern "C" fn invokeIRQHandler_SetIRQHandler(irq: u8, cap: cap_t, slot: *mut cte_t) {
    let irqSlot = intStateIRQNode.offset(irq as isize);
    cteDeleteOne(irqSlot);
    cteInsert(cap, slot, irqSlot);
}

#[no_mangle]
pub unsafe extern "C" fn invokeIRQHandler_ClearIRQHandler(irq: u8) {
    let irqSlot = intStateIRQNode.offset(irq as isize);
    cteDeleteOne(irqSlot);
}

#[no_mangle]
pub unsafe extern "C" fn deletingIRQHandler(irq: u8) {
    let slot = intStateIRQNode.offset(irq as isize);
    cteDeleteOne(slot);
}

#[no_mangle]
pub unsafe extern "C" fn deletedIRQHandler(irq: u8) {
    setIRQState(irq_state::IRQInactive as u64, irq);
}

#[no_mangle]
pub unsafe extern "C" fn isIRQActive(irq: u8) -> bool_t {
    (intStateIRQTable[irq as usize] != irq_state::IRQInactive as u64) as u64
}

#[no_mangle]
pub unsafe extern "C" fn setIRQState(irqState: u64, irq: u8) {
    intStateIRQTable[irq as usize] = irqState;
    maskInterrupt((irqState == irq_state::IRQInactive as u64) as u64, irq);
}

#[inline]
unsafe fn updateIRQState(irq: u8, state: x86_irq_state_t) {
    x86KSIRQState[irq as usize] = state;
}

#[inline]
unsafe fn maskInterrupt(disable: bool_t, irq: u8) {
    if irq >= 16 && irq <= 155 {
        let mut state = x86KSIRQState[irq as usize];
        if x86_irq_state_get_irqType(state) == 1u32 {
            let ioapic = x86_irq_state_irq_ioapic_get_id(state);
            let pin = x86_irq_state_irq_ioapic_get_pin(state);
            ioapic_mask(disable, ioapic, pin);
            state = x86_irq_state_irq_ioapic_set_masked(state, disable as u32);
            updateIRQState(irq, state);
        }
    }
}
