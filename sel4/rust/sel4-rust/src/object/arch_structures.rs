#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(dead_code)]
#![allow(non_upper_case_globals)]

use crate::structures::cap_t;
use crate::structures::cap_tag_t;
use crate::structures::lookup_fault_t;
use crate::structures::mdb_node_t;
use crate::structures::notification_t;
use crate::structures::tcb_cnode_index;
use crate::structures::thread_state_t;
use crate::types;
use crate::types::bool_t;
use crate::types::word_t;

//generated/arch/object/structures_gen.h
#[repr(C)]
pub struct pde {
    words: [u64; 1],
}
pub type pde_t = pde;

#[repr(C)]
pub struct pte {
    words: [u64; 1],
}
pub type pte_t = pte;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct x86_irq_state_t {
    words: [u32; 2],
}

#[inline]
pub fn x86_irq_state_get_irqType(x86_irq_state: x86_irq_state_t) -> u32 {
    (x86_irq_state.words[1] >> 28) & 0xfu32
}

#[inline]
pub fn x86_irq_state_irq_ioapic_get_id(x86_irq_state: x86_irq_state_t) -> u32 {
    (x86_irq_state.words[1] & 0xf800000u32) >> 23
}

#[inline]
pub fn x86_irq_state_irq_ioapic_get_pin(x86_irq_state: x86_irq_state_t) -> u32 {
    (x86_irq_state.words[1] & 0x7c0000u32) >> 18
}

#[inline]
pub fn x86_irq_state_irq_ioapic_set_masked(
    mut x86_irq_state: x86_irq_state_t,
    v32: u32,
) -> x86_irq_state_t {
    x86_irq_state.words[1] &= !0x8000u32;
    x86_irq_state.words[1] |= (v32 << 15) & 0x8000u32;
    x86_irq_state
}

const cap_zombie_cap: u64 = 18;
#[inline]
pub fn cap_zombie_cap_new(capZombieID: u64, capZombieType: u64) -> cap_t {
    cap_t {
        words: [
            0 | (cap_zombie_cap & 31u64) << 59 | (capZombieType & 127u64) << 0,
            0 | capZombieID << 0,
        ],
    }
}

#[inline]
pub fn cap_zombie_cap_get_capZombieType(cap: cap_t) -> u64 {
    cap.words[0] & 127u64
}

#[inline]
pub fn cap_zombie_cap_get_capZombieID(cap: cap_t) -> u64 {
    cap.words[1] & 0xffffffffffffffffu64
}

#[inline]
pub fn cap_zombie_cap_set_capZombieID(mut cap: cap_t, v64: u64) -> cap_t {
    cap.words[1] &= !0xffffffffffffffffu64;
    cap.words[1] |= v64 & 0xffffffffffffffffu64;
    cap
}

#[inline]
pub fn cap_frame_cap_get_capFSize(cap: cap_t) -> u64 {
    (cap.words[0] >> 59) & 0x1fu64
}

#[inline]
pub fn cap_get_capType(cap: cap_t) -> u64 {
    (cap.words[0] >> 59) & 0x1fu64
}

#[inline]
pub fn cap_get_archCapPtr(cap: cap_t) -> u64 {
    let ctag = cap_get_capType(cap);
    if ctag == cap_tag_t::cap_frame_cap as u64 {
        return cap_frame_cap_get_capFBasePtr(cap);
    } else if ctag == cap_tag_t::cap_page_table_cap as u64 {
        return cap_page_table_cap_get_capPTBasePtr(cap);
    } else if ctag == cap_tag_t::cap_page_directory_cap as u64 {
        return cap_page_directory_cap_get_capPDBasePtr(cap);
    } else if ctag == cap_tag_t::cap_io_port_cap as u64
        || ctag == cap_tag_t::cap_asid_control_cap as u64
    {
        return 0u64;
    } else if ctag == cap_tag_t::cap_asid_pool_cap as u64 {
        return cap_asid_pool_cap_get_capASIDPool(cap);
    }
    cap_get_modeCapPtr(cap)
}

#[inline]
pub fn cap_frame_cap_get_capFBasePtr(cap: cap_t) -> u64 {
    let mut ret = cap.words[1] & 0xffffffffffffu64;
    if (ret & (1u64 << 47)) != 0u64 {
        ret |= 0xffff000000000000u64;
    }
    ret
}

#[inline]
pub fn cap_page_table_cap_get_capPTBasePtr(cap: cap_t) -> u64 {
    let mut ret = cap.words[1] & 0xffffffffffffu64;
    if (ret & (1u64 << 47)) != 0u64 {
        ret |= 0xffff000000000000u64;
    }
    ret
}

