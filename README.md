# Meal Roulette

Randomizes a week of meals from a Postgres library of recipes and builds a shopping list
from the result. Rolls follow per-week rules on difficulty, cuisine, protein and must-use ingredients, and each
day of the roll (Day 1, Day 2, ...) can have its own difficulty and cook-time limits.

Personal project — also a deliberate excuse to learn a Rust backend (on the path toward
embedded). The full feature vision lives in [docs/product/roadmap.md](docs/product/roadmap.md).

## Stack

- **Frontend** — Dioxus 0.7 (Rust, fullstack; `#[server]` functions call the data layer in-process) (`web/`)
- **Backend** — Rust (Axum + SQLx) as a library the web server embeds; one Cargo workspace: `api`, `domain`, `solver`, `web`
- **Solver** — the meal roller, written in C and compiled into the `solver` crate (`solver/`), called by the backend through a safe Rust wrapper
- **Database** — Postgres (via Docker locally)

## Layout

```
api/                 data layer: state, errors, repositories, REST router (+ a thin standalone bin) + SQL migrations
domain/              shared wire types (api <-> web <-> future device)
solver/              the meal roller: C sources in csrc/, compiled by build.rs, safe wrapper in src/lib.rs
web/                 Dioxus app (fullstack): SSR, #[server] functions, REST routes under /api — one process
docs/                architecture, product, decisions, guides
docker-compose.yml   local Postgres
```

## Prerequisites

- [Rust](https://rustup.rs) (stable)
- A C compiler for the solver: on Windows, the MSVC Build Tools the Rust toolchain already needs; gcc or clang elsewhere
- [dioxus-cli](https://dioxuslabs.com/) (`dx`) — `cargo install dioxus-cli`
- [Docker](https://www.docker.com/)
- [sqlx-cli](https://crates.io/crates/sqlx-cli) and [just](https://github.com/casey/just) — migrations and the task recipes

## Run locally

Three steps; the first two run once and return, the last stays running:

```bash
# 1. Postgres
docker compose up -d
```

```bash
# 2. Apply the migrations (the app does not run them at startup)
just migrate
```

```bash
# 3. Web  ->  http://localhost:8080  (the app, and the REST routes under /api)
cd web && dx serve
```

The REST routes can also run on their own with `cargo run -p api` (http://localhost:3000);
the web app does not need that.

Environment variables, migrations, and the `just` task recipes are covered in
[docs/guides/local-dev.md](docs/guides/local-dev.md).
