#![allow(non_camel_case_types)]

pub enum seL4_Error {
    seL4_NoError = 0,
    seL4_InvalidArgument = 1,
    seL4_InvalidCapability = 2,
    seL4_IllegalOperation = 3,
    seL4_RangeError = 4,
    seL4_AlignmentError = 5,
    seL4_FailedLookup = 6,
    seL4_TruncatedMessage = 7,
    seL4_DeleteFirst = 8,
    seL4_RevokeFirst = 9,
    seL4_NotEnoughMemory = 10,
    seL4_NumErrors = 11,
}

#[macro_export]

macro_rules! userError {
    ($($x:expr),*) => {};
}
