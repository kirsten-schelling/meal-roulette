# Meal Roulette

Personal project: randomizes a week of meals from Postgres recipes and builds a shopping
list. Learning-oriented — Rust backend as an on-ramp toward embedded. Feature vision in
[docs/product/roadmap.md](docs/product/roadmap.md).

## Stack
- Frontend: Dioxus 0.7 (Rust, fullstack — `#[server]` functions call the data layer in-process) — `web/`
- Backend: Rust (Axum + SQLx) — `api/` is a library (state, errors, repositories, REST router)
  embedded by the web server; shared wire types in `domain/`. One root Cargo workspace: `api`,
  `domain`, `solver`, `web`.
- Solver: the meal roller, written in C (`solver/csrc/`) and compiled by `solver/build.rs` into
  the `solver` crate, which exposes a safe `roll()` over one `extern "C"` call. The `api` roll
  feature (`api/src/features/roll/`) flattens recipes and rules into its integer-only input.
- Database: Postgres (Docker locally)
- `web2/` (SvelteKit) and `web3/` (Next) — gitignored reference spikes from the UI comparison; not part of the build.

## Web app
`web/` follows a required module structure — features, the `#[server]` BFF surface, and the
split between generic and feature-scoped components/hooks. Read
[docs/architecture/web.md](docs/architecture/web.md) before adding or changing code under
`web/`.

## Local dev
- DB:  `docker compose up -d`
- Web: `cd web && dx serve`   -> http://localhost:8080 (app + `#[server]` functions + REST under `/api`)
- REST on its own (optional): `cargo run -p api` -> http://localhost:3000

## Conventions
- **Teaching mode (learning project):** For API / web-server / non-UI logic, default to *hints*
  toward the fix, not finished code — until I explicitly ask for the answer or to implement. A
  question means no change by default; only edit when I say "implement". Exception: components and
  styling/UI — just build those.
- **Comments:** default to no comment. Add one only when something is genuinely hard to
  understand from the code itself, or when the reason for something is not clear from the
  code — explain **why**, not **what**. Comments must never be responses to a prompt or
  replies to review feedback.
- **Git attribution:** never add AI / Claude / Anthropic attribution to commit messages, PR
  titles or bodies, or branch names. No `Co-Authored-By: Claude ...`, no "Generated with
  Claude Code". Keep commit messages plain and descriptive.
- **Commit messages:** use `type(scope): description` (Conventional Commits). **Every commit
  has a scope.** Types: `feat`, `fix`, `docs`, `chore`, `refactor`, `test`, `build`, `ci`,
  `perf` (add others as needed). Scope: `api` or `web`; use `repo` for repo-wide changes
  (e.g. `chore(repo): …`, `docs(repo): …`). More explicit feature scopes will be defined after
  the MVP. Example: `feat(api): add recipes roll endpoint`.