#[inline]
pub fn cap_page_directory_cap_get_capPDBasePtr(cap: cap_t) -> u64 {
    let mut ret = cap.words[1] & 0xffffffffffffu64;
    if (ret & (1u64 << 47)) != 0u64 {
        ret |= 0xffff000000000000u64;
    }
    ret
}

#[inline]
pub fn cap_asid_pool_cap_get_capASIDPool(cap: cap_t) -> u64 {
    let mut ret = (cap.words[0] & 0x1fffffffffu64) << 11;
    if (ret & (1u64 << 47)) != 0u64 {
        ret |= 0xffff000000000000u64;
    }
    ret
}

#[inline]
pub fn cap_get_modeCapPtr(cap: cap_t) -> u64 {
    let ctag = cap_get_capType(cap);
    if ctag == cap_tag_t::cap_pml4_cap as u64 {
        return cap_pml4_cap_get_capPML4BasePtr(cap);
    } else if ctag == cap_tag_t::cap_pdpt_cap as u64 {
        return cap_pdpt_cap_get_capPDPTBasePtr(cap);
    }
    0u64
}

#[inline]
pub fn cap_pml4_cap_get_capPML4BasePtr(cap: cap_t) -> u64 {
    cap.words[1] & 0xffffffffffffffffu64
}

#[inline]
pub fn cap_pdpt_cap_get_capPDPTBasePtr(cap: cap_t) -> u64 {
    let mut ret = cap.words[1] & 0xffffffffffffu64;
    if (ret & (1u64 << 47)) != 0u64 {
        ret |= 0xffff000000000000u64;
    }
    ret
}

#[inline]
pub fn cap_endpoint_cap_get_capEPPtr(cap: cap_t) -> u64 {
    let mut ret: u64 = cap.words[0] & 0xffffffffffffu64;
    if (ret & (1u64 << 47)) != 0 {
        ret |= 0xffff000000000000;
    }
    ret
}

#[inline]
pub fn isArchCap(cap: cap_t) -> word_t {
    cap_get_capType(cap) % 2
}

#[inline]
pub fn cap_cnode_cap_get_capCNodeRadix(cap: cap_t) -> u64 {
    (cap.words[0] & 0x1f800000000000u64) >> 47
}

#[inline]
pub fn cap_cnode_cap_get_capCNodeGuard(cap: cap_t) -> u64 {
    cap.words[1] & 0xffffffffffffffffu64
}

#[inline]
pub fn cap_cnode_cap_set_capCNodeGuard(mut cap: cap_t, v64: u64) -> cap_t {
    cap.words[1] &= !0xffffffffffffffffu64;
    cap.words[1] |= v64 & 0xffffffffffffffffu64;
    cap
}

#[inline]
pub fn cap_cnode_cap_get_capCNodeGuardSize(cap: cap_t) -> u64 {
    (cap.words[0] & 0x7e0000000000000u64) >> 53
}

#[inline]
pub fn cap_cnode_cap_set_capCNodeGuardSize(mut cap: cap_t, v64: u64) -> cap_t {
    cap.words[0] &= !0x7e0000000000000u64;
    cap.words[0] |= (v64 << 53) & 0x7e0000000000000u64;
    cap
}

#[inline]
pub fn cap_endpoint_cap_get_capEPBadge(cap: cap_t) -> u64 {
    cap.words[1] & 0xffffffffffffffffu64
}

#[inline]
pub fn cap_endpoint_cap_set_capEPBadge(mut cap: cap_t, v64: u64) -> cap_t {
    cap.words[1] &= !0xffffffffffffffffu64;
    cap.words[1] |= v64 & 0xffffffffffffffffu64;
    cap
}

#[inline]
pub fn cap_endpoint_cap_set_capCanSend(mut cap: cap_t, v64: u64) -> cap_t {
    cap.words[0] &= !0x100000000000000u64;
    cap.words[0] |= (v64 << 56) & 0x100000000000000u64;
    cap
}

#[inline]
pub fn cap_endpoint_cap_set_capCanReceive(mut cap: cap_t, v64: u64) -> cap_t {
    cap.words[0] &= !0x200000000000000u64;
    cap.words[0] |= (v64 << 57) & 0x200000000000000u64;
    cap
}

