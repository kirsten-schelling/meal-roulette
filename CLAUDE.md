# Meal Roulette

Personal project: randomizes a week of meals from Postgres recipes and builds a shopping
list. Learning-oriented — Rust backend as an on-ramp toward embedded. Feature vision in
[docs/ROADMAP.md](docs/ROADMAP.md).

## Stack
- Frontend: SvelteKit + TypeScript (yarn) — `web/`
- Backend: Rust (Axum + SQLx) — Cargo workspace: `crates/api`, `crates/domain`
- Database: Postgres (Docker locally)

## Local dev
- DB:  `docker compose up -d`
- API: `cargo run -p api`     -> http://localhost:3000
- Web: `cd web && yarn dev`   -> http://localhost:5173

## Conventions
- **Git attribution:** never add AI / Claude / Anthropic attribution to commit messages, PR
  titles or bodies, or branch names. No `Co-Authored-By: Claude ...`, no "Generated with
  Claude Code". Keep commit messages plain and descriptive.
