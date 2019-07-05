#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

pub enum exception {
    EXCEPTION_NONE = 0,
    EXCEPTION_FAULT = 1,
    EXCEPTION_LOOKUP_FAULT = 2,
    EXCEPTION_SYSCALL_ERROR = 3,
    EXCEPTION_PREEMPTED = 4,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct syscall_error_t {
    pub invalidArgumentNumber: u64,
    pub invalidCapNumber: u64,
    pub rangeErrorMin: u64,
    pub rangeErrorMax: u64,
    pub memoryLeft: u64,
    pub failedLookupWasSource: u64,
    pub type_: u64,
}
