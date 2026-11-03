# Local development

How to run Meal Roulette locally. See [../architecture/stack.md](../architecture/stack.md) for
the tooling rationale.

## Prerequisites

- Rust (rustup; MSVC toolchain on Windows)
- A C compiler for the `solver` crate (the MSVC Build Tools on Windows, gcc or clang elsewhere)
- Docker (local Postgres)
- `dioxus-cli` (`dx`) — the web app
- `sqlx-cli` — migrations
- `just` — the task recipes

## Services

Run the database, apply the migrations, then start the web server:

| Service    | Command                | URL                                             |
| ---------- | ---------------------- | ----------------------------------------------- |
| Database   | `docker compose up -d` | Postgres on localhost:5432                      |
| Migrations | `just migrate`         | — (first run, and after pulling new migrations) |
| Web        | `cd web && dx serve`   | http://localhost:8080                           |

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

Task recipes live in the `justfile` — run `just` to list them. The main ones: `db`, `db-down`,
`web`, `api`, `solver`, `migrate`, `migrate-down`, `new-migration`, `prepare`, `fmt`, `lint`,
`check`. `cargo test --workspace` runs the tests.

## Solver

The C solver in `solver/csrc` is compiled by cargo through `solver/build.rs`, so any `cargo`
command that builds `api` or the web server build also builds it. `cargo test -p solver` runs
the solver tests (the C/Rust struct layouts agree, plus behaviour tests for validation, required rules, caps, difficulty bands, per-day limits, seeding and preference loosening). `just solver` compiles
the standalone host harness and runs it against `solver/input.txt` for printf-style debugging
without the server.

## Migrations

SQL migrations live in `api/migrations`, managed with `sqlx-cli` and reversible (`-r`).
Migrations are the schema source of truth — never edit one that has been applied; add a new
one. The app does not apply migrations at startup: `just migrate` runs the pending ones and then
refreshes `.sqlx`; `just migrate-down` reverts the latest; `just new-migration <name>` creates a
reversible pair. See [../architecture/data-model.md](../architecture/data-model.md).
