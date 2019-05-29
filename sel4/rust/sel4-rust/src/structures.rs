#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(dead_code)]
#![allow(non_upper_case_globals)]
use crate::types;
use crate::types::word_t;
use crate::object::arch_structures;
use crate::object::arch_structures::arch_tcb_t;

//以下对应include/object/structures.h中的内容

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
enum irq_state {
    IRQInactive = 0,
    IRQSignal = 1,
    IRQTimer = 2,
    IRQReserved
}
type irq_state_t=u32;

#[repr(C)]
struct dschedule {
    domain:types::dom_t,
    length:word_t
}

#[repr(C)]
enum endpoint_state {
    EPState_Idle = 0,
    EPState_Send = 1,
    EPState_Recv = 2
}
type endpoint_state_t=word_t;

#[repr(C)]
enum notification_state {
    NtfnState_Idle = 0,
    NtfnState_Waiting = 1,
    NtfnState_Active = 2
}
type notification_state_t=word_t;

//include/object/structures.h中的函数

//zombie相关
macro_rules! MASK {
    ($x:expr) => {
        (1u64<<($x))-1u64
    };
}

const ZombieType_ZombieTCB:u64=1u64<<6;
const TCB_CNODE_RADIX:u64=4;

#[inline]
pub fn Zombie_new(number:word_t, r#type:word_t, ptr:word_t)->cap_t{
    let mask:word_t=
        if r#type==ZombieType_ZombieTCB {
            MASK!(TCB_CNODE_RADIX+1)
            //(1u64<<(TCB_CNODE_RADIX+1))-1u64
        }else{
            MASK!(r#type+1)
            //(1u64<<(r#type+1))-1u64
        };
    arch_structures::cap_zombie_cap_new((ptr&!mask)|(number&mask), r#type)
}


#[inline]
pub fn cap_zombie_cap_get_capZombieBits(cap:cap_t)->word_t{
    let r#type=arch_structures::cap_zombie_cap_get_capZombieType(cap);
    if r#type==ZombieType_ZombieTCB {
        return TCB_CNODE_RADIX;
    }
    r#type&MASK!(6)
}

#[inline]
pub fn cap_zombie_cap_get_capZombieNumber(cap:cap_t)->word_t{
    let radix:word_t=cap_zombie_cap_get_capZombieBits(cap);
    arch_structures::cap_zombie_cap_get_capZombieID(cap) & MASK!(radix+1)
}

#[inline]
pub fn cap_zombie_cap_get_capZombiePtr(cap:cap_t)->word_t{
    let radix:word_t=cap_zombie_cap_get_capZombieBits(cap);
    arch_structures::cap_zombie_cap_get_capZombieID(cap) & ! MASK!(radix+1)
}

#[inline]
fn cap_zombie_cap_set_capZombieNumber(cap:cap_t,n:word_t)->cap_t{
    let radix:word_t=cap_zombie_cap_get_capZombieBits(cap);
    let ptr=arch_structures::cap_zombie_cap_get_capZombieID(cap) & !MASK!(radix+1);
    arch_structures::cap_zombie_cap_set_capZombieID(cap, ptr | (n & MASK!(radix+1) ))
}

//线程相关
#[repr(C)]
enum _thread_state {
    ThreadState_Inactive = 0,
    ThreadState_Running,
    ThreadState_Restart,
    ThreadState_BlockedOnReceive,
    ThreadState_BlockedOnSend,
    ThreadState_BlockedOnReply,
    ThreadState_BlockedOnNotification,
    ThreadState_IdleThreadState
}
type _thread_state_t=word_t;

#[repr(C)]
enum tcb_cnode_index {
    tcbCTable = 0,
    tcbVTable = 1,
    tcbReply = 2,
    tcbCaller = 3,
    tcbBuffer = 4,
    tcbCNodeEntries
}
type tcb_cnode_index_t=word_t;

#[repr(C)]
enum vm_rights {
    VMKernelOnly = 1,
    VMReadOnly = 2,
    VMReadWrite = 3
}
type vm_rights_t=word_t;

//这两个函数宛若智障= =
#[inline]
fn wordFromVMRights(vm_rights:vm_rights_t)->word_t{
    vm_rights
}
#[inline]
fn vmRightsFromWord(w:word_t)->vm_rights_t{
    w
}

#[repr(C)]
struct vm_attributes{
    words:[u64;1]
}
type vm_attributes_t=vm_attributes;

#[inline]
fn vmAttributesFromWord(w:word_t)->vm_attributes_t{
    vm_attributes_t{
        words:[w]
    }
}

//TCB相关
#[repr(C)]
struct thread_state_t{
    words:[u64;3]
}

#[repr(C)]
struct notification_t{
    words:[u64;4]
}

#[repr(C)]
struct seL4_Fault_t{
    words:[u64;2]
}

#[repr(C)]
struct lookup_fault_t{
    words:[u64;2]
}

#[repr(C)]
pub struct tcb_t {
    tcbArch: arch_tcb_t,
    tcbState: thread_state_t,
    tcbBoundNotification: *mut notification_t,
    tcbFault: seL4_Fault_t,
    tcbLookupFailure: lookup_fault_t,
    tcbDomain: types::dom_t,
    tcbMCP: types::prio_t,
    tcbPriority: types::prio_t,
    tcbTimeSlice: word_t,
    tcbFaultHandler: types::cptr_t,
    tcbIPCBuffer: word_t,
    
