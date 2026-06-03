FROM rust:1-bookworm AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY craftman-server ./craftman-server
COPY protocol ./protocol
COPY craftman-agent ./craftman-agent

RUN SQLX_OFFLINE=true CARGO_BUILD_JOBS=1 cargo build --release -p craftman-server


FROM debian:bookworm-slim

RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates \
 && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/craftman-server /usr/local/bin/craftman-server

EXPOSE 3000

CMD ["craftman-server"]
