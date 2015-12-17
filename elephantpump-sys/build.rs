fn main() {
    println!("cargo:rustc-link-search=native=/usr/local/opt/openssl/lib");
    println!("cargo:rustc-link-lib=static=ssl");
    println!("cargo:rustc-link-search=native=/usr/local/lib/");
    println!("cargo:rustc-link-lib=static=pq");
    println!("cargo:rustc-link-lib=static=pgport");
    println!("cargo:rustc-link-lib=static=pgcommon");
}
