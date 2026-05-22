FROM rust:1-bookworm

ARG DEBIAN_FRONTEND=noninteractive

RUN apt-get update && apt-get install -y --no-install-recommends \
    git \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY . /app/bbs/

WORKDIR /app/bbs

RUN cargo build --release

CMD ["bash", "-lc", "ls -lh target/release/libbbs_ffi.*"]
