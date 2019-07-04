#![allow(non_upper_case_globals)]
#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(unused_imports)]
#![allow(unused_attributes)]

use crate::types::*;
use crate::structures::*;
use crate::thread::*;
use crate::object::cap::*;
use crate::object::cnode::*;
use crate::object::notification::*;
use crate::object::arch_structures::*;
use crate::object::endpoint::sendIPC;

extern "C" {
    static mut ksCurThread: *mut tcb_t;
    fn deletedIRQHandler(irq: u8);
    fn Arch_postCapDeletion(cap: cap_t);
    fn Arch_getObjectSize(t: u64) -> u64;
    fn Arch_deriveCap(slot: *mut cte_t, cap: cap_t) -> deriveCap_ret_t;
    fn Arch_finaliseCap(cap: cap_t, final_: bool_t) -> finaliseCap_ret_t;
    fn Arch_prepareThreadDelete(thread: *mut tcb_t);
    fn Arch_updateCapData(preserve: bool_t, newData: u64, cap: cap_t) -> cap_t;
    //fn deletedIRQHandler(irq: u8);
    fn Arch_sameRegionAs(cap_a: cap_t, cap_b: cap_t) -> bool_t;
    fn Arch_sameObjectAs(cap_a: cap_t, cap_b: cap_t) -> bool_t;
    fn tcbDebugRemove(tcb: *mut tcb_t);
    fn cancelAllIPC(epptr: *mut endpoint_t);
}

pub enum seL4_ObjectType {
    seL4_UntypedObject = 0,
    seL4_TCBObject = 1,
    seL4_EndpointObject = 2,
    seL4_NotificationObject = 3,
    seL4_CapTableObject = 4,
    seL4_NonArchObjectTypeCount = 5,
}

pub enum seL4_seL4ArchObjectType {
    seL4_X86_PDPObject = 5,
    seL4_X64_PML4Object = 6,
    seL4_X64_HugePageobject = 7,
    seL4_ModeObjectTypeCOunt = 8,
}

pub enum seL4_ArchObjectType {
    seL4_X86_4K = 8,
    seL4_X86_LargePageObject = 9,
    seL4_X86_PageTableObject = 10,
    seL4_X86_PageDirectoryObject = 11,
    seL4_ObjectTypeCount = 12,
}

#[no_mangle]
pub unsafe extern "C" fn getObjectSize(t: u64, userObjSize: u64) -> u64 {
    if t >= seL4_ObjectType::seL4_NonArchObjectTypeCount as u64 {
        return Arch_getObjectSize(t);
    } else if t == seL4_ObjectType::seL4_TCBObject as u64 {
        return seL4_TCBBits;
    } else if t == seL4_ObjectType::seL4_EndpointObject as u64 {
        return seL4_EndpointBits;
    } else if t == seL4_ObjectType::seL4_NotificationObject as u64 {
        return seL4_NotificationBits;
    } else if t == seL4_ObjectType::seL4_CapTableObject as u64 {
        return seL4_SlotBits + userObjSize;
    } else if t == seL4_ObjectType::seL4_UntypedObject as u64 {
        return userObjSize;
    }
    panic!("Invalid object type");
}

#[no_mangle]
pub unsafe extern "C" fn deriveCap(slot: *mut cte_t, cap: cap_t) -> deriveCap_ret_t {
    if isArchCap(cap) != 0u64 {
        return Arch_deriveCap(slot, cap);
    }
    let cap_type = cap_get_capType(cap);
    if cap_type == cap_tag_t::cap_zombie_cap as u64 {
        return deriveCap_ret_t {
            status: 0u64,
            cap: cap_null_cap_new(),
        };
    } else if cap_type == cap_tag_t::cap_irq_control_cap as u64 {
        return deriveCap_ret_t {
            status: 0u64,
            cap: cap_null_cap_new(),
        };
    } else if cap_type == cap_tag_t::cap_untyped_cap as u64 {
        let status = ensureNoChildren(slot);
        if status != 0u64 {
            return deriveCap_ret_t {
                status: status,
                cap: cap_null_cap_new(),
            };
        } else {
            return deriveCap_ret_t {
                status: status,
                cap: cap,
            };
        }
    } else if cap_type == cap_tag_t::cap_reply_cap as u64 {
        return deriveCap_ret_t {
            status: 0u64,
            cap: cap_null_cap_new(),
        };
    }
    deriveCap_ret_t {
        status: 0u64,
        cap: cap,
    }
}

