EXTENSION    = jsoncdc
EXTVERSION   = $(shell egrep '^default_version += +' *.control | cut -d"'" -f2)

DATA         = $(filter-out $(wildcard sql/*--*.sql),$(wildcard sql/*.sql))
DOCS         = $(wildcard doc/*.md)
TESTS        = $(wildcard test/sql/*.sql)
REGRESS      = $(patsubst test/sql/%.sql,%,$(TESTS))
REGRESS_OPTS = --inputdir=test --load-language=plpgsql
PG_CONFIG    = pg_config
PG91         = $(shell $(PG_CONFIG) --version | \
                       grep -qE " 8\.| 9\.0" && echo no || echo yes)

ifeq ($(PG91),yes)
all: sql/$(EXTENSION)--$(EXTVERSION).sql

sql/$(EXTENSION)--$(EXTVERSION).sql: sql/$(EXTENSION).sql
	cp $< $@

DATA = $(wildcard sql/*--*.sql) sql/$(EXTENSION)--$(EXTVERSION).sql
EXTRA_CLEAN = sql/$(EXTENSION)--$(EXTVERSION).sql
endif


# Note that `MODULES = jsoncdc` implies a dependency on `jsoncdc.so`.
MODULES      = jsoncdc

ifeq ($(shell uname -s),Darwin)
LINK_FLAGS   = -C link-args='-Wl,-undefined,dynamic_lookup'
endif

generate:
	bash generate_bindings.sh 

jsoncdc.so: generate
	cargo rustc --release -- $(LINK_FLAGS)
	cp target/release/libjsoncdc.* $@

cargoclean:
	cargo clean


PGXS := $(shell $(PG_CONFIG) --pgxs)
include $(PGXS)

clean: cargoclean

all: jsoncdc.so
