# Pending decisions

Decisions the project will need soon but has not made, plus decisions already living in the
code that have no ADR yet. When one is settled, it becomes a numbered ADR and leaves this
list.

## To decide

1. **Auth foundations** — the user table name (`app_user` vs alternatives; deferred in
   [data-model.md](../architecture/data-model.md)) and the OIDC client library (open in
   [stack.md](../architecture/stack.md); the client is the Rust server, so the candidates are Rust
   crates). Roadmap ordering puts this next; `household_id`
   already exists on the core tables.
2. **Route-target layering in `web/`** — `pages/` (`home.rs`, a wired dashboard, and `cooking_log.rs`, still a stub) currently coexists with
   `features/<f>/page.rs` (wired screens), which [web.md](../architecture/web.md) doesn't
   allow for. Decide whether `pages/` collapses into `features/` as screens get wired, or
   stays as a documented layer.
3. **Is `page.rs` mandatory per feature?** — cuisine, ingredient, protein, and unit ship
   hooks + a `#[server]` surface with no page, which web.md's "Adding a feature" steps
   don't permit (tag ships hooks and a server surface with no page too). Loosen the rule or
   plan the pages.
4. **Testing strategy** — there are 26 unit tests (`solver` 17, the `api` roll service 4, `web`
   `recipe/filter.rs` 3 and `roll/settings.rs` 2) and `just lint` runs clippy with
   `-D warnings`. Still missing: DB- or API-level tests and CI (issues #3, #4). Decide the
   harness (unit vs API-level, test DB approach).
5. **JS leftovers** — the justfile no longer calls `yarn` and there is no `package.json`, but
   `.node-version`, `.prettierignore` and the `/web2/`, `/web3/` lines in `.gitignore` remain.
   Delete them or keep them.
6. **Shopping-list unit math** — the roadmap offers simple vs advanced modes;
   `recipe_ingredient` deliberately has no `unit_id`, and choosing "advanced" later forces
   a migration. The shopping list has shipped (generate, check-off, sleep and wake) and works
   in simple mode. Record that as a decision, or choose advanced.
7. **Household scoping on the API** — all endpoints are unauthenticated and unscoped;
   `household_id` is nullable. The one scoping pattern so far is
   `household::repository::default_household` (the oldest household, or an auto-created
   "Home"), which the roll service uses when a request names no household. The first real household/auth work makes every repository
   query grow a scope predicate — worth deciding the pattern once, up front.
8. **Fate of `web/src/showcase.rs`** — says "delete once real pages exist" and is still routed
   at `/showcase`. Decide keep-or-delete so it stops drifting.

## Made in code, not yet recorded

Candidates for retro-ADRs, roughly by how much they'd surprise a reader:

- Crates live at the repo root (`api/`, `domain/`, `solver/`, `web/`), not under `crates/`,
  and `web` is a workspace member.
- The roller is the `solver` workspace crate at `solver/`: C sources in `csrc/`, compiled by
  `build.rs` through `cc`, with the one `unsafe` call wrapped in `src/lib.rs`. The `api` roll
  feature assigns integer codes to lookup ids, calls it from `POST /roll` and
  `POST /planned-meals/{id}/reroll`, and stores the result in `meal_plan` + `planned_meal`.
- `domain` is the shared wire-type crate consumed by both `api` and `web`.
- API feature modules are `mod.rs` + `handlers.rs` + `repository.rs`; `roll` adds a
  `service.rs` instead of a repository (the one service layer).
- A CRUD-per-table REST surface over all 16 tables (17 feature modules), plus task-shaped
  endpoints: `POST /roll`, `POST /planned-meals/{id}/reroll`, `GET /meal-plans/latest`,
  `GET /meal-plans/history` and shopping-list generate.
- Error taxonomy: six variants in `api/src/error.rs` (`Db`, `NotFound`, `Conflict`,
  `BadRequest`, `Unprocessable`, `Internal`), with per-constraint messages, unique violations
  and `ON DELETE RESTRICT` (23001) → 409, and check violations → 400.
- Dates cross the wire as ISO-8601 strings via `domain/src/date_format.rs`.
- `rust_decimal` with `serde-float` for `numeric` quantities (precision trade-off that
  matters when costs/pricing land).
- The sqlx offline query cache is committed and refreshed with `just prepare`.
- Hand-built UI primitives + CSS design tokens ("Warm" theme) instead of a component kit,
  with the `/showcase` gallery route.
- The roll as a weighted constraint solver: Required / High / Medium / Low importance, with
  unmet preferences loosened and reported (`domain::roll`, `solver/csrc/solver.c`).
- Roll presets, with at most one default per household.
- Shopping-list simple-mode aggregation (no unit conversion).
- `default_household` as the interim household scope.
- LF-only migration files via `.gitattributes` (commit 8602283), so sqlx checksums match on
  Windows.
