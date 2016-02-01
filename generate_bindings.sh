#!/bin/sh
set -xe
BINDGEN_HASH="3d2f57cabe8fbb9ce098e04b55450d9a52d92946"
export DYLD_LIBRARY_PATH=/Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/lib/:$DYLD_LIBRARY_PATH
git clone https://github.com/crabtw/rust-bindgen.git|| true
(cd rust-bindgen && git checkout ${BINDGEN_HASH} && cargo build --release)

echo '#include <stdarg.h>' > /tmp/postgres.c
echo '#include "postgres.h"' >> /tmp/postgres.c
echo '#include "fmgr.h"' >> /tmp/postgres.c
echo '#include "replication/output_plugin.h"' >> /tmp/postgres.c
echo '#include "replication/logical.h"' >> /tmp/postgres.c

INCLUDE_DIR=`pg_config --includedir-server`
gcc -I $INCLUDE_DIR -E /tmp/postgres.c > /tmp/libpq.c

# if it's an export statement rust can't handle dedups as order works different
# in c than rust
cat /tmp/libpq.c | python src/remove_duplicate_single_line_statements.py	 > /tmp/libpq_dedup.c

# add a bogus entry since single entry enums don't work in rust
cat /tmp/libpq_dedup.c | sed 's/SCM_RIGHTS = 0x01/SCM_RIGHTS = 0x01,DUMMY = 0x00/' > /tmp/n &&  mv /tmp/n /tmp/libpq_dedup.c

# For linux we need the builtins, older versions of rust prefered allow-bitfields
# allow-bitfield may be removed in the future


rust-bindgen/target/release/bindgen -allow-bitfields -builtins \
	/tmp/libpq_dedup.c  -o src/libpq.rs
