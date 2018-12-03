FROM rust:1.30.1

COPY . .
RUN cargo test
