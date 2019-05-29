#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(dead_code)]
#![allow(non_upper_case_globals)]
use crate::types;
use crate::types::bool_t;
use crate::types::word_t;
use crate::structures::cap_t;
use crate::structures::cap_tag_t;

//对应generated/arch/object/structures_gen.h中的部分内容
const cap_zombie_cap:u64=18;
#[inline]
pub fn cap_zombie_cap_new(capZombieID:u64, capZombieType:u64)->cap_t{
    cap_t{
        words:[0|(cap_zombie_cap&31u64)<<59|(capZombieType&127u64)<<0,
        0|capZombieID<<0]
    }
}

#[inline]
pub fn cap_zombie_cap_get_capZombieType(cap:cap_t)->u64{
    cap.words[0]&127u64
}

#[inline]
pub fn cap_zombie_cap_get_capZombieID(cap:cap_t)->u64{
    cap.words[1]&0xffffffffffffffffu64
}

#[inline]
pub fn cap_zombie_cap_set_capZombieID(mut cap:cap_t,v64:u64)->cap_t{
    cap.words[1] &= ! 0xffffffffffffffffu64;
    cap.words[1] |= v64 & 0xffffffffffffffffu64;
    cap
}

#[inline]
pub fn cap_frame_cap_get_capFSize(cap:cap_t)->u64{
    (cap.words[0]>>59) & 0x1fu64
}

#[inline]
pub fn cap_get_capType(cap:cap_t)->u64{
    (cap.words[0]>>59) & 0x1fu64
}

#[inline]
pub fn isArchCap(cap:cap_t)->word_t{
    cap_get_capType(cap) % 2
}

#[inline]
pub fn cap_cnode_cap_get_capCNodeRadix(cap:cap_t)->u64{
    (cap.words[0] & 0x1f800000000000u64)>>47
}

#[inline]
pub fn cap_endpoint_cap_get_capEPBadge(cap:cap_t)->u64{
    cap.words[1] & 0xffffffffffffffffu64
}

#[inline]
pub fn cap_notification_cap_get_capNtfnBadge(cap:cap_t)->u64{
    cap.words[1] & 0xffffffffffffffffu64
}

//对应generated/arch/object/structures_gen.h，这些是从pdl的structures.rs搬来的……
pub fn cap_untyped_cap_get_capPtr(cap: cap_t) -> u64 {
    let mut ret = cap.words[0] & 0xffffffffffffu64;
    if ret & (1u64 << 47) != 0 {
        ret |= 0xffff000000000000u64;
    }
    ret
}

pub fn cap_untyped_cap_set_capFreeIndex(mut cap: cap_t, v64: u64) -> cap_t {
    cap.words[1] &= !0xffffffffffff0000u64;
    cap.words[1] |= (v64 << 16) & 0xffffffffffff0000u64;
    cap
}
pub fn cap_untyped_cap_get_capFreeIndex(cap: cap_t) -> u64 {
    (cap.words[1] & 0xffffffffffff0000u64) >> 16
}
pub fn cap_untyped_cap_get_capBlockSize(cap: cap_t) -> u64 {
    cap.words[1] & 0x3fu64
}
pub fn cap_untyped_cap_get_capIsDevice(cap: cap_t) -> u64 {
    (cap.words[1] & 0x40u64) >> 6
}

//对应include/arch/x86/arch/machine/registerset.h中的部分内容
const CONFIG_XSAVE_SIZE:usize=512;
#[repr(C)]
struct user_fpu_state_t{
    state: [u8;CONFIG_XSAVE_SIZE]
}

const n_contextRegisters:usize=23;
#[repr(C)]
struct user_context_t{
    fpuState: user_fpu_state_t,
    registers: [word_t;n_contextRegisters]
}

//对应include/arch/x86/arch/machine/hardware.h中的部分内容
#[repr(C)]
enum vm_page_size {
    X86_SmallPage,
    X86_LargePage,
    X64_HugePage
}
type vm_page_size_t=word_t;

const seL4_PageBits:u64=12;
const seL4_LargePageBits:u64=21;
const seL4_HugePageBits:u64=30;

