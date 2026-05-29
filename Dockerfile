FROM rust:1-bookworm AS base

ARG DEBIAN_FRONTEND=noninteractive

RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    ca-certificates \
    git \
    golang-go \
    make \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app/bbs

COPY Cargo.toml Cargo.lock Makefile ./
COPY go ./go
COPY include ./include
COPY src ./src

FROM base AS ci

RUN make ci

FROM scratch AS artifacts

COPY --from=ci /app/bbs/dist/libbbsplus-linux-amd64.tar.gz /libbbsplus-linux-amd64.tar.gz
