fn main() {
    println!("cargo:rustc-link-search=native=/usr/local/opt/openssl/lib");
    println!("cargo:rustc-link-lib=dylib=ssl");
    println!("cargo:rustc-link-search=native=/usr/local/lib/");
    println!("cargo:rustc-link-lib=dylib=pq");
    println!("cargo:rustc-link-lib=dylib=pgport");
    println!("cargo:rustc-link-lib=static=pgcommon");
}
