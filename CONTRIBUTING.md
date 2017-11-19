Contributing to JSONCDC
-----------------------

Please use `cargo fmt` as part of the commit process. (CI and `make test` do a
style check so non-conformance will break the build.)

Please ensure that contributed Bash, SQL, Python and other files all:

1. resemble the surrounding code, and

2. contain lines no longer than 79 columns.

Using Docker for development
----------------------------

You can use the project's [Dockerfile](Dockerfile) to get a working development environment.

##### Usage

Build the image locally:

```sh
docker build -t jsoncdc-dev:9.5 --build-arg PG_VERSION=9.5 .
```

Start the container by mapping the source code volume - this should be done on the directory where you've checked out jsoncdc:

```sh
docker run --rm -it --name jsoncdc -v $(pwd):/src jsoncdc-dev:9.5
```

Run the test suite on another shell using the `postgres`:

```sh
docker exec jsoncdc bash -c 'make install && make test PGUSER=postgres'
```

##### Environment

- Cargo binaries are exposed on the `$PATH`. This means that `make test` will be able to run the `style` script correctly.
- Postgres does not need any additional manual configuration.
- The PGXN Client tool (`pgxnclient`) is pre-installed to allow `installcheck` to run correctly.
