extern crate libc;

extern "C" {
    pub fn pg_str_endswith(s1: *const ::libc::c_char,
                           s2: *const ::libc::c_char) -> ::libc::c_int;
}

