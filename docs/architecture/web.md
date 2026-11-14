# Web app module structure

The **web app** is the Dioxus 0.7 fullstack application in `web/`. This document defines the
required module layout, the responsibility of each layer, and the rules that govern the
client/server boundary. It is normative: new code under `web/` is expected to follow it.

The `recipe` feature is the reference implementation.

## Build model

The web app compiles from one source tree into **two binaries**:

- **Client build** — WebAssembly, runs in the browser. Enabled by the `web` feature.
- **Server build** — native, runs the whole server side in one process: server-side rendering,
  the `#[server]` functions, and the REST routes from the `api` crate. Enabled by the `server`
  feature, which is also the only thing that pulls the `api` crate (and with it sqlx, axum and
  tokio) into the build.

Every component and most modules are compiled into _both_. Server-only code is compiled into
the server build alone, gated with `#[cfg(feature = "server")]`.

## Layout

```
web/src/
├── main.rs                     # entry point: launch (client) or server::serve (server), root App
├── router.rs                   # Route enum: URL <-> page component
├── models.rs                   # small view helpers (wire types come from the domain crate)
├── showcase.rs                 # temporary /showcase component gallery
│
├── components/                 # GENERIC, cross-feature UI only
│   ├── ui/                     #   primitives (button, card, input, combobox, modal, ...)
│   ├── layout/                 #   app shell (header, sidebar, app_layout)
│   ├── difficulty_badge.rs     #   shared difficulty badge
│   └── toast_master.rs         #   toast host
│
├── pages/                      # shell route pages outside a feature: home.rs (a wired
│                               #   dashboard) (its fate: docs/decisions/pending.md)
├── hooks/                      # GENERIC, cross-feature client hooks
│
├── server/                     # SERVER-SIDE boundary
│   ├── api/                    #   #[server] functions (the RPC surface the browser calls)
│   │   └── <feature>.rs
│   ├── serve.rs                #   [server] pool + AppState, Axum router: app + REST under /api
│   ├── state.rs                #   [server] app_state(): AppState from the request extension
│   └── error.rs                #   [server] server_fn_error(): AppError -> ServerFnError
│
└── features/                   # one folder per feature
    └── <feature>/
        ├── page.rs             #   route-target component
        ├── hooks/              #   client data hooks (wrap use_resource)
        └── components/         #   feature-specific UI
```

Routes (`router.rs`): `/`, `/showcase`, `/recipes`, `/recipes/:id`, `/library`, `/shopping-list`,
`/shopping-list/:plan_id`, `/cooking-log`, `/planned-meals`, `/planned-meals/:id`, `/history`,
`/roll`.

`[server]` marks modules compiled only with the `server` feature.

## Responsibilities

- **`features/<feature>/page.rs`** — the component a route renders. Composes UI and reads data
  from the feature's hooks. No data access, no business logic.
- **`features/<feature>/hooks/`** — client-side data hooks (`use_*`) that wrap `use_resource`
  and call a `#[server]` function. They own the call-to-type mapping.
- **`features/<feature>/components/`** — UI used only by this feature.
- **`components/`** — UI reusable by more than one feature. Litmus test: if a second feature
  could use it, it lives here; otherwise it lives in the feature.
- **`server/api/<feature>.rs`** — the `#[server]` functions for that feature. The only surface
  the browser calls. Each takes `AppState` from `app_state()`, calls the feature's repository
  (or, for roll and re-roll, `api::features::roll::service`) in the `api` crate, and maps the
  error with `server_fn_error`.
- **`server/serve.rs`** — server-only bootstrap: connects the pool, builds the Axum router
  (Dioxus app, server functions, REST routes nested under `/api`), attaches `AppState` to every
  request as an extension, and serves.
- **`server/state.rs`**, **`server/error.rs`** — the two helpers every server function uses.
- **`hooks/`** — generic client hooks shared across features.

## Data flow

```
page.rs  ->  feature hook  ->  #[server] fn      ->  api::features::<x>::repository  ->  Postgres
(browser)    (browser)         (RPC -> server)       (api crate, same process)
```

The browser never touches the data layer directly. The `#[server]` function is the boundary: on
the client build it is replaced by a network call; on the server build it runs its body. The
REST routes under `/api` exist for other clients (curl, the device); the web app does not use
them.

## Rules

1. **Server boundary.** The browser reaches data only through a `#[server]` function. No
   client-side calls to the REST routes.
2. **Server function placement and shape.** `#[server]` functions live in
   `server/api/<feature>.rs`. They are concrete — no generic type parameters, because each is
   registered as a fixed endpoint — and return `Result<T, ServerFnError>`. `ServerFnError` is
   the only error type permitted to cross the boundary; `AppError`, `sqlx::Error` and other
   server-only errors are mapped to it inside the function (`server_fn_error`) and must not
   appear in a hook or component type.
3. **Server-only imports go inside the function body.** The `#[server]` macro gates only the
   function _body_ for the server build — not the `use` statements beside it. A server-only
   import (`api`, `sqlx`, `crate::server::{app_state, server_fn_error}`, ...) at module scope
   in a file that also compiles on the client will fail the client build. Put such imports
   inside the `#[server]` body, or gate them with `#[cfg(feature = "server")]`. Types that
   appear in the function _signature_ (e.g. `Recipe`, `ServerFnError`) are the exception: they
   exist in both builds and belong at module scope.
4. **Thin server functions.** A server function takes the state, calls one repository
   or service function, and maps the error. Logic that both the REST handler and the server function
   need — a transaction spanning tables, resolving an inline lookup value — belongs in the
   `api` crate's repository, not in the server function.
5. **Hooks call server functions, not the data layer.** A feature hook imports its `#[server]`
   function; it must not import `api` or anything under `server/` other than `server/api`.
6. **Type safety over generics.** The web app favours explicit per-resource `#[server]`
   functions and typed hooks over a single generic proxy, accepting the extra files as the cost
   of end-to-end type checking. See
   [decisions/0001-typed-server-functions-over-generic-proxy.md](../decisions/0001-typed-server-functions-over-generic-proxy.md).

## Adding a feature

1. Create `features/<feature>/` with `mod.rs` and `hooks/`, plus `page.rs` if the feature has
   a screen.
2. Add a `#[server]` function per data call in `server/api/<feature>.rs`, calling the feature's
   repository in `api/src/features/<feature>/repository.rs` (declare the module `pub` in the
   feature's `mod.rs` if it is not yet).
3. Add a hook per call in `features/<feature>/hooks/` that awaits the server function.
4. If the feature has a page, register it in `router.rs`.
