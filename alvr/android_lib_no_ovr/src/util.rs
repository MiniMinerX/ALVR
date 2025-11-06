use std::time::SystemTime;


pub type StrResult<T = ()> = Result<T, String>;



/// Execute the $b with the return value $t, call 'show_err' and return Option<$t>.
/// The default of $t is ().
#[macro_export]
macro_rules! catch_err {
    ($b:block,$t:ty) => {{
        use alvr_common::show_err;
        let s = || -> $crate::util::StrResult<$t> {
            Ok($b)
        };
        show_err(s())
    }};
    ($b:block) => {
        catch_err!($b,())
    };
}

pub const US_IN_SEC: u64 = 1000 * 1000;

pub fn get_timestamp_us() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH).unwrap()
        .as_micros() as u64
}

#[macro_export]
macro_rules! trace_str {
    () => {
        format!("At {}:{}", file!(), line!())
    };
}

#[macro_export]
macro_rules! trace_err {
    ($res:expr) => {
        $res.map_err(|e| format!("{}: {}", $crate::trace_str!(), e))
    };
}