#[no_mangle]
pub unsafe extern "C" fn finaliseCap(cap: cap_t, final_: bool_t, exposed: bool_t) -> finaliseCap_ret_t {
    if isArchCap(cap) != 0u64 {
        return Arch_finaliseCap(cap, final_);
    }
    let cap_type = cap_get_capType(cap);
    if cap_type == cap_tag_t::cap_endpoint_cap as u64 {
        if final_ != 0u64 {
            cancelAllIPC(cap_endpoint_cap_get_capEPPtr(cap) as *mut endpoint_t)
        }
        return finaliseCap_ret_t {
            remainder: cap_null_cap_new(),
            cleanupInfo: cap_null_cap_new(),
        };
    } else if cap_type == cap_tag_t::cap_notification_cap as u64 {
        if final_ != 0u64 {
            let ntfn = cap_notification_cap_get_capNtfnPtr(cap) as *mut notification_t;
            unbindMaybeNotification(ntfn);
            cancelAllSignals(ntfn);
        }
        return finaliseCap_ret_t {
            remainder: cap_null_cap_new(),
            cleanupInfo: cap_null_cap_new(),
        };
    } else if cap_type == cap_tag_t::cap_reply_cap as u64 ||
        cap_type == cap_tag_t::cap_null_cap as u64 ||
        cap_type == cap_tag_t::cap_domain_cap as u64 {
        return finaliseCap_ret_t {
            remainder: cap_null_cap_new(),
            cleanupInfo: cap_null_cap_new(),
        };
    }
    if exposed != 0u64 {
        panic!("finaliseCap: failed to finalise immediately.");
    }
    if cap_type == cap_tag_t::cap_cnode_cap as u64 {
        if final_ != 0u64 {
            return finaliseCap_ret_t {
                remainder: Zombie_new(
                    1u64 << cap_cnode_cap_get_capCNodeRadix(cap),
                    cap_cnode_cap_get_capCNodeRadix(cap),
                    cap_cnode_cap_get_capCNodePtr(cap)
                ),
                cleanupInfo: cap_null_cap_new(),
            };
        }
    } else if cap_type == cap_tag_t::cap_thread_cap as u64 {
        if final_ != 0u64 {
            let tcb = cap_thread_cap_get_capTCBPtr(cap) as *mut tcb_t;
            //ignore smp
            let cte_ptr = tcb_ptr_cte_ptr(tcb, tcb_cnode_index::tcbCTable as u64);
            unbindNotification(tcb);
            suspend(tcb);
            //debug
            tcbDebugRemove(tcb);
            Arch_prepareThreadDelete(tcb);
            return finaliseCap_ret_t {
                remainder: Zombie_new(
                    tcb_arch_cnode_index::tcbArchCNodeEntries as u64,
                    ZombieType_ZombieTCB,
                    cte_ptr as u64
                ),
                cleanupInfo: cap_null_cap_new(),
            };
        }
    } else if cap_type == cap_tag_t::cap_zombie_cap as u64 {
        return finaliseCap_ret_t {
            remainder: cap,
            cleanupInfo: cap_null_cap_new(),
        };
    } else if cap_type == cap_tag_t::cap_irq_handler_cap as u64 {
        if final_ != 0u64 {
            let irq = cap_irq_handler_cap_get_capIRQ(cap) as u8;
            deletedIRQHandler(irq);
            return finaliseCap_ret_t {
                remainder: cap_null_cap_new(),
                cleanupInfo: cap,
            };
        }
    }
    finaliseCap_ret_t {
        remainder: cap_null_cap_new(),
        cleanupInfo: cap_null_cap_new(),
    }
}