#[inline]
pub fn cap_endpoint_cap_set_capCanGrant(mut cap: cap_t, v64: u64) -> cap_t {
    cap.words[0] &= !0x400000000000000u64;
    cap.words[0] |= (v64 << 58) & 0x400000000000000u64;
    cap
}

#[inline]
pub fn cap_notification_cap_get_capNtfnBadge(cap: cap_t) -> u64 {
    cap.words[1] & 0xffffffffffffffffu64
}

#[inline]
pub fn cap_notification_cap_set_capNtfnBadge(mut cap: cap_t, v64: u64) -> cap_t {
    cap.words[1] &= !0xffffffffffffffffu64;
    cap.words[1] |= v64 & 0xffffffffffffffffu64;
    cap
}

#[inline]
pub fn cap_notification_cap_get_capNtfnCanSend(cap: cap_t) -> u64 {
    (cap.words[0] & 0x200000000000000u64) >> 57
}

#[inline]
pub fn cap_notification_cap_set_capNtfnCanSend(mut cap: cap_t, v64: u64) -> cap_t {
    cap.words[0] &= !0x200000000000000u64;
    cap.words[0] |= (v64 << 57) & 0x200000000000000u64;
    cap
}

#[inline]
pub fn cap_notification_cap_get_capNtfnCanReceive(cap: cap_t) -> u64 {
    (cap.words[0] & 0x400000000000000u64) >> 58
}

#[inline]
pub fn cap_notification_cap_set_capNtfnCanReceive(mut cap: cap_t, v64: u64) -> cap_t {
    cap.words[0] &= !0x400000000000000u64;
    cap.words[0] |= (v64 << 58) & 0x400000000000000u64;
    cap
}

#[inline]
pub fn cap_untyped_cap_get_capPtr(cap: cap_t) -> u64 {
    let mut ret = cap.words[0] & 0xffffffffffffu64;
    if ret & (1u64 << 47) != 0 {
        ret |= 0xffff000000000000u64;
    }
    ret
}

#[inline]
pub fn cap_untyped_cap_set_capFreeIndex(mut cap: cap_t, v64: u64) -> cap_t {
    cap.words[1] &= !0xffffffffffff0000u64;
    cap.words[1] |= (v64 << 16) & 0xffffffffffff0000u64;
    cap
}

#[inline]
pub fn cap_untyped_cap_get_capFreeIndex(cap: cap_t) -> u64 {
    (cap.words[1] & 0xffffffffffff0000u64) >> 16
}

#[inline]
pub fn cap_untyped_cap_get_capBlockSize(cap: cap_t) -> u64 {
    cap.words[1] & 0x3fu64
}

#[inline]
pub fn cap_untyped_cap_get_capIsDevice(cap: cap_t) -> u64 {
    (cap.words[1] & 0x40u64) >> 6
}

#[inline]
pub fn cap_untyped_cap_ptr_set_capFreeIndex(cap_ptr: &mut cap_t, v64: u64) {
    cap_ptr.words[1] &= !0xffffffffffff0000u64;
    cap_ptr.words[1] |= (v64 << 16) & 0xffffffffffff0000u64;
}

#[inline]
pub fn cap_reply_cap_get_capReplyMaster(cap: cap_t) -> u64 {
    cap.words[0] & 0x1u64
}

#[inline]
pub fn cap_reply_cap_new(capReplyMaster: u64, capTCBPtr: u64) -> cap_t {
    cap_t {
        words: [
            (capReplyMaster & 0x1u64) | (((cap_tag_t::cap_reply_cap as u64) & 0x1fu64) << 59),
            capTCBPtr,
        ],
    }
}

#[inline]
pub fn cap_reply_cap_get_capTCBPtr(cap: cap_t) -> u64 {
    cap.words[1] & 0xffffffffffffffffu64
}

#[inline]
pub fn cap_cnode_cap_get_capCNodePtr(cap: cap_t) -> u64 {
    let mut ret = (cap.words[0] & 0x7fffffffffffu64) << 1;
    if ret & (1u64 << (47)) != 0u64 {
        ret |= 0xffff000000000000u64;
    }
    ret
}

#[inline]
pub fn cap_notification_cap_get_capNtfnPtr(cap: cap_t) -> u64 {
    let mut ret = cap.words[0] & 0xffffffffffffu64;
    if (ret & (1u64 << 47)) != 0u64 {
        ret |= 0xffff000000000000u64;
    }
    ret
}

#[inline]
pub fn cap_thread_cap_new(capTCBPtr: u64) -> cap_t {
    cap_t {
        words: [
            ((cap_tag_t::cap_thread_cap as u64 & 0x1fu64) << 59) | (capTCBPtr & 0xffffffffffffu64),
            0,
        ],
    }
}

