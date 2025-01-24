# Build Stage
FROM rust:slim-bullseye AS builder
WORKDIR /build

# Compile deps in a separate layer (for caching)
COPY Cargo.toml Cargo.lock ./
# Cargo requires at least one source file for compiling dependencies
RUN mkdir src && echo "fn main() { println!(\"Hello, world!\"); }" > src/main.rs
RUN apt-get update
RUN apt install -y pkg-config libssl-dev
RUN cargo build --release

# Compile for release
COPY ./src ./src
RUN rm ./target/release/deps/amd*
RUN cargo build --release

# Release Stage
FROM debian:bullseye-slim AS release
COPY --from=builder /build/target/release/amd /usr/local/bin
CMD ["/usr/local/bin/amd"]
