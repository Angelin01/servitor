FROM rust:alpine3.21 AS builder

ARG TARGETPLATFORM="linux/amd64"
ARG VERSION

RUN apk add --no-cache musl-dev gcc make cmake g++

RUN if [ "$TARGETPLATFORM" = "linux/amd64" ]; then \
        echo "x86_64-unknown-linux-musl" > /tmp/target; \
    elif [ "$TARGETPLATFORM" = "linux/arm64" ]; then \
        echo "aarch64-unknown-linux-musl" > /tmp/target; \
    elif [ "$TARGETPLATFORM" = "linux/arm/v7" ]; then \
        echo "armv7-unknown-linux-musleabihf" > /tmp/target; \
    else \
        echo "unsupported platform" && exit 1; \
    fi

RUN rustup default nightly-2025-02-03 && \
    rustup target add "$(cat /tmp/target)"

COPY .cargo/ ./.cargo/

RUN cargo install -f --no-default-features --features "set-version" cargo-edit

WORKDIR /app
RUN cargo new --bin servitor
WORKDIR /app/servitor

COPY Cargo.toml Cargo.lock ./

RUN cargo build --release --locked --target --target "$(cat /tmp/target)" && \
    rm src/*.rs

RUN if [ ! -z "$VERSION" ]; then \
    cargo set-version "${VERSION}"; \
    fi

COPY src src/

RUN cargo build --release --locked --target-dir /target --target "$(cat /tmp/target)" && \
    mv "/target/$(cat /tmp/target)/release/servitor" /target/servitor

FROM alpine:3.21

WORKDIR /app

RUN addgroup -S servitor && adduser -S servitor -G servitor

COPY --from=builder /target/servitor /app/servitor

USER servitor

ENTRYPOINT ["/app/servitor"]