#[inline]
pub fn cap_thread_cap_get_capTCBPtr(cap: cap_t) -> u64 {
    let mut ret = cap.words[0] & 0xffffffffffffu64;
    if (ret & (1u64 << 47)) != 0u64 {
        ret |= 0xffff000000000000u64;
    }
    ret
}

#[inline]
pub fn cap_endpoint_cap_get_capCanSend(cap: cap_t) -> u64 {
    (cap.words[0] & 0x100000000000000u64) >> 56
}

#[inline]
pub fn cap_endpoint_cap_get_capCanReceive(cap: cap_t) -> u64 {
    (cap.words[0] & 0x200000000000000u64) >> 57
}

#[inline]
pub fn cap_endpoint_cap_get_capCanGrant(cap: cap_t) -> u64 {
    (cap.words[0] & 0x400000000000000u64) >> 58
}

#[inline]
pub fn cap_irq_handler_cap_get_capIRQ(cap: cap_t) -> u64 {
    cap.words[1] & 0xffu64
}

#[inline]
pub fn cap_irq_handler_cap_new(capIRQ: u64) -> cap_t {
    cap_t {
        words: [
            ((cap_tag_t::cap_irq_handler_cap as u64) & 0x1fu64) << 59,
            (capIRQ & 0xffu64),
        ],
    }
}

#[inline]
pub fn thread_state_get_tsType(thread_state: &thread_state_t) -> u64 {
    thread_state.words[0] & 0xfu64
}

#[inline]
pub fn thread_state_ptr_set_tsType(thread_state_ptr: &mut thread_state_t, v64: u64) {
    thread_state_ptr.words[0] &= !0xfu64;
    thread_state_ptr.words[0] |= v64 & 0xfu64;
}

#[inline]
pub fn thread_state_ptr_get_blockingObject(thread_state_ptr:&mut thread_state_t)->u64{
    let mut ret:u64=thread_state_ptr.words[0] & 0xfffffffffff0u64;
    if (ret & (1u64<<47)) !=0{
        ret |= 0xffff000000000000;
    }
    ret
}

#[inline]
pub fn thread_state_ptr_set_blockingObject(thread_state_ptr: &mut thread_state_t, v64: u64) {
    thread_state_ptr.words[0] &= !0xfffffffffff0u64;
    thread_state_ptr.words[0] |= v64 & 0xfffffffffff0u64;
}

#[inline]
pub fn thread_state_get_tcbQueued(thread_state: thread_state_t) -> u64 {
    thread_state.words[1] & 0x1u64
}

#[inline]
pub fn thread_state_ptr_set_tcbQueued(thread_state_ptr: &mut thread_state_t, v64: u64) {
    thread_state_ptr.words[1] &= !0x1u64;
    thread_state_ptr.words[1] |= v64 & 0x1u64;
}

#[inline]
pub fn thread_state_ptr_get_blockingIPCBadge(thread_state_ptr:&thread_state_t)->u64{
    thread_state_ptr.words[2] & 0xffffffffffffffffu64
}

#[inline]
pub fn thread_state_ptr_set_blockingIPCBadge(thread_state_ptr: &mut thread_state_t, v64: u64) {
    thread_state_ptr.words[2] &= !0xffffffffffffffffu64;
    thread_state_ptr.words[2] |= v64 & 0xffffffffffffffff;
}

#[inline]
pub fn thread_state_ptr_get_blockingIPCCanGrant(thread_state_ptr:&thread_state_t)->u64{
    (thread_state_ptr.words[1] & 0x8u64) >>3
}

#[inline]
pub fn thread_state_ptr_set_blockingIPCCanGrant(thread_state_ptr: &mut thread_state_t, v64: u64) {
    thread_state_ptr.words[1] &= !0x8u64;
    thread_state_ptr.words[1] |= (v64 << 3) & 0x8;
}

#[inline]
pub fn thread_state_ptr_get_blockingIPCIsCall(thread_state_ptr:&thread_state_t)->u64{
    (thread_state_ptr.words[1] & 0x4u64) >> 2
}

#[inline]
pub fn thread_state_ptr_set_blockingIPCIsCall(thread_state_ptr: &mut thread_state_t, v64: u64) {
    thread_state_ptr.words[1] &= !0x4u64;
    thread_state_ptr.words[1] |= (v64 << 2) & 0x4;
}

