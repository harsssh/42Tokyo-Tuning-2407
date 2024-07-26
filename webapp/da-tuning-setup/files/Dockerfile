ARG BUILD_TYPE=debug

FROM rust:1.79.0 AS with-mold
RUN apt-get update && apt-get install -y mold


FROM with-mold AS dev
RUN cargo install cargo-watch
WORKDIR /app
CMD [ "cargo", "watch", "-s", "mold -run cargo run" ]


FROM with-mold AS builder

RUN apt-get update && apt-get install -y cmake curl && \
    curl -L https://github.com/mozilla/sccache/releases/download/v0.8.1/sccache-v0.8.1-x86_64-unknown-linux-musl.tar.gz | \
    tar xz -C /usr/local/bin --strip-components=1 sccache-v0.8.1-x86_64-unknown-linux-musl/sccache

ENV RUSTC_WRAPPER=sccache
ENV SCCACHE_CACHE_SIZE=10G
ENV SCCACHE_DIR=/usr/local/sccache

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src/ ./src
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/usr/local/sccache \
    cargo fetch --locked

ARG BUILD_TYPE
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/usr/local/sccache \
    if [ "$BUILD_TYPE" = "release" ]; then \
    mold -run cargo build --release; \
    else \
    mold -run cargo build; \
    fi


FROM rust:1.79.0-slim-bookworm AS runtime
ARG BUILD_TYPE
ARG CMD_NAME
RUN test -n "$CMD_NAME"
COPY --from=builder "/app/target/$BUILD_TYPE/$CMD_NAME" "/usr/local/bin/$CMD_NAME"
ENV CMD_NAME_ENV="$CMD_NAME"
CMD [ "sh", "-c", "/usr/local/bin/$CMD_NAME_ENV" ]
