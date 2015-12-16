fn main() {
    println!("cargo:rustc-link-lib=static=pgcommon");
    println!("cargo:rustc-link-search=native=/usr/local/lib/");
}
