FROM postgres:9.5

ENV PATH ~/.cargo/bin/:$PATH
ENV CARGO_HOME /cargo
ENV SRC_PATH /src

COPY util/docker-pg /tmp/docker-pg
RUN /tmp/docker-pg

COPY util/docker-rust /tmp/docker-rust
RUN mkdir -p $CARGO_HOME
RUN /tmp/docker-rust

WORKDIR $SRC_PATH

VOLUME $SRC_PATH

COPY util/docker /docker-entrypoint-initdb.d/docker.sh
