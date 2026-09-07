# Meal Roulette

Randomizes a week of meals from a Postgres library of recipes and builds a shopping list
from the result. Weeknight picks lean quick-and-easy; weekends can run more ambitious.

Personal project — also a deliberate excuse to learn a Rust backend (on the path toward
embedded). The full feature vision lives in [docs/product/roadmap.md](docs/product/roadmap.md).

## Stack

- **Frontend** — Dioxus 0.7 (Rust, fullstack; `#[server]` functions call the data layer in-process) (`web/`)
- **Backend** — Rust (Axum + SQLx) as a library the web server embeds; one Cargo workspace: `api`, `domain`, `web`
- **Database** — Postgres (via Docker locally)

## Layout

```
api/                 data layer: state, errors, repositories, REST router (+ a thin standalone bin) + SQL migrations
domain/              shared wire types (api <-> web <-> future device)
web/                 Dioxus app (fullstack): SSR, #[server] functions, REST routes under /api — one process
docs/                architecture, product, decisions, guides
docker-compose.yml   local Postgres
```

## Prerequisites

- [Rust](https://rustup.rs) (stable)
- [dioxus-cli](https://dioxuslabs.com/) (`dx`) — `cargo install dioxus-cli`
- [Docker](https://www.docker.com/)

## Run locally

Two processes, one per terminal:

```bash
# 1. Postgres
docker compose up -d
```

```bash
# 2. Web  ->  http://localhost:8080  (the app, and the REST routes under /api)
cd web && dx serve
```

The REST routes can also run on their own with `cargo run -p api` (http://localhost:3000);
the web app does not need that.

Environment variables, migrations, and the `just` task recipes are covered in
[docs/guides/local-dev.md](docs/guides/local-dev.md).
