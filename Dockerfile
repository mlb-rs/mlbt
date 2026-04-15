FROM rust:1.88 AS builder
LABEL org.opencontainers.image.source=https://github.com/agiacalone/mlbtg
LABEL org.opencontainers.image.description="MLB in your terminal — a fork of mlbt with visual accessibility enhancements."
LABEL org.opencontainers.image.licenses=MIT

WORKDIR /usr/src/mlbtg
COPY . .
RUN cargo build --release

FROM gcr.io/distroless/cc
COPY --from=builder /usr/src/mlbtg/target/release/mlbtg /usr/local/bin/mlbtg
CMD ["/usr/local/bin/mlbtg"]
