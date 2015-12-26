fn main() {
    println!("cargo:rustc-link-lib=pgcommon");
    println!("cargo:rustc-link-lib=pgport");
}
