FROM rust:1-bookworm AS build
RUN rustup target add wasm32-unknown-unknown \
 && cargo install dioxus-cli --version 0.7.10 --locked
WORKDIR /src
COPY . .
ENV SQLX_OFFLINE=true
RUN dx bundle --package web --web --release

FROM debian:bookworm-slim
RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates \
 && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=build /src/target/dx/web/release/web/ /app/
ENV IP=0.0.0.0 PORT=8080
EXPOSE 8080
CMD ["/app/web"]
