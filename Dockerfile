# syntax=docker/dockerfile:1

FROM rust:1.87 AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim

COPY --from=builder /app/target/release/omsn /app/omsn

ADD https://github.com/krallin/tini/releases/download/v0.19.0/tini /tini
RUN chmod +x /tini

ENTRYPOINT ["/tini", "--", "/app/omsn"]