# Meal Roulette

Randomizes a week of meals from a Postgres library of recipes and builds a shopping list
from the result. Weeknight picks lean quick-and-easy; weekends can run more ambitious.

Personal project — also a deliberate excuse to learn a Rust backend (on the path toward
embedded). The full feature vision lives in [docs/product/roadmap.md](docs/product/roadmap.md).

## Stack

- **Frontend** — SvelteKit + TypeScript (`web/`), managed with yarn
- **Backend** — Rust (Axum + SQLx), a Cargo workspace: `crates/api` + a shared `crates/domain`
- **Database** — Postgres (via Docker locally)

## Layout

```
crates/api/          Axum HTTP server
crates/domain/       shared types (server <-> future device)
web/                 SvelteKit app
docs/                architecture, product, decisions, guides
docker-compose.yml   local Postgres
```

## Prerequisites

- [Rust](https://rustup.rs) (stable)
- Node 24 — pinned in `.node-version` (e.g. via [fnm](https://github.com/Schniz/fnm))
- [yarn](https://classic.yarnpkg.com/)
- [Docker](https://www.docker.com/)

## Run locally

Three processes, one per terminal:

```bash
# 1. Postgres
docker compose up -d
```

```bash
# 2. API  ->  http://localhost:3000
cargo run -p api
```

```bash
# 3. Web  ->  http://localhost:5173
cd web && yarn dev
```

> Early scaffold: the API doesn't connect to Postgres yet — wiring that up is the next slice.
