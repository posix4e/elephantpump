ARG PG_VERSION

FROM postgres:${PG_VERSION}

ENV CARGO_HOME /cargo
ENV PATH $CARGO_HOME/bin:$PATH
ENV SRC_PATH /src

RUN apt-get update \
  && apt-get install -y --no-install-recommends \
    ca-certificates curl git make gcc gcc-multilib postgresql-server-dev-$PG_MAJOR=$PG_VERSION \
    python-pip python-setuptools \
  && rm -rf /var/lib/apt/lists/* \
  && curl -sf https://static.rust-lang.org/rustup.sh -o rustup.sh \
  && bash rustup.sh --disable-sudo -y --verbose \
  && pip install pgxnclient \
  && cargo install rustfmt \
  && mkdir -p "$CARGO_HOME"

WORKDIR $SRC_PATH

VOLUME $SRC_PATH

COPY util/docker /docker-entrypoint-initdb.d/docker.sh