    tcbSchedNext: *mut tcb_t,
    tcbSchedPrev: *mut tcb_t,
    tcbEPNext: *mut tcb_t,
    tcbEPPrev: *mut tcb_t,
    
    tcbDebugNext: *mut tcb_t,
    tcbDebugPrev: *mut tcb_t,
    tcbName: *mut u8 //C语言中是char tcbName[]，这里直接翻译成指针了
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
    cap_io_port_control_cap = 31
}

const seL4_EndpointBits:u64=4;
const seL4_NotificationBits:u64=5;
const seL4_SlotBits:u64=5;
const seL4_TCBBits:u64=11;

#[inline]
pub fn cap_get_capSizeBits(cap:cap_t)->word_t{
    let ctag=arch_structures::cap_get_capType(cap);
    //rust不允许整数直接转枚举体，所以只能用这种别扭的写法了
    match ctag{
        ctag if ctag==(cap_tag_t::cap_null_cap as u64) => 
            arch_structures::cap_untyped_cap_get_capBlockSize(cap),
        ctag if ctag==(cap_tag_t::cap_endpoint_cap as u64) =>
            seL4_EndpointBits,
        ctag if ctag==(cap_tag_t::cap_notification_cap as u64) =>
            seL4_NotificationBits,
        ctag if ctag==(cap_tag_t::cap_cnode_cap as u64) =>
            arch_structures::cap_cnode_cap_get_capCNodeRadix(cap)+seL4_SlotBits,
        ctag if ctag==(cap_tag_t::cap_thread_cap as u64) =>
            seL4_TCBBits,
        ctag if ctag==(cap_tag_t::cap_zombie_cap as u64) =>{
            let r#type=arch_structures::cap_zombie_cap_get_capZombieType(cap);
            if r#type == ZombieType_ZombieTCB {
                seL4_TCBBits
            } else {
                (r#type & MASK!(6)) + seL4_SlotBits
            }
        }
        ctag if ctag==(cap_tag_t::cap_null_cap as u64) ||
                ctag==(cap_tag_t::cap_domain_cap as u64) ||
                ctag==(cap_tag_t::cap_reply_cap as u64) ||
                ctag==(cap_tag_t::cap_irq_control_cap as u64) ||
                ctag==(cap_tag_t::cap_irq_handler_cap as u64) =>
            0,
        _ => arch_structures::cap_get_archCapSizeBits(cap)
    }
}

#[inline]
pub fn cap_get_capIsPhysical(cap:cap_t)->types::bool_t{
    let ctag=arch_structures::cap_get_capType(cap);
    match ctag{
        ctag if ctag==(cap_tag_t::cap_untyped_cap as u64) ||
                ctag==(cap_tag_t::cap_endpoint_cap as u64) ||
                ctag==(cap_tag_t::cap_notification_cap as u64) ||
                ctag==(cap_tag_t::cap_cnode_cap as u64) ||
                ctag==(cap_tag_t::cap_thread_cap as u64) ||
                ctag==(cap_tag_t::cap_zombie_cap as u64) =>
            types::_bool::r#true as u64,
        ctag if ctag==(cap_tag_t::cap_domain_cap as u64) ||
                ctag==(cap_tag_t::cap_reply_cap as u64) ||
                ctag==(cap_tag_t::cap_irq_control_cap as u64) ||
                ctag==(cap_tag_t::cap_irq_handler_cap as u64) =>
            types::_bool::r#false as u64,
        _ => arch_structures::cap_get_archCapIsPhysical(cap)
    }
}

//cap_get_capPtr由于在rust中没有对应的void*，所以暂时不翻译了
//等真的需要用这个函数的时候，再考虑怎么解决这个问题！

#[inline]
pub fn isCapRevocable(derivedCap:cap_t,srcCap:cap_t)->types::bool_t{
    if arch_structures::isArchCap(derivedCap)!=0 {
        return arch_structures::Arch_isCapRevocable(derivedCap, srcCap);
    }
    let ctag=arch_structures::cap_get_capType(derivedCap);
    match ctag{
        ctag if ctag==(cap_tag_t::cap_endpoint_cap as u64) =>
            ( arch_structures::cap_endpoint_cap_get_capEPBadge(derivedCap) !=
            arch_structures::cap_endpoint_cap_get_capEPBadge(srcCap) ) as u64,
        ctag if ctag==(cap_tag_t::cap_notification_cap as u64) =>
            ( arch_structures::cap_notification_cap_get_capNtfnBadge(derivedCap) !=
            arch_structures::cap_notification_cap_get_capNtfnBadge(srcCap) ) as u64,
        ctag if ctag==(cap_tag_t::cap_irq_handler_cap as u64) =>
            ( arch_structures::cap_get_capType(srcCap) == cap_tag_t::cap_irq_control_cap as u64 ) as u64,
        ctag if ctag==(cap_tag_t::cap_untyped_cap as u64) =>
            types::_bool::r#true as u64,
        _ => types::_bool::r#false as u64
    }
}