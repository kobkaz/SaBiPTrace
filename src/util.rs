#[macro_export]
macro_rules! impl_wrap_from_many {
    ($t:ident, $m:ident, [$($var:ident),*]) => {
        $(impl From<$m::$var> for $t {
            fn from(m: $m::$var) -> Self {
                $t::$var(m)
            }
        })*
    };
}
