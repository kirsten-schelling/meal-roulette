# Meal Roulette

Rolls a random week of dinners from recipes in Postgres and builds a shopping list. Feature
ideas and their status: [docs/product/roadmap.md](docs/product/roadmap.md).

## Stack

- Frontend: Dioxus 0.7 (Rust, fullstack: `#[server]` functions call the data layer
  in-process), in `web/`
- Backend: Rust (Axum + SQLx). `api/` is a library (state, errors, repositories, REST router)
  embedded by the web server; shared wire types in `domain/`. One root Cargo workspace: `api`,
  `domain`, `solver`, `web`.
- Solver: the meal roller, written in C (`solver/csrc/`) and compiled by `solver/build.rs` into
  the `solver` crate, which exposes a safe `roll()` over one `extern "C"` call. The `api` roll
  feature (`api/src/features/roll/`) flattens recipes and rules into its integer-only input.
- Database: Postgres 18 (Docker locally, Neon in production)

## Web app

`web/` follows a required module structure: features, the `#[server]` surface, and the split
between generic and feature-scoped components and hooks. Read
[docs/architecture/web.md](docs/architecture/web.md) before adding or changing code under
`web/`.

## Local dev

- DB: `docker compose up -d`, then `just migrate` (the app does not run migrations at startup)
- Web: `cd web && dx serve` -> http://localhost:8080 (app + `#[server]` functions + REST under `/api`)
- REST on its own (optional): `cargo run -p api` -> http://localhost:3000
- Tests: `cargo test --workspace`; lint: `just lint`
- Deploy: a merge to `main` deploys to Fly.io and runs migrations
  ([docs/guides/deploy.md](docs/guides/deploy.md))

## Conventions

- **Comments:** default to no comment. Add one only when something is hard to understand from
  the code itself, or when the reason for it is not clear from the code: explain **why**, not
  **what**. Comments are never responses to a prompt or replies to review feedback.
- **Commit messages:** `type(scope): description` (Conventional Commits). **Every commit has a
  scope.** Types: `feat`, `fix`, `docs`, `chore`, `refactor`, `test`, `build`, `ci`, `perf`.
  Scope: `api` or `web`, or `repo` for repo-wide changes such as CI, docs and tooling (for
  example `docs(repo): ...`). Example: `feat(api): add recipes roll endpoint`.
