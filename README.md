# Meal Roulette

Randomizes a week of meals from a Postgres library of recipes and builds a shopping list
from the result. Weeknight picks lean quick-and-easy; weekends can run more ambitious.

Personal project — also a deliberate excuse to learn a Rust backend (on the path toward
embedded). The full feature vision lives in [docs/product/roadmap.md](docs/product/roadmap.md).

## Stack

- **Frontend** — Dioxus 0.7 (Rust, fullstack — `#[server]` BFF) (`web/`)
- **Backend** — Rust (Axum + SQLx); one Cargo workspace: `api`, `domain`, `web`
- **Database** — Postgres (via Docker locally)

## Layout

```
api/                 Axum HTTP server + SQL migrations
domain/              shared wire types (api <-> web <-> future device)
web/                 Dioxus app (fullstack, #[server] BFF)
docs/                architecture, product, decisions, guides
docker-compose.yml   local Postgres
```

## Prerequisites

- [Rust](https://rustup.rs) (stable)
- [dioxus-cli](https://dioxuslabs.com/) (`dx`) — `cargo install dioxus-cli`
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
# 3. Web  ->  http://localhost:8080
cd web && dx serve
```

Environment variables, migrations, and the `just` task recipes are covered in
[docs/guides/local-dev.md](docs/guides/local-dev.md).
