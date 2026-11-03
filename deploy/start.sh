#!/bin/sh
set -eu

mkdir -p /data/tailscale /var/run/tailscale

tailscaled \
  --tun=userspace-networking \
  --state=/data/tailscale/tailscaled.state \
  --socket=/var/run/tailscale/tailscaled.sock &

tailscale up --auth-key="${TS_AUTHKEY}" --hostname=meal-roulette

tailscale serve --bg 8080

exec /app/server
