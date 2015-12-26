extern crate libc;

#[allow(dead_code, non_camel_case_types)]

pub type Oid = libc::c_uint;
pub type pg_int64 = libc::c_long;

extern "C" {
    pub fn pg_str_endswith(s1: *const libc::c_char,
                           s2: *const libc::c_char) -> libc::c_int;
}
