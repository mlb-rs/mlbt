FROM rust:1.88

WORKDIR /usr/src/mlbt
COPY . .

RUN cargo install --path .

CMD ["./target/release/mlbt"]
