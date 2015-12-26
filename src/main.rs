extern crate elephantpump;
use elephantpump::libpq::*;

fn main() {
    unsafe {
        println!("A random number: {}", pg_lrand48());
        println!("Another: {}", pg_lrand48());
    }
}