#[inline]
fn pageBitsForSize(pagesize:vm_page_size_t)->word_t{
    match pagesize{
        pagesize if pagesize==(vm_page_size::X86_SmallPage as u64) =>
            seL4_PageBits,
        pagesize if pagesize==(vm_page_size::X86_LargePage as u64) =>
            seL4_LargePageBits,
        pagesize if pagesize==(vm_page_size::X64_HugePage as u64) =>
            seL4_HugePageBits,
        _ => panic!("Invalid page size") //原来是fail，这里改成panic
    }
}

//对应include/arch/x86/arch/64/mode/object/structures.h中的部分内容
const seL4_PML4Bits:u64=12;
const seL4_PDPTBits:u64=12;

#[inline]
fn cap_get_modeCapSizeBits(cap:cap_t)->word_t{
    let ctag=cap_get_capType(cap);
    match ctag{
        ctag if ctag==(cap_tag_t::cap_pml4_cap as u64) =>
            seL4_PML4Bits,
        ctag if ctag==(cap_tag_t::cap_pdpt_cap as u64) =>
            seL4_PDPTBits,
        _ => 0
    }
}

#[inline]
fn cap_get_modeCapIsPhysical(cap:cap_t)->bool_t{
    let ctag=cap_get_capType(cap);
    match ctag{
        ctag if ctag==(cap_tag_t::cap_pml4_cap as u64) ||
                ctag==(cap_tag_t::cap_pdpt_cap as u64) =>
            types::_bool::r#true as u64,
        _ => types::_bool::r#false as u64
    }
}


//对应include/arch/x86/arch/object/structures.h中的部分内容
#[repr(C)]
pub struct arch_tcb_t{
    tcbContext: user_context_t
}

const seL4_PageTableBits:u64=12;
const seL4_PageDirBits:u64=12;
const seL4_ASIDPoolBits:u64=12;

#[inline]
pub fn cap_get_archCapSizeBits(cap:cap_t)->word_t{
    let ctag=cap_get_capType(cap);
    match ctag{
        ctag if ctag==(cap_tag_t::cap_frame_cap as u64) =>
            pageBitsForSize(cap_frame_cap_get_capFSize(cap)),
        ctag if ctag==(cap_tag_t::cap_page_table_cap as u64) =>
            seL4_PageTableBits,
        ctag if ctag==(cap_tag_t::cap_page_directory_cap as u64) =>
            seL4_PageDirBits,
        ctag if ctag==(cap_tag_t::cap_io_port_cap as u64) =>
            0,
        ctag if ctag==(cap_tag_t::cap_asid_control_cap as u64) =>
            0,
        ctag if ctag==(cap_tag_t::cap_asid_pool_cap as u64) =>
            seL4_ASIDPoolBits,
        _ => cap_get_modeCapSizeBits(cap)
    }
}

#[inline]
pub fn cap_get_archCapIsPhysical(cap:cap_t)->bool_t{
    let ctag=cap_get_capType(cap);
    match ctag{
        ctag if ctag==(cap_tag_t::cap_frame_cap as u64) ||
                ctag==(cap_tag_t::cap_page_table_cap as u64) ||
                ctag==(cap_tag_t::cap_page_directory_cap as u64) ||
                ctag==(cap_tag_t::cap_asid_pool_cap as u64) =>
            types::_bool::r#true as u64,
        ctag if ctag==(cap_tag_t::cap_io_port_cap as u64) ||
                ctag==(cap_tag_t::cap_asid_control_cap as u64) =>
            types::_bool::r#false as u64,
        _ => cap_get_modeCapIsPhysical(cap)
    }
}

#[inline]
pub fn Arch_isCapRevocable(derivedCap:cap_t,srcCap:cap_t)->bool_t{
    if cap_get_capType(derivedCap) == cap_tag_t::cap_io_port_cap as u64 {
        ( cap_get_capType(srcCap) == cap_tag_t::cap_io_port_control_cap as u64 ) as u64
    } else {
        types::_bool::r#false as u64
    }
}