#[inline]
pub fn endpoint_ptr_set_state(endpoint_ptr: &mut endpoint_t, v64: u64) {
    endpoint_ptr.words[0] &= !0x3u64;
    endpoint_ptr.words[0] |= v64 & 0x3;
}

#[repr(C)]
pub struct endpoint {
    words: [u64; 2],
}
pub type endpoint_t = endpoint;

#[inline]
pub fn endpoint_ptr_get_epQueue_head(endpoint_ptr: &endpoint_t) -> u64 {
    endpoint_ptr.words[1] & 0xffffffffffffffffu64
}

#[inline]
pub fn endpoint_ptr_get_epQueue_tail(endpoint_ptr: &endpoint_t) -> u64 {
    let mut ret: u64;
    ret = endpoint_ptr.words[0] & 0xfffffffffffcu64;
    if (ret & (1u64 << 47)) != 0 {
        ret |= 0xffff000000000000;
    }
    ret
}

#[inline]
pub fn endpoint_ptr_set_epQueue_head(endpoint_ptr: &mut endpoint_t, v64: u64) {
    endpoint_ptr.words[1] &= !0xffffffffffffffffu64;
    endpoint_ptr.words[1] |= v64 & 0xffffffffffffffff;
}

#[inline]
pub fn endpoint_ptr_set_epQueue_tail(endpoint_ptr: &mut endpoint_t, v64: u64) {
    endpoint_ptr.words[0] &= !0xfffffffffffcu64;
    endpoint_ptr.words[0] |= v64 & 0xfffffffffffc;
}

#[inline]
pub fn endpoint_ptr_get_state(endpoint_ptr: &endpoint_t) -> u64 {
    endpoint_ptr.words[0] & 0x3u64
}

#[repr(C)]
pub struct seL4_Fault {
    words: [u64; 2],
}
pub type seL4_Fault_t = seL4_Fault;

pub enum seL4_Fault_tag_t {
    seL4_Fault_NullFault = 0,
    seL4_Fault_CapFault = 1,
    seL4_Fault_UnknownSyscall = 2,
    seL4_Fault_UserException = 3,
    seL4_Fault_VMFault = 5,
}

#[inline]
pub fn seL4_Fault_get_seL4_FaultType(seL4_Fault: &seL4_Fault_t) -> u64 {
    seL4_Fault.words[0] & 0x7u64
}

#[inline]
pub fn seL4_Fault_NullFault_new() -> seL4_Fault_t {
    seL4_Fault_t { words: [0, 0] }
}

#[allow(unused_variables)]
#[inline]
pub fn seL4_Fault_CapFault_new(address: u64, inReceivePhase: u64) -> seL4_Fault_t {
    seL4_Fault_t {
        words: [
            ((inReceivePhase & 0x1u64) << 63)
                | ((seL4_Fault_tag_t::seL4_Fault_CapFault as u64) & 0x7u64),
            0,
        ],
    }
}

#[inline]
pub fn seL4_Fault_ptr_get_seL4_FaultType(seL4_Fault: &seL4_Fault_t) -> u64 {
    seL4_Fault.words[0] & 0x7u64
}

//include/arch/x86/arch/machine/registerset.h
const CONFIG_XSAVE_SIZE: usize = 512;
#[repr(C)]
struct user_fpu_state_t {
    state: [u8; CONFIG_XSAVE_SIZE],
}

const n_contextRegisters: usize = 23;
#[repr(C)]
pub struct user_context_t {
    fpuState: user_fpu_state_t,
    pub registers: [word_t; n_contextRegisters],
}

//include/arch/x86/arch/machine/hardware.h
#[repr(C)]
enum vm_page_size {
    X86_SmallPage,
    X86_LargePage,
    X64_HugePage,
}
type vm_page_size_t = word_t;

const seL4_PageBits: u64 = 12;
const seL4_LargePageBits: u64 = 21;
const seL4_HugePageBits: u64 = 30;

#[inline]
fn pageBitsForSize(pagesize: vm_page_size_t) -> word_t {
    match pagesize {
        pagesize if pagesize == (vm_page_size::X86_SmallPage as u64) => seL4_PageBits,
        pagesize if pagesize == (vm_page_size::X86_LargePage as u64) => seL4_LargePageBits,
        pagesize if pagesize == (vm_page_size::X64_HugePage as u64) => seL4_HugePageBits,
        _ => panic!("Invalid page size"), //原来是fail，这里改成panic
    }
}

//include/arch/x86/arch/64/mode/object/structures.h
const seL4_PML4Bits: u64 = 12;
const seL4_PDPTBits: u64 = 12;

