#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(dead_code)]
#![allow(non_upper_case_globals)]
use crate::object::arch_structures::*;
use crate::types;
use crate::types::word_t;

//include/object/structures.h

#[repr(C)]
#[derive(Copy, Clone)]
pub struct cap_t {
    pub words: [u64; 2],
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct mdb_node_t {
    pub words: [u64; 2],
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct cte_t {
    pub cap: cap_t,
    pub cteMDBNode: mdb_node_t,
}

#[repr(C)]
pub enum irq_state {
    IRQInactive = 0,
    IRQSignal = 1,
    IRQTimer = 2,
    IRQReserved,
}

#[repr(C)]
pub struct dschedule {
    pub domain: types::dom_t,
    pub length: word_t,
}

#[repr(C)]
pub enum endpoint_state {
    EPState_Idle = 0,
    EPState_Send = 1,
    EPState_Recv = 2,
}
type endpoint_state_t = word_t;

#[repr(C)]
pub enum notification_state {
    NtfnState_Idle = 0,
    NtfnState_Waiting = 1,
    NtfnState_Active = 2,
}
type notification_state_t = word_t;

//include/object/structures.h中的函数

//zombie相关
macro_rules! MASK {
    ($x:expr) => {
        (1u64 << ($x)) - 1u64
    };
}

pub const ZombieType_ZombieTCB: u64 = 1u64 << 6;
pub const TCB_CNODE_RADIX: u64 = 4;

#[inline]
pub fn Zombie_new(number: word_t, r#type: word_t, ptr: word_t) -> cap_t {
    let mask: word_t = if r#type == ZombieType_ZombieTCB {
        MASK!(TCB_CNODE_RADIX + 1)
    //(1u64<<(TCB_CNODE_RADIX+1))-1u64
    } else {
        MASK!(r#type + 1)
        //(1u64<<(r#type+1))-1u64
    };
    cap_zombie_cap_new((ptr & !mask) | (number & mask), r#type)
}

#[inline]
pub fn cap_zombie_cap_get_capZombieBits(cap: cap_t) -> word_t {
    let r#type = cap_zombie_cap_get_capZombieType(cap);
    if r#type == ZombieType_ZombieTCB {
        return TCB_CNODE_RADIX;
    }
    r#type & MASK!(6)
}

#[inline]
pub fn cap_zombie_cap_get_capZombieNumber(cap: cap_t) -> word_t {
    let radix: word_t = cap_zombie_cap_get_capZombieBits(cap);
    cap_zombie_cap_get_capZombieID(cap) & MASK!(radix + 1)
}

#[inline]
pub fn cap_zombie_cap_get_capZombiePtr(cap: cap_t) -> word_t {
    let radix: word_t = cap_zombie_cap_get_capZombieBits(cap);
    cap_zombie_cap_get_capZombieID(cap) & !MASK!(radix + 1)
}

#[inline]
pub fn cap_zombie_cap_set_capZombieNumber(cap: cap_t, n: word_t) -> cap_t {
    let radix: word_t = cap_zombie_cap_get_capZombieBits(cap);
    let ptr = cap_zombie_cap_get_capZombieID(cap) & !MASK!(radix + 1);
    cap_zombie_cap_set_capZombieID(cap, ptr | (n & MASK!(radix + 1)))
}

//线程相关
#[repr(C)]
pub enum _thread_state {
    ThreadState_Inactive = 0,
    ThreadState_Running,
    ThreadState_Restart,
    ThreadState_BlockedOnReceive,
    ThreadState_BlockedOnSend,
    ThreadState_BlockedOnReply,
    ThreadState_BlockedOnNotification,
    ThreadState_IdleThreadState,
}
pub type _thread_state_t = word_t;

#[repr(C)]
pub enum tcb_cnode_index {
    tcbCTable = 0,
    tcbVTable = 1,
    tcbReply = 2,
    tcbCaller = 3,
    tcbBuffer = 4,
    tcbCNodeEntries,
}
type tcb_cnode_index_t = word_t;

#[repr(C)]
enum vm_rights {
    VMKernelOnly = 1,
    VMReadOnly = 2,
    VMReadWrite = 3,
}
type vm_rights_t = word_t;

//这两个函数宛若智障= =
#[inline]
fn wordFromVMRights(vm_rights: vm_rights_t) -> word_t {
    vm_rights
}
#[inline]
fn vmRightsFromWord(w: word_t) -> vm_rights_t {
    w
}

#[repr(C)]
struct vm_attributes {
    words: [u64; 1],
}
type vm_attributes_t = vm_attributes;

#[inline]
fn vmAttributesFromWord(w: word_t) -> vm_attributes_t {
    vm_attributes_t { words: [w] }
}

//TCB相关
#[repr(C)]
#[derive(Copy, Clone)]
pub struct thread_state_t {
    pub words: [u64; 3],
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct notification_t {
    pub words: [u64; 4],
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct lookup_fault_t {
    pub words: [u64; 2],
}

#[repr(C)]
pub struct tcb_t {
    pub tcbArch: arch_tcb_t,
    pub tcbState: thread_state_t,
    pub tcbBoundNotification: *mut notification_t,
    pub tcbFault: seL4_Fault_t,
    pub tcbLookupFailure: lookup_fault_t,
    pub tcbDomain: types::dom_t,
    pub tcbMCP: types::prio_t,
    pub tcbPriority: types::prio_t,
    pub tcbTimeSlice: word_t,
    pub tcbFaultHandler: types::cptr_t,
    pub tcbIPCBuffer: word_t,

    pub tcbSchedNext: *mut tcb_t,
    pub tcbSchedPrev: *mut tcb_t,
    pub tcbEPNext: *mut tcb_t,
    pub tcbEPPrev: *mut tcb_t,

    pub tcbDebugNext: *mut tcb_t,
    pub tcbDebugPrev: *mut tcb_t,
    pub tcbName: *mut u8, //C语言中是char tcbName[]，这里直接翻译成指针了
}

//cap相关
#[repr(C)]
pub enum cap_tag_t {
    cap_null_cap = 0,
    cap_untyped_cap = 2,
    cap_endpoint_cap = 4,
    cap_notification_cap = 6,
    cap_reply_cap = 8,
    cap_cnode_cap = 10,
    cap_thread_cap = 12,
    cap_irq_control_cap = 14,
    cap_irq_handler_cap = 16,
    cap_zombie_cap = 18,
    cap_domain_cap = 20,
    cap_frame_cap = 1,
    cap_page_table_cap = 3,
    cap_page_directory_cap = 5,
    cap_pdpt_cap = 7,
    cap_pml4_cap = 9,
    cap_asid_control_cap = 11,
    cap_asid_pool_cap = 13,
    cap_io_port_cap = 19,
    cap_io_port_control_cap = 31,
}

pub const seL4_EndpointBits: u64 = 4;
pub const seL4_NotificationBits: u64 = 5;
pub const seL4_SlotBits: u64 = 5;
pub const seL4_TCBBits: u64 = 11;

#[inline]
pub fn cap_get_capSizeBits(cap: cap_t) -> word_t {
    let ctag = cap_get_capType(cap);
    //rust不允许整数直接转枚举体，所以只能用这种别扭的写法了
    match ctag {
        ctag if ctag == (cap_tag_t::cap_null_cap as u64) => cap_untyped_cap_get_capBlockSize(cap),
        ctag if ctag == (cap_tag_t::cap_endpoint_cap as u64) => seL4_EndpointBits,
        ctag if ctag == (cap_tag_t::cap_notification_cap as u64) => seL4_NotificationBits,
        ctag if ctag == (cap_tag_t::cap_cnode_cap as u64) => {
            cap_cnode_cap_get_capCNodeRadix(cap) + seL4_SlotBits
        }
        ctag if ctag == (cap_tag_t::cap_thread_cap as u64) => seL4_TCBBits,
        ctag if ctag == (cap_tag_t::cap_zombie_cap as u64) => {
            let r#type = cap_zombie_cap_get_capZombieType(cap);
            if r#type == ZombieType_ZombieTCB {
                seL4_TCBBits
            } else {
                (r#type & MASK!(6)) + seL4_SlotBits
            }
        }
        ctag if ctag == (cap_tag_t::cap_null_cap as u64)
            || ctag == (cap_tag_t::cap_domain_cap as u64)
            || ctag == (cap_tag_t::cap_reply_cap as u64)
            || ctag == (cap_tag_t::cap_irq_control_cap as u64)
            || ctag == (cap_tag_t::cap_irq_handler_cap as u64) =>
        {
            0
        }
        _ => cap_get_archCapSizeBits(cap),
    }
}

#[inline]
pub fn cap_get_capIsPhysical(cap: cap_t) -> types::bool_t {
    let ctag = cap_get_capType(cap);
    match ctag {
        ctag if ctag == (cap_tag_t::cap_untyped_cap as u64)
            || ctag == (cap_tag_t::cap_endpoint_cap as u64)
            || ctag == (cap_tag_t::cap_notification_cap as u64)
            || ctag == (cap_tag_t::cap_cnode_cap as u64)
            || ctag == (cap_tag_t::cap_thread_cap as u64)
            || ctag == (cap_tag_t::cap_zombie_cap as u64) =>
        {
            types::_bool::r#true as u64
        }
        ctag if ctag == (cap_tag_t::cap_domain_cap as u64)
            || ctag == (cap_tag_t::cap_reply_cap as u64)
            || ctag == (cap_tag_t::cap_irq_control_cap as u64)
            || ctag == (cap_tag_t::cap_irq_handler_cap as u64) =>
        {
            types::_bool::r#false as u64
        }
        _ => cap_get_archCapIsPhysical(cap),
    }
}

#[inline]
pub unsafe fn cap_get_capPtr(cap: cap_t) -> u64 {
    let ctag = cap_get_capType(cap);
    if ctag == cap_tag_t::cap_untyped_cap as u64 {
        return cap_untyped_cap_get_capPtr(cap);
    } else if ctag == cap_tag_t::cap_endpoint_cap as u64 {
        return cap_endpoint_cap_get_capEPPtr(cap);
    } else if ctag == cap_tag_t::cap_notification_cap as u64 {
        return cap_notification_cap_get_capNtfnPtr(cap);
    } else if ctag == cap_tag_t::cap_cnode_cap as u64 {
        return cap_cnode_cap_get_capCNodePtr(cap);
    } else if ctag == cap_tag_t::cap_thread_cap as u64 {
        return tcb_ptr_cte_ptr(cap_thread_cap_get_capTCBPtr(cap) as *mut tcb_t, 0) as u64;
    } else if ctag == cap_tag_t::cap_zombie_cap as u64 {
        return cap_zombie_cap_get_capZombiePtr(cap);
    } else if ctag == cap_tag_t::cap_domain_cap as u64
        || ctag == cap_tag_t::cap_reply_cap as u64
        || ctag == cap_tag_t::cap_irq_control_cap as u64
        || ctag == cap_tag_t::cap_irq_handler_cap as u64
    {
        return 0u64;
    }
    cap_get_archCapPtr(cap)
}

#[inline]
pub fn isCapRevocable(derivedCap: cap_t, srcCap: cap_t) -> types::bool_t {
    if isArchCap(derivedCap) != 0 {
        return Arch_isCapRevocable(derivedCap, srcCap);
    }
    let ctag = cap_get_capType(derivedCap);
    match ctag {
        ctag if ctag == (cap_tag_t::cap_endpoint_cap as u64) => {
            (cap_endpoint_cap_get_capEPBadge(derivedCap) != cap_endpoint_cap_get_capEPBadge(srcCap))
                as u64
        }
        ctag if ctag == (cap_tag_t::cap_notification_cap as u64) => {
            (cap_notification_cap_get_capNtfnBadge(derivedCap)
                != cap_notification_cap_get_capNtfnBadge(srcCap)) as u64
        }
        ctag if ctag == (cap_tag_t::cap_irq_handler_cap as u64) => {
            (cap_get_capType(srcCap) == cap_tag_t::cap_irq_control_cap as u64) as u64
        }
        ctag if ctag == (cap_tag_t::cap_untyped_cap as u64) => types::_bool::r#true as u64,
        _ => types::_bool::r#false as u64,
    }
}

#[inline]
pub unsafe fn tcb_ptr_cte_ptr(p: *mut tcb_t, i: u64) -> *mut cte_t {
    (((p as u64) & (!MASK!(seL4_TCBBits))) as *mut cte_t).offset(i as isize)
}

// include/object/tcb.h 因为不想翻译tcb.h整个文件所以就放这里了
#[repr(C)]
#[derive(Copy, Clone)]
pub struct tcb_queue {
    pub head: *mut tcb_t,
    pub end: *mut tcb_t,
}
pub type tcb_queue_t = tcb_queue;
