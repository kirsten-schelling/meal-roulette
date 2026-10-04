# Pending decisions

Decisions the project will need soon but has not made, plus decisions already living in the
code that have no ADR yet. When one is settled, it becomes a numbered ADR and leaves this
list.

## To decide

1. **Auth foundations** — the user table name (`app_user` vs alternatives; deferred in
   [data-model.md](../architecture/data-model.md)) and the OIDC client library (open in
   [stack.md](../architecture/stack.md)). Roadmap ordering puts this next; `household_id`
   already exists on the core tables.
2. **Route-target layering in `web/`** — `pages/` (shell stubs) currently coexists with
   `features/<f>/page.rs` (wired screens), which [web.md](../architecture/web.md) doesn't
   allow for. Decide whether `pages/` collapses into `features/` as screens get wired, or
   stays as a documented layer.
3. **Is `page.rs` mandatory per feature?** — cuisine, ingredient, protein, and unit ship
   hooks + a `#[server]` surface with no page, which web.md's "Adding a feature" steps
   don't permit. Loosen the rule or plan the pages.
4. **Testing strategy** — the workspace has zero tests and clippy is commented out of
   `just lint`, while stack.md promises TDD and a passing harness. Decide the harness
   (unit vs API-level, test DB approach) and re-enable clippy.
5. **JS toolchain in `web/`** — `just fmt`/`lint`/`check` still call `yarn` in `web/`,
   which has no `package.json` since the SvelteKit spike left. Either restore a formatter
   for CSS or strip the yarn lines.
6. **Shopping-list unit math** — the roadmap offers simple vs advanced modes;
   `recipe_ingredient` deliberately has no `unit_id`, and choosing "advanced" later forces
   a migration. Cheapest to decide before the shopping-list feature starts.
7. **Household scoping on the API** — all endpoints are unauthenticated and unscoped;
   `household_id` is nullable. The first real household/auth work makes every repository
   query grow a scope predicate — worth deciding the pattern once, up front.
8. **Fate of the leftovers** — `web/src/showcase.rs` (says "delete once real pages exist"),
   `web/src/server-fake.rs` (unreferenced, doesn't compile), and the `web2/`/`web3/`
   spikes ("deletable anytime"). Decide keep-or-delete so they stop drifting.

## Made in code, not yet recorded

Candidates for retro-ADRs, roughly by how much they'd surprise a reader:

- Crates live at the repo root (`api/`, `domain/`, `solver/`, `web/`), not under `crates/`,
  and `web` is a workspace member — several docs still describe the old layout.
- The roller is the `solver` workspace crate at `solver/`: C sources in `csrc/`, compiled by
  `build.rs` through `cc`, with the one `unsafe` call wrapped in `src/lib.rs`. The `api` roll
  feature assigns integer codes to lookup ids, calls it from `POST /roll` and
  `POST /planned-meals/{id}/reroll`, and stores the result in `meal_plan` + `planned_meal`.
- `domain` is the shared wire-type crate consumed by both `api` and `web` (web.md still
  calls this interim and points at the api crate).
- API feature modules are `mod.rs` + `handlers.rs` + `repository.rs` — no service layer
  (stack.md describes a different shape).
- Uniform CRUD-per-table REST surface over all 15 tables, rather than task-shaped
  endpoints.
- Error taxonomy: four variants in `api/src/error.rs`, with `RowNotFound` → 404 and
  unique violations → 409 mapped in `From<sqlx::Error>`.
- Dates cross the wire as ISO-8601 strings via `domain/src/date_format.rs`.
- `rust_decimal` with `serde-float` for `numeric` quantities (precision trade-off that
  matters when costs/pricing land).
- The sqlx offline query cache is committed and refreshed with `just prepare`.
- Hand-built UI primitives + CSS design tokens ("Warm" theme) instead of a component kit,
  with the `/showcase` gallery route.