#[inline]
fn cap_get_modeCapSizeBits(cap: cap_t) -> word_t {
    let ctag = cap_get_capType(cap);
    match ctag {
        ctag if ctag == (cap_tag_t::cap_pml4_cap as u64) => seL4_PML4Bits,
        ctag if ctag == (cap_tag_t::cap_pdpt_cap as u64) => seL4_PDPTBits,
        _ => 0,
    }
}

#[inline]
fn cap_get_modeCapIsPhysical(cap: cap_t) -> bool_t {
    let ctag = cap_get_capType(cap);
    match ctag {
        ctag if ctag == (cap_tag_t::cap_pml4_cap as u64)
            || ctag == (cap_tag_t::cap_pdpt_cap as u64) =>
        {
            types::_bool::r#true as u64
        }
        _ => types::_bool::r#false as u64,
    }
}

//include/arch/x86/arch/object/structures.h
#[repr(C)]
pub struct arch_tcb_t {
    pub tcbContext: user_context_t,
}

pub enum tcb_arch_cnode_index {
    tcbArchCNodeEntries = tcb_cnode_index::tcbCNodeEntries as isize,
}

const seL4_PageTableBits: u64 = 12;
const seL4_PageDirBits: u64 = 12;
const seL4_ASIDPoolBits: u64 = 12;

#[inline]
pub fn cap_get_archCapSizeBits(cap: cap_t) -> word_t {
    let ctag = cap_get_capType(cap);
    match ctag {
        ctag if ctag == (cap_tag_t::cap_frame_cap as u64) => {
            pageBitsForSize(cap_frame_cap_get_capFSize(cap))
        }
        ctag if ctag == (cap_tag_t::cap_page_table_cap as u64) => seL4_PageTableBits,
        ctag if ctag == (cap_tag_t::cap_page_directory_cap as u64) => seL4_PageDirBits,
        ctag if ctag == (cap_tag_t::cap_io_port_cap as u64) => 0,
        ctag if ctag == (cap_tag_t::cap_asid_control_cap as u64) => 0,
        ctag if ctag == (cap_tag_t::cap_asid_pool_cap as u64) => seL4_ASIDPoolBits,
        _ => cap_get_modeCapSizeBits(cap),
    }
}

#[inline]
pub fn cap_get_archCapIsPhysical(cap: cap_t) -> bool_t {
    let ctag = cap_get_capType(cap);
    match ctag {
        ctag if ctag == (cap_tag_t::cap_frame_cap as u64)
            || ctag == (cap_tag_t::cap_page_table_cap as u64)
            || ctag == (cap_tag_t::cap_page_directory_cap as u64)
            || ctag == (cap_tag_t::cap_asid_pool_cap as u64) =>
        {
            types::_bool::r#true as u64
        }
        ctag if ctag == (cap_tag_t::cap_io_port_cap as u64)
            || ctag == (cap_tag_t::cap_asid_control_cap as u64) =>
        {
            types::_bool::r#false as u64
        }
        _ => cap_get_modeCapIsPhysical(cap),
    }
}

#[inline]
pub fn cap_null_cap_new() -> cap_t {
    cap_t { words: [0; 2] }
}

#[inline]
pub fn Arch_isCapRevocable(derivedCap: cap_t, srcCap: cap_t) -> bool_t {
    if cap_get_capType(derivedCap) == cap_tag_t::cap_io_port_cap as u64 {
        (cap_get_capType(srcCap) == cap_tag_t::cap_io_port_control_cap as u64) as u64
    } else {
        types::_bool::r#false as u64
    }
}

#[inline]
pub fn mdb_node_set_mdbPrev(mut mdb_node: mdb_node_t, v64: u64) -> mdb_node_t {
    mdb_node.words[0] &= !0xffffffffffffffffu64;
    mdb_node.words[0] |= (v64 << 0) & 0xffffffffffffffffu64;
    mdb_node
}

#[inline]
pub fn mdb_node_get_mdbRevocable(mdb_node: mdb_node_t) -> u64 {
    (mdb_node.words[1] & 0x2u64) >> 1
}

#[inline]
pub fn mdb_node_set_mdbRevocable(mut mdb_node: mdb_node_t, v64: u64) -> mdb_node_t {
    mdb_node.words[1] &= !0x2u64;
    mdb_node.words[1] |= (v64 << 1) & 0x2u64;
    mdb_node
}

