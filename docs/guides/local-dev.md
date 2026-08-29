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

Three processes, one per terminal:

| Service | Command | URL |
| --- | --- | --- |
| Database | `docker compose up -d` | Postgres on localhost:5432 |
| API | `cargo run -p api` | http://localhost:3000 |
| Web | `cd web && dx serve` | http://localhost:8080 |

Both env vars live in the root `.env` (see `.env.example`):

- `DATABASE_URL` — feeds `sqlx-cli` and the compile-time query macros.
- `API_URL` — the base URL the web server's `#[server]` functions use to reach the API
  (locally `http://localhost:3000`); the web server panics on startup without it.

## just recipes

Task recipes live in the `justfile` — run `just` to list them.

## Migrations

SQL migrations live in `api/migrations`, managed with `sqlx-cli` and reversible (`-r`).
Migrations are the schema source of truth — never edit one that has been applied; add a new
one. See [../architecture/data-model.md](../architecture/data-model.md).
