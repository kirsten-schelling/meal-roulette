FROM rust:1-bookworm AS build
RUN rustup target add wasm32-unknown-unknown \
 && cargo install dioxus-cli --version 0.7.10 --locked \
 && cargo install sqlx-cli --version 0.9.0 --locked --no-default-features --features rustls,postgres
WORKDIR /src
COPY . .
ENV SQLX_OFFLINE=true
RUN dx bundle --package web --web --release

FROM tailscale/tailscale:v1.104.4 AS tailscale

FROM debian:bookworm-slim
RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates \
 && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=tailscale /usr/local/bin/tailscaled /usr/local/bin/tailscale /usr/local/bin/
COPY --from=build /usr/local/cargo/bin/sqlx /usr/local/bin/sqlx
COPY --from=build /src/target/dx/web/release/web/ /app/
COPY --from=build /src/api/migrations /app/migrations
COPY deploy/start.sh /app/start.sh
RUN chmod +x /app/start.sh
ENV IP=127.0.0.1 PORT=8080
CMD ["/app/start.sh"]