#[inline]
pub fn mdb_node_set_mdbFirstBadged(mut mdb_node: mdb_node_t, v64: u64) -> mdb_node_t {
    mdb_node.words[1] &= !0x1u64;
    mdb_node.words[1] |= v64 & 0x1u64;
    mdb_node
}

#[inline]
pub fn mdb_node_get_mdbNext(mdb_node: mdb_node_t) -> u64 {
    let mut ret = mdb_node.words[1] & 0xfffffffffffcu64;
    if (ret & (1u64 << 47)) != 0 {
        ret |= 0xffff000000000000u64;
    }
    ret
}

#[inline]
pub fn mdb_node_get_mdbFirstBadged(mdb_node: mdb_node_t) -> u64 {
    mdb_node.words[1] & 0x1u64
}

#[inline]
pub fn mdb_node_get_mdbPrev(mdb_node: mdb_node_t) -> u64 {
    mdb_node.words[0] & 0xffffffffffffffffu64
}

#[inline]
pub fn mdb_node_ptr_set_mdbNext(mdb_node_ptr: &mut mdb_node_t, v64: u64) {
    mdb_node_ptr.words[1] &= !0xfffffffffffcu64;
    mdb_node_ptr.words[1] |= v64 & 0xfffffffffffcu64;
}

#[inline]
pub fn mdb_node_ptr_set_mdbPrev(mdb_node_ptr: &mut mdb_node_t, v64: u64) {
    mdb_node_ptr.words[0] &= !0xffffffffffffffffu64;
    mdb_node_ptr.words[0] |= v64 & 0xffffffffffffffffu64;
}

#[inline]
pub fn mdb_node_ptr_set_mdbFirstBadged(mdb_node_ptr: &mut mdb_node_t, v64: u64) {
    mdb_node_ptr.words[1] &= !0x1u64;
    mdb_node_ptr.words[1] |= v64 & 0x1u64;
}

#[inline]
pub fn mdb_node_new(
    mdbNext: u64,
    mdbRevocable: u64,
    mdbFirstBadged: u64,
    mdbPrev: u64,
) -> mdb_node_t {
    mdb_node_t {
        words: [
            mdbPrev,
            (mdbNext & 0xfffffffffffcu64)
                | ((mdbRevocable & 0x1u64) << 1)
                | (mdbFirstBadged & 0x1u64),
        ],
    }
}

#[inline]
pub fn mdb_node_ptr_set_mdbRevocable(mdb_node_ptr: &mut mdb_node_t, v64: u64) {
    mdb_node_ptr.words[1] &= !0x2u64;
    mdb_node_ptr.words[1] |= (v64 << 1) & 0x2u64;
}

pub enum lookup_fault_tag_t {
    lookup_fault_invalid_root = 0,
    lookup_fault_missing_capability = 1,
    lookup_fault_depth_mismatch = 2,
    lookup_fault_guard_mismatch = 3,
}

#[inline]
pub fn lookup_fault_invalid_root_new() -> lookup_fault_t {
    lookup_fault_t { words: [0; 2] }
}

#[inline]
pub fn lookup_fault_get_lufType(lookup_fault: lookup_fault_t) -> u64 {
    lookup_fault.words[0] & 0x3u64
}

#[inline]
pub fn lookup_fault_depth_mismatch_new(bitsFound: u64, bitsLeft: u64) -> lookup_fault_t {
    lookup_fault_t {
        words: [
            ((bitsFound & 0x7fu64) << 9)
                | ((bitsLeft & 0x7fu64) << 2)
                | (lookup_fault_tag_t::lookup_fault_depth_mismatch as u64 & 0x3u64),
            0u64,
        ],
    }
}

#[inline]
pub fn lookup_fault_depth_mismatch_get_bitsFound(lookup_fault: lookup_fault_t) -> u64 {
    (lookup_fault.words[0] & 0xfe00u64) >> 9
}

#[inline]
pub fn lookup_fault_depth_mismatch_get_bitsLeft(lookup_fault: lookup_fault_t) -> u64 {
    (lookup_fault.words[0] & 0x1fu64) >> 2
}

#[inline]
pub fn lookup_fault_missing_capability_get_bitsLeft(lookup_fault: lookup_fault_t) -> u64 {
    (lookup_fault.words[0] & 0x1fcu64) >> 2
}

#[inline]
pub fn lookup_fault_guard_mismatch_new(
    guardFound: u64,
    bitsLeft: u64,
    bitsFound: u64,
) -> lookup_fault_t {
    lookup_fault_t {
        words: [
            ((bitsLeft & 0x7fu64) << 9)
                | ((bitsFound & 0x7fu64) << 2)
                | (lookup_fault_tag_t::lookup_fault_guard_mismatch as u64 & 0x3u64),
            guardFound,
        ],
    }
}

