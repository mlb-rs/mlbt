FROM rust:latest

WORKDIR /usr/src/mlbt
COPY . .

RUN cargo install --path .

CMD ["./target/release/mlbt"]
