use std::sync::LazyLock;
use std::collections::BTreeMap;
use pipa::vm::{StringVars, ArrayVars};
use pipa::error::{CompileError, ErrorReason};


pub static VARS: LazyLock<StringVars> = LazyLock::new(|| {
    BTreeMap::from([
        // ASCII vars
        ("first".into(), "first arg".into()),
        ("second".into(), "second arg".into()),
        ("third".into(), "third arg".into()),
        // ASCII vars with int component
        ("value_0".into(), "value 0".into()),
        ("value_1".into(), "value 1".into()),
        // UTF-8 vars
        ("forth".into(), "четвертый аргумент".into()),
        // don't really know these languages :)
        ("fifth".into(), "第五の議論".into()),
        ("sixth".into(), "第六个论点".into()),
        ("seventh".into(), "séptimo argumento".into()),
    ])
});


pub static ARRAYS: LazyLock<ArrayVars> = LazyLock::new(|| {
    BTreeMap::from([
        // ASCII arrays
        ("ARGS".into(), vec!["first element".into(), "second element".into(), "third element".into()]),
        ("PHONES".into(), vec!["555-123-4567".into(), "555-987-6543".into(), "555-555-0000".into()]),
        // ASCII arrays with int component
        ("ARGS0".into(), vec!["first element".into(), "second element".into(), "third element".into()]),
        ("ARGS_0".into(), vec!["first element".into(), "second element".into(), "third element".into()]),
        // UTF-8 arrays
        ("UTF".into(), vec!["первый".into(), "segunda".into(), "三番目".into()]),
    ])
});


pub fn err_reason<T>(r: Result<T, CompileError>) -> ErrorReason {
    match r {
        Ok(_) => panic!("Result should be an error"),
        Err(r) => r.reason
    }
}

#[macro_export]
macro_rules! assert_matches {
    ($left:expr, $right:pat_param) => {
        if !matches!($left, $right) {
            panic!(
                "
assertion `left == right` failed
 left: {:?}
 right: {}
                ", $left, stringify!($right)
            );
        }
    }
}

#[macro_export]
macro_rules! assert_ok {
    ($cond:expr,) => {
        $crate::assert_ok!($cond);
    };
    ($cond:expr) => {
        match $cond {
            Ok(t) => t,
            Err(e) => {
                panic!("assertion failed, expected Ok(..), got Err({:?})", e);
            }
        }
    };
    ($cond:expr, $($arg:tt)+) => {
        match $cond {
            Ok(t) => t,
            Err(e) => {
                panic!("assertion failed, expected Ok(..), got Err({:?}): {}", e, format_args!($($arg)+));
            }
        }
    };
}
