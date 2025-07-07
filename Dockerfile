FROM rust:1.88 AS builder
LABEL org.opencontainers.image.source=https://github.com/mlb-rs/mlbt
LABEL org.opencontainers.image.description="A terminal user interface for the MLB Statcast API, written in Rust."
LABEL org.opencontainers.image.licenses=MIT

WORKDIR /usr/src/mlbt
COPY . .
RUN cargo build --release

FROM gcr.io/distroless/cc
COPY --from=builder /usr/src/mlbt/target/release/mlbt /usr/local/bin/mlbt
CMD ["/usr/local/bin/mlbt"]
