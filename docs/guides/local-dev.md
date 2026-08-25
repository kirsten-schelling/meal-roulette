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
| Database | `docker compose up -d` | Postgres on the default port |
| API | `cargo run -p api` | http://localhost:3000 |
| Web | `cd web && dx serve` | http://localhost:8080 |

`DATABASE_URL` lives in the root `.env`; it feeds both `sqlx-cli` and the compile-time query
macros.

## just recipes

`just db`, `just migrate`, `just new-migration <name>`, `just api`, `just web`, `just fmt`,
`just lint`, `just check`.

## Migrations

SQL migrations live in `crates/api/migrations`, managed with `sqlx-cli` and reversible (`-r`).
Migrations are the schema source of truth — never edit one that has been applied; add a new
one. See [../architecture/data-model.md](../architecture/data-model.md).
