extern crate elephantpump;
use elephantpump::*;

fn main() {
    let a = std::ffi::CString::new("a").unwrap();
    let b = std::ffi::CString::new("b").unwrap();
    unsafe {
        println!("A ends with A: {}", pg_str_endswith(a.as_ptr(), a.as_ptr()));
        println!("A ends with B: {}", pg_str_endswith(a.as_ptr(), b.as_ptr()));
    }
}


