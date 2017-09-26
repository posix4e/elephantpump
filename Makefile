EXTENSION    = jsoncdc
EXTVERSION   = $(shell egrep '^default_version += +' *.control | cut -d"'" -f2)

DATA         = $(filter-out $(wildcard sql/*--*.sql),$(wildcard sql/*.sql))
DOCS         = $(wildcard doc/*.md)
TESTS        = $(wildcard test/sql/*.sql)
REGRESS      = $(patsubst test/sql/%.sql,%,$(TESTS))
REGRESS_OPTS = --inputdir=test --load-language=plpgsql
PG_CONFIG    = pg_config
PG91         = $(shell $(PG_CONFIG) --version | \
                       grep -qE " 8[.]| 9[.]0" && echo no || echo yes)
PG96         = $(shell $(PG_CONFIG) --version | \
                       grep -qE " 8[.]| 9[.][0-5]" && echo no || echo yes)

ifeq ($(PG91),yes)
all: sql/$(EXTENSION)--$(EXTVERSION).sql

sql/$(EXTENSION)--$(EXTVERSION).sql: sql/$(EXTENSION).sql
	cp $< $@

EXTRA_CLEAN = sql/$(EXTENSION)--$(EXTVERSION).sql
endif


# Note that `MODULES = jsoncdc` implies a dependency on `jsoncdc.so`.
MODULES     := jsoncdc
PGXX        := $(shell util/get_version)
HAZRUST     := $(shell which cargo >/dev/null && echo yes || echo no)

ifeq ($(shell uname -s),Darwin)
LINK_FLAGS  := -C link-args='-Wl,-undefined,dynamic_lookup'
endif

ifeq ($(PG96),yes)
FEATURES    := --features pg-ldc-messages
else
REGRESS     := $(subst message,,$(REGRESS))
endif

ifeq ($(HAZRUST),yes)
.PHONY: jsoncdc.so
jsoncdc.so:
	cargo rustc $(FEATURES) --release -- $(LINK_FLAGS)
	cp target/release/libjsoncdc.* $@

.PHONY: cargoclean
cargoclean:
	find . -name Cargo.lock -exec rm {} \;
	cargo clean
else
define CAN_HAZ_RUST

We need a Rust toolchain (rustc and cargo) to compile this extension.

See: https://www.rust-lang.org/downloads.html


endef
# NB: Not phony so if they build the extension somehow, the rest of the
#     install can be completed.
jsoncdc.so:
	$(error $(CAN_HAZ_RUST))

.PHONY: cargoclean
cargoclean:
	$(warning No Rust toolchain so not cleaning anything.)
endif

PGXS := $(shell $(PG_CONFIG) --pgxs)
include $(PGXS)


clean: cargoclean

all: jsoncdc.so

.PHONY: test
test:
	pgxn check ./
	util/checkstyle