#[no_mangle]
pub extern "C" fn hasCancelSendRights(cap: cap_t) -> bool_t {
    if cap_get_capType(cap) == cap_tag_t::cap_endpoint_cap as u64 {
        return (cap_endpoint_cap_get_capCanSend(cap) != 0u64 &&
            cap_endpoint_cap_get_capCanReceive(cap) != 0u64 &&
            cap_endpoint_cap_get_capCanGrant(cap) != 0u64) as u64;
    }
    0u64
}

macro_rules! MASK {
    ($x:expr) => {
        (1u64<<($x))-1u64
    };
}

#[no_mangle]
pub unsafe extern "C" fn sameRegionAs(cap_a: cap_t, cap_b: cap_t) -> bool_t {
    let cap_type = cap_get_capType(cap_a);
    if cap_type == cap_tag_t::cap_untyped_cap as u64 {
        if cap_get_capIsPhysical(cap_b) != 0u64 {
            let aBase = cap_untyped_cap_get_capPtr(cap_a);
            let bBase = cap_get_capPtr(cap_b);
            let aTop = aBase + MASK!(cap_untyped_cap_get_capBlockSize(cap_a));
            let bTop = bBase + MASK!(cap_get_capSizeBits(cap_b));
            return ((aBase <= bBase) && (bTop <= aTop) && (bBase <= bTop)) as u64;
        }
    } else if cap_type == cap_tag_t::cap_endpoint_cap as u64 {
        if cap_get_capType(cap_b) == cap_tag_t::cap_endpoint_cap as u64 {
            return (cap_endpoint_cap_get_capEPPtr(cap_a) ==
                cap_endpoint_cap_get_capEPPtr(cap_b)) as u64;
        }
    } else if cap_type == cap_tag_t::cap_notification_cap as u64 {
        if cap_get_capType(cap_b) == cap_tag_t::cap_endpoint_cap as u64 {
            return (cap_notification_cap_get_capNtfnPtr(cap_a) ==
                cap_notification_cap_get_capNtfnPtr(cap_b)) as u64;
        }
    } else if cap_type == cap_tag_t::cap_cnode_cap as u64 {
        if cap_get_capType(cap_b) == cap_tag_t::cap_cnode_cap as u64 {
            return ((cap_cnode_cap_get_capCNodePtr(cap_a) ==
                cap_cnode_cap_get_capCNodePtr(cap_b)) &&
                (cap_cnode_cap_get_capCNodeRadix(cap_a) ==
                cap_cnode_cap_get_capCNodeRadix(cap_b))) as u64;
        }
    } else if cap_type == cap_tag_t::cap_thread_cap as u64 {
        if cap_get_capType(cap_b) == cap_tag_t::cap_thread_cap as u64  {
            return (cap_thread_cap_get_capTCBPtr(cap_a) ==
                cap_thread_cap_get_capTCBPtr(cap_b)) as u64;
        }
    } else if cap_type == cap_tag_t::cap_reply_cap as u64 {
        if cap_get_capType(cap_b) == cap_tag_t::cap_reply_cap as u64 {
            return (cap_reply_cap_get_capTCBPtr(cap_a) ==
                cap_reply_cap_get_capTCBPtr(cap_b)) as u64;
        }
    } else if cap_type == cap_tag_t::cap_domain_cap as u64 {
        if cap_get_capType(cap_b) == cap_tag_t::cap_domain_cap as u64 {
            return 1u64;
        }
    } else if cap_type == cap_tag_t::cap_irq_control_cap as u64 {
        if cap_get_capType(cap_b) == cap_tag_t::cap_irq_control_cap as u64 ||
            cap_get_capType(cap_b) == cap_tag_t::cap_irq_handler_cap as u64 {
            return 1u64;
        }
    } else if cap_type == cap_tag_t::cap_irq_handler_cap as u64 {
        if cap_get_capType(cap_b) == cap_tag_t::cap_irq_handler_cap as u64 {
            return ((cap_irq_handler_cap_get_capIRQ(cap_a) as u8)
                == (cap_irq_handler_cap_get_capIRQ(cap_b) as u8)) as u64;
        }
    }
    if isArchCap(cap_a) != 0u64 && isArchCap(cap_b) != 0u64 {
        return Arch_sameRegionAs(cap_a, cap_b);
    }
    0u64
}

