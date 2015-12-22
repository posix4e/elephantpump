extern crate pkg_config;

fn main ()
{
    match pkg_config::find_library("pgcommon") {
        Ok(_) => {
            if cfg!(target_os = "macos") {
                println!("cargo:rustc-flags=-L /usr/local/lib -lpgcommon");
            } else {
                println!("cargo:rustc-flags=-lpgcommon");
            }
        },
        Err(e) => {
            println!("error: SMB Client library not found! Probably libsmbclient is not installed.");
            panic!("{}", e);
        }
    };
}