#[inline]
pub fn lookup_fault_guard_mismatch_get_bitsFound(lookup_fault: lookup_fault_t) -> u64 {
    (lookup_fault.words[0] & 0x1fcu64) >> 2
}

#[inline]
pub fn lookup_fault_guard_mismatch_get_bitsLeft(lookup_fault: lookup_fault_t) -> u64 {
    (lookup_fault.words[0] & 0xfe00u64) >> 9
}

#[inline]
pub fn lookup_fault_missing_capability_new(bitsLeft: u64) -> lookup_fault_t {
    lookup_fault_t {
        words: [
            ((bitsLeft & 0x7fu64) << 2)
                | ((lookup_fault_tag_t::lookup_fault_missing_capability as u64) & 0x3u64),
            0,
        ],
    }
}

#[inline]
pub fn notification_ptr_get_ntfnQueue_head(notification_ptr: &notification_t) -> u64 {
    let mut ret = notification_ptr.words[1] & 0xffffffffffffu64;
    if (ret & (1u64 << 47)) != 0u64 {
        ret |= 0xffff000000000000u64;
    }
    ret
}

#[inline]
pub fn notification_ptr_set_ntfnQueue_head(notification_ptr: &mut notification_t, v64: u64) {
    notification_ptr.words[1] &= !0xffffffffffffu64;
    notification_ptr.words[1] |= v64 & 0xffffffffffffu64;
}

#[inline]
pub fn notification_ptr_get_ntfnQueue_tail(notification_ptr: &notification_t) -> u64 {
    let mut ret = notification_ptr.words[0] & 0xffffffffffff0000u64;
    if (ret & (1u64 << 47)) != 0u64 {
        ret |= 0xffff000000000000u64;
    }
    ret
}

#[inline]
pub fn notification_ptr_set_ntfnQueue_tail(notification_ptr: &mut notification_t, v64: u64) {
    notification_ptr.words[0] &= !0xffffffffffff0000u64;
    notification_ptr.words[0] |= (v64 << 16) & 0xffffffffffff0000u64;
}

#[inline]
pub fn notification_ptr_get_state(notification_ptr: &notification_t) -> u64 {
    notification_ptr.words[0] & 0x3u64
}

#[inline]
pub fn notification_ptr_set_state(notification_ptr: &mut notification_t, v64: u64) {
    notification_ptr.words[0] &= !0x3u64;
    notification_ptr.words[0] |= v64 & 0x3u64;
}

#[inline]
pub fn notification_ptr_get_ntfnMsgIdentifier(notification_ptr: &notification_t) -> u64 {
    notification_ptr.words[2] & 0xffffffffffffffffu64
}

#[inline]
pub fn notification_ptr_set_ntfnMsgIdentifier(notification_ptr: &mut notification_t, v64: u64) {
    notification_ptr.words[2] &= !0xffffffffffffffffu64;
    notification_ptr.words[2] |= v64 & 0xffffffffffffffffu64;
}

#[inline]
pub fn notification_ptr_get_ntfnBoundTCB(notification_ptr: &notification_t) -> u64 {
    let mut ret = notification_ptr.words[3] & 0xffffffffffffu64;
    if (ret & (1u64 << 47)) != 0u64 {
        ret |= 0xffff000000000000u64;
    }
    ret
}

#[inline]
pub fn notification_ptr_set_ntfnBoundTCB(notification_ptr: &mut notification_t, v64: u64) {
    (*notification_ptr).words[3] &= !0xffffffffffffu64;
    (*notification_ptr).words[3] |= v64 & 0xffffffffffffu64;
}

#[inline]
pub fn thread_state_ptr_get_tsType(thread_state_ptr: &thread_state_t) -> u64 {
    thread_state_ptr.words[0] & 0xfu64
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct seL4_CNode_CapData_t {
    pub words: [u64; 1],
}

#[inline]
pub fn seL4_CNode_CapData_get_guard(seL4_CNode_CapData: seL4_CNode_CapData_t) -> u64 {
    (seL4_CNode_CapData.words[0] & 0xffffffffffffffc0u64) >> 6
}

#[inline]
pub fn seL4_CNode_CapData_get_guardSize(seL4_CNode_CapData: seL4_CNode_CapData_t) -> u64 {
    seL4_CNode_CapData.words[0] & 0x3fu64
}
