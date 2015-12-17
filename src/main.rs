extern crate elephantpump_sys;

fn main() {
    unsafe {
        println!("Hello World! {}", elephantpump_sys::h_errno);
    }
}


