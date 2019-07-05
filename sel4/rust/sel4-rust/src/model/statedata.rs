#[macro_export]
#[cfg(not(smp))]
macro_rules! node_state {
    ($i:ident) => {
        $i
    };
}
