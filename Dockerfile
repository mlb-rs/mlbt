FROM rust:1.86

WORKDIR /usr/src/mlbt
COPY . .

RUN cargo install --path .

CMD ["./target/release/mlbt"]
