FROM rust:1.88 AS builder
WORKDIR /usr/src/mlbt
COPY . .
RUN cargo build --release

FROM gcr.io/distroless/cc
COPY --from=builder /usr/src/mlbt/target/release/mlbt /usr/local/bin/mlbt
CMD ["/usr/local/bin/mlbt"]
