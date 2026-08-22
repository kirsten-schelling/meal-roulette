# Meal Roulette

Personal project: randomizes a week of meals from Postgres recipes and builds a shopping
list. Learning-oriented — Rust backend as an on-ramp toward embedded. Feature vision in
[docs/ROADMAP.md](docs/ROADMAP.md).

## Stack
- Frontend: Dioxus 0.7 (Rust, fullstack — `#[server]` BFF) — `web/`
- Backend: Rust (Axum + SQLx) — Cargo workspace: `crates/api`, `crates/domain`
- Database: Postgres (Docker locally)
- `web2/` (SvelteKit) and `web3/` (Next) — gitignored reference spikes from the UI comparison; not part of the build.

## Local dev
- DB:  `docker compose up -d`
- API: `cargo run -p api`     -> http://localhost:3000
- Web: `cd web && dx serve`   -> http://localhost:8080

## Conventions
- **Git attribution:** never add AI / Claude / Anthropic attribution to commit messages, PR
  titles or bodies, or branch names. No `Co-Authored-By: Claude ...`, no "Generated with
  Claude Code". Keep commit messages plain and descriptive.
- **Commit messages:** use `type(scope): description` (Conventional Commits). **Every commit
  has a scope.** Types: `feat`, `fix`, `docs`, `chore`, `refactor`, `test`, `build`, `ci`,
  `perf` (add others as needed). Scope: `api` or `web`; use `repo` for repo-wide changes
  (e.g. `chore(repo): …`, `docs(repo): …`). More explicit feature scopes will be defined after
  the MVP. Example: `feat(api): add recipes roll endpoint`.
