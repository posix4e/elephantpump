#!/bin/sh 
set -ex
which bindgen || cargo install bindgen

echo '#include <stdarg.h>' > /tmp/postgres.c
echo '#include "postgres.h"' >> /tmp/postgres.c
echo '#include "fmgr.h"' >> /tmp/postgres.c
echo '#include "replication/output_plugin.h"' >> /tmp/postgres.c
echo '#include "replication/logical.h"' >> /tmp/postgres.c


gcc -I /usr/include/postgresql/9.4/server -E /tmp/postgres.c > /tmp/libpq.c

cat /tmp/libpq.c | python src/remove_duplicate_single_line_statements.py  > /tmp/libpq_dedup.c

bindgen  -allow-bitfields  -builtins  /tmp/libpq_dedup.c  > src/libpq.rs
