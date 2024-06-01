use proc_macro_utils::*;

// #[shared_fn]
// fn func_cell(arg1: i32, arg2: String) -> Option<u64> {
//     // Function body
//     Some(8)
// }

shared_group! {{

    fn func_cell(arg1: i32, arg2: String) -> Option<u64> {
        // Function body
        Some(8)
    }

    fn func_go(arg1: i32) -> u8 {
        // Function body
        1
    }
}}

fn main() {}
