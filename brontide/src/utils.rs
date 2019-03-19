#![feature(core_intrinsics)]

// Slightly modified version of dbg! macro which includes also type information and column name
// This works only in night builds of rust.
// It uses approach from https://stackoverflow.com/questions/21747136/how-do-i-print-the-type-of-a-variable-in-rust/29168659#29168659
macro_rules! tdbg {
    ($val:expr) => {
        // Use of `match` here is intentional because it affects the lifetimes
        // of temporaries - https://stackoverflow.com/a/48732525/1063961
        match $val {
            tmp => {
                eprintln!("[{}:{}:{}] {} = {:#?} ({})",
                    file!(), line!(), column!(), stringify!($val), &tmp, type_of(&tmp));
                tmp
            }
        }
    }
}

pub fn print_type_of<T>(_: &T) {
    println!("{}", unsafe { std::intrinsics::type_name::<T>() });
}