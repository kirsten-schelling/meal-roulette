# 2. One process for the web app and the data layer

- Status: Accepted
- Date: 2026-09-07

## Context

The web app started as a backend-for-frontend in front of a separate Rust REST API on `:3000`.
That shape was chosen while SvelteKit and Next were still candidates, when a separate Rust
process was the only way to have Rust on the server. With Dioxus fullstack the web server is
already Rust on Axum, and the hop stopped paying for itself:

- every `#[server]` function was pure forwarding: serialise, HTTP, deserialise, re-wrap the error;
- `fetcher.rs`, `reqwest`, `serde_json` and `API_URL` existed only to make the hop work;
- the CORS layer allowed `localhost:8080`, but the browser never called `:3000`;
- each request paid a second serialisation and round trip, and the two binaries could disagree
  on a `domain` type during a rolling deploy.

A separate process earns its place when two parts need different owners, release cadence,
scaling shape or trust boundary. None of those applies: one user base, one owner, and Postgres
is the bottleneck long before an Axum process is. "Many users" is served by replicas of one
binary.

## Decision

One process serves the Dioxus app and the data layer.

- `api` is a **library** crate: `state::AppState`, `error::AppError`, `features/*` (handlers
  and repositories) and `app::build`, the REST router. A thin `main.rs` still runs the REST
  routes on their own (`cargo run -p api`), so a second binary stays one `main.rs` away.
- The web server build depends on `api` behind the `server` feature only (`optional = true`,
  `server = ["dioxus/server", "dep:api"]`), so sqlx, axum and tokio never enter the wasm build.
- `#[server]` functions call `api::features::<x>::repository` directly. Logic both adapters
  need — the recipe-creation transaction, resolving an inline unit — lives in the repository,
  so the REST handlers and the server functions stay thin adapters over the same functions.
- Errors flow through `AppError`, whose `From<sqlx::Error>` maps row-not-found to 404 and
  constraint violations to 409, and become a `ServerFnError` in one place
  (`web/src/server/error.rs`) that keeps the status code.
- `api::app::build(state)` is nested under `/api` in the same Axum router, for curl and the
  device / simulator. Page routes such as `/recipes` and REST routes such as `/api/recipes`
  therefore never collide.
- `AppState` reaches server functions as an Axum request extension. In Dioxus 0.7,
  `ServeConfig::context` only provides context to server-side rendering, not to
  server-function requests, so `web/src/server/state.rs` reads the extension back through
  `FullstackContext`.
- `domain` is unchanged: it is still the wire contract across the wasm/server boundary and for
  the device.

## Consequences

- One config (`DATABASE_URL`), one deploy, one Fly app. `dx serve` against Docker Postgres is
  the whole local stack.
- The server build compiles the sqlx queries. The committed `.sqlx` offline cache (refreshed by
  `just prepare`) plus `SQLX_OFFLINE=true` covers builds with no database reachable.
- Auth becomes one layer: the Dioxus server validates the session cookie and passes the
  household scope into the repositories. JWT validation on the REST routes is only needed once
  an outside client calls them.
- The solver is server-gated inside the same process; the device payload endpoint is a REST
  route in the same router.
- Splitting again is a new binary over the `api` library, not a rewrite. Reasons to do it, in
  rough order of likelihood: a worker for differently-shaped work; a client outside our control
  that needs a versioned REST contract; a second owner with their own release cadence; a trust
  boundary; measured contention.
