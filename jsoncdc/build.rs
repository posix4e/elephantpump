extern crate gcc;

use std::process::Command;


#[derive(Default)]
struct PGConfig {
    includedir: String,
    includedir_server: String,
    libdir: String
}


fn pg_config() -> PGConfig {
    let output = Command::new("pg_config").output().unwrap_or_else(|e| {
        panic!("Failed to execute process: {}", e)
    });
    /* Sample result:
        ...
        INCLUDEDIR = /usr/local/Cellar/postgresql/9.4.5/include
        PKGINCLUDEDIR = /usr/local/Cellar/postgresql/9.4.5/include
        INCLUDEDIR-SERVER = /usr/local/Cellar/postgresql/9.4.5/include/server
        LIBDIR = /usr/local/Cellar/postgresql/9.4.5/lib
        ...
     */

    let mut config = PGConfig { ..Default::default() };

    let text = String::from_utf8(output.stdout)
                      .expect("Expected UTF-8 from call to `pg_config`.");

    for words in text.lines().map(|line| line.split_whitespace()) {
        let vec: Vec<&str> = words.collect();
        match vec[0] {
            "INCLUDEDIR"        => config.includedir = vec[2].into(),
            "INCLUDEDIR-SERVER" => config.includedir_server = vec[2].into(),
            "LIBDIR"            => config.libdir = vec[2].into(),
            _                   => {}
        }
    }

    config
}

fn main() {
    let config = pg_config();
    gcc::Config::new()
                .file("src/magic.c")
                .include(config.includedir)
                .include(config.includedir_server)
                .compile("libmagic.a");
    // The GCC module emits `rustc-link-lib=static=magic` for us.
}
