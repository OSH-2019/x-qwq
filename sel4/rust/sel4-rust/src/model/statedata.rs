#[macro_export]

#[cfg(not(smp))]
macro_rules! note_state {
    ($i:ident) => ($i)
}