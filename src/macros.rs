#[macro_export]
macro_rules! skip_if {
    ($e:expr) => { if $e { continue; }}
}