#[no_mangle]
pub unsafe extern "C" fn sameObjectAs(cap_a: cap_t, cap_b: cap_t) -> bool_t {
    if cap_get_capType(cap_a) == cap_tag_t::cap_untyped_cap as u64 {
        return 0u64;
    }
    if cap_get_capType(cap_a) == cap_tag_t::cap_irq_control_cap as u64 &&
        cap_get_capType(cap_b) == cap_tag_t::cap_irq_handler_cap as u64 {
        return 0u64;
    }
    if isArchCap(cap_a) != 0u64 && isArchCap(cap_b) != 0u64 {
        return Arch_sameObjectAs(cap_a, cap_b);
    }
    sameRegionAs(cap_a, cap_b)
}

#[no_mangle]
pub unsafe extern "C" fn updateCapData(preserve: bool_t, newData: u64, cap: cap_t) -> cap_t {
    if isArchCap(cap) != 0u64 {
        return Arch_updateCapData(preserve, newData, cap);
    }
    let cap_type = cap_get_capType(cap);
    if cap_type == cap_tag_t::cap_endpoint_cap as u64 {
        if preserve == 0u64 && cap_endpoint_cap_get_capEPBadge(cap) == 0 {
            return cap_endpoint_cap_set_capEPBadge(cap, newData);
        } else {
            return cap_null_cap_new();
        }
    } else if cap_type == cap_tag_t::cap_notification_cap as u64 {
        if preserve == 0u64 && cap_notification_cap_get_capNtfnBadge(cap) == 0 {
            return cap_notification_cap_set_capNtfnBadge(cap, newData);
        } else {
            return cap_null_cap_new();
        }
    } else if cap_type == cap_tag_t::cap_cnode_cap as u64 {
        let w = seL4_CNode_CapData_t {
            words: [newData],
        };
        let guardSize = seL4_CNode_CapData_get_guardSize(w);
        if guardSize + cap_cnode_cap_get_capCNodeRadix(cap) > wordBits {
            return cap_null_cap_new();
        } else {
            let guard = seL4_CNode_CapData_get_guard(w) & MASK!(guardSize);
            let mut new_cap = cap_cnode_cap_set_capCNodeGuard(cap, guard);
            new_cap = cap_cnode_cap_set_capCNodeGuardSize(new_cap, guardSize);
            return new_cap;
        }
    }
    cap
}

#[no_mangle]
pub unsafe extern "C" fn
performInvocation_Endpoint(ep: *mut endpoint_t, badge: u64,
                           canGrant: bool_t, block: bool_t, call: bool_t) ->u64 {
    sendIPC(block, call, badge, canGrant, node_state!(ksCurThread), ep);
    0u64
}

#[no_mangle]
pub unsafe extern "C" fn
performInvocation_Notification(ntfn: *mut notification_t, badge: u64) -> u64 {
    sendSignal(ntfn, badge);
    0u64
}

#[no_mangle]
pub unsafe extern "C" fn
performInvocation_Reply(thread: *mut tcb_t, slot: *mut cte_t) -> u64 {
    doReplyTransfer(node_state!(ksCurThread), thread, slot);
    0u64
}

#[inline]
pub unsafe fn postCapDeletion(cap: cap_t) {
    if cap_get_capType(cap) == cap_tag_t::cap_irq_handler_cap as u64 {
        let irq: u8 = cap_irq_handler_cap_get_capIRQ(cap) as u8;
        deletedIRQHandler(irq);
    } else if isArchCap(cap) != 0u64 {
        Arch_postCapDeletion(cap);
    }
}