# Pending decisions

Decisions the project will need soon but has not made, plus decisions already living in the
code that have no ADR yet. When one is settled, it becomes a numbered ADR and leaves this
list.

## To decide

1. **Auth foundations.** The user table name (`app_user` vs alternatives; deferred in
   [data-model.md](../architecture/data-model.md)) and the OIDC client library (open in
   [stack.md](../architecture/stack.md); the client is the Rust server, so the candidates are Rust
   crates). Roadmap ordering puts this next; `household_id` already exists on the core tables.
2. **Route-target layering in `web/`.** `pages/` (`home.rs`, a wired dashboard) coexists with
   `features/<f>/page.rs` (wired screens), which [web.md](../architecture/web.md) doesn't
   allow for. Decide whether `pages/` collapses into `features/` as screens get wired, or
   stays as a documented layer.
3. **API-level tests and lint rules.** CI (`.github/workflows/ci.yml`) runs `cargo fmt --check`,
   clippy with `-D warnings`, `cargo test --workspace` against a Postgres service, and
   Prettier. Unit tests sit beside the code and repository tests use `#[sqlx::test]`. Still
   open: whether to test the REST handlers and server functions as well, and stricter lint
   rules across the stack (issue #4).
4. **Household scoping on the API.** All endpoints are unauthenticated and unscoped;
   `household_id` is nullable. The one scoping pattern so far is
   `household::repository::default_household` (the oldest household, or an auto-created
   "Home"), which the roll service uses when a request names no household. The first real
   household or auth work makes every repository query grow a scope predicate, so the pattern
   is worth deciding once, up front.
5. **Fate of `web/src/showcase.rs`.** It says "delete once real pages exist" and is still
   routed at `/showcase`. Decide keep or delete so it stops drifting.

## Made in code, not yet recorded

Candidates for retro-ADRs, roughly by how much they'd surprise a reader:

- Crates live at the repo root (`api/`, `domain/`, `solver/`, `web/`), not under `crates/`,
  and `web` is a workspace member.
- The roller is the `solver` workspace crate at `solver/`: C sources in `csrc/`, compiled by
  `build.rs` through `cc`, with the one `unsafe` call wrapped in `src/lib.rs`. The `api` roll
  feature assigns integer codes to lookup ids, calls it from the roll routes and
  `POST /planned-meals/{id}/reroll`, and stores the result in `meal_plan` + `planned_meal`.
- `domain` is the shared wire-type crate consumed by both `api` and `web`.
- API feature modules are `mod.rs` + `handlers.rs` + `repository.rs`. `roll` and
  `housekeeping` have a `service.rs` instead of a repository; `recipe_import` is a service with
  no handlers, reached through a server function; `ingredient_alias` has a repository and no
  handlers.
- A CRUD-per-table REST surface, plus task-shaped endpoints such as `POST /roll`,
  `POST /planned-meals/{id}/reroll`, `GET /meal-plans/latest`, `GET /meal-plans/history` and
  shopping-list generate.
- Error taxonomy: six variants in `api/src/error.rs` (`Db`, `NotFound`, `Conflict`,
  `BadRequest`, `Unprocessable`, `Internal`), with per-constraint messages, unique violations
  and `ON DELETE RESTRICT` (23001) mapped to 409, and check violations to 400.
- Dates cross the wire as ISO-8601 strings via `domain/src/date_format.rs`.
- `rust_decimal` with `serde-float` for `numeric` quantities (a precision trade-off that
  matters when costs and pricing land).
- The sqlx offline query cache is committed and refreshed with `just prepare`.
- Hand-built UI primitives + CSS design tokens ("Warm" theme) instead of a component kit,
  with the `/showcase` gallery route.
- The roll as a weighted constraint solver: Required / High / Medium / Low importance, with
  unmet preferences loosened and reported (`domain::roll`, `solver/csrc/solver.c`).
- Roll presets, with at most one default per household.
- `default_household` as the interim household scope.
- LF-only migration files via `.gitattributes` (commit 8602283), so sqlx checksums match on
  Windows.
