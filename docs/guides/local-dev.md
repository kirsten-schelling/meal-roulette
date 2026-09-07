# Local development

How to run Meal Roulette locally. See [../architecture/stack.md](../architecture/stack.md) for
the tooling rationale.

## Prerequisites

- Rust (rustup; MSVC toolchain on Windows)
- Docker (local Postgres)
- `dioxus-cli` (`dx`) — the web app
- `sqlx-cli` — migrations
- `just` — the task recipes
- Node 24 via fnm — only for the gitignored `web2/` / `web3/` spikes

## Services

Two processes, one per terminal:

| Service | Command | URL |
| --- | --- | --- |
| Database | `docker compose up -d` | Postgres on localhost:5432 |
| Web | `cd web && dx serve` | http://localhost:8080 |

The web server is the whole backend: it renders the app, runs the `#[server]` functions, and
serves the REST routes under http://localhost:8080/api (for curl and the device). The REST
routes can also run on their own with `cargo run -p api` (http://localhost:3000); the web app
does not need that process.

One env var lives in the root `.env` (see `.env.example`):

- `DATABASE_URL` — feeds `sqlx-cli`, the compile-time query macros, and both servers at
  startup; the web server panics on startup without it.

The server build compiles the sqlx queries. When no database is reachable at build time, set
`SQLX_OFFLINE=true` to build from the committed `.sqlx` cache (refresh it with `just prepare`
after changing a query or the schema).

## just recipes

Task recipes live in the `justfile` — run `just` to list them.

## Migrations

SQL migrations live in `api/migrations`, managed with `sqlx-cli` and reversible (`-r`).
Migrations are the schema source of truth — never edit one that has been applied; add a new
one. See [../architecture/data-model.md](../architecture/data-model.md).
