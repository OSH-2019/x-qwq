#[macro_export]

#[cfg(not(smp))]
macro_rules! node_state {
    ($i:ident) => ($i)
}

use crate::structures::tcb_t;
extern "C"{
    pub static mut ksCurThread:*mut tcb_t;
}