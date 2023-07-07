FROM rust:1.68 as builder
WORKDIR /app
COPY . .
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl-dev \
    git clang curl libssl-dev llvm libudev-dev make protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

RUN rustup default stable && rustup update && rustup target add wasm32-unknown-unknown
RUN rustup show

RUN cargo build --release

FROM ubuntu:22.04
COPY --from=builder /app/target/release/node-template /app/

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl-dev \
    git clang gcc curl libssl-dev llvm libudev-dev make \
    && rm -rf /var/lib/apt/lists/* && mkdir /data

CMD ["./app/node-template"]
