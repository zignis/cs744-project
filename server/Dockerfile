# builder
FROM rust:1.86-alpine3.21 AS builder

# see https://github.com/rust-lang/docker-rust/issues/85
ENV RUSTFLAGS="-C target-feature=-crt-static"

RUN apk add --no-cache musl-dev openssl-dev pkgconfig

WORKDIR /app
COPY ./ /app

RUN cargo build --release
RUN strip target/release/server

# runner
FROM alpine:3.21

RUN apk add --no-cache libgcc
COPY --from=builder /app/target/release/server .
ENTRYPOINT ["/server"]