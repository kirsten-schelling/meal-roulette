# Web app module structure

The **web app** is the Dioxus 0.7 fullstack application in `web/`. This document defines the
required module layout, the responsibility of each layer, and the rules that govern the
client/server boundary. It is normative: new code under `web/` is expected to follow it.

The `recipe` feature is the reference implementation.

## Build model

The web app compiles from one source tree into **two binaries**:

- **Client build** — WebAssembly, runs in the browser. Enabled by the `web` feature.
- **Server build** — native, runs the backend-for-frontend (BFF). Enabled by the `server` feature.

Every component and most modules are compiled into *both*. Server-only code is compiled into
the server build alone, gated with `#[cfg(feature = "server")]`.

## Layout

```
web/src/
├── main.rs                     # entry point, launch, root App component
├── router.rs                   # Route enum: URL <-> page component
├── models.rs                   # small view helpers (wire types come from the domain crate)
├── showcase.rs                 # temporary /showcase component gallery
│
├── components/                 # GENERIC, cross-feature UI only
│   ├── ui/                     #   primitives (button, card, input, ...)
│   ├── layout/                 #   app shell (header, sidebar, app_layout)
│   └── stub.rs                 #   shared placeholders
│
├── pages/                      # shell route stubs not yet wired to a feature
│                               #   (their fate: docs/decisions/pending.md)
├── hooks/                      # GENERIC, cross-feature client hooks (reserved)
│
├── server/                     # SERVER-SIDE boundary
│   ├── api/                    #   #[server] functions (the BFF RPC surface)
│   │   └── <feature>.rs
│   └── fetcher.rs              #   #[cfg(server)] HTTP plumbing to the backend API
│
└── features/                   # one folder per feature
    └── <feature>/
        ├── page.rs             #   route-target component
        ├── hooks/              #   client data hooks (wrap use_resource)
        └── components/         #   feature-specific UI
```

## Responsibilities

- **`features/<feature>/page.rs`** — the component a route renders. Composes UI and reads data
  from the feature's hooks. No HTTP, no business logic.
- **`features/<feature>/hooks/`** — client-side data hooks (`use_*`) that wrap `use_resource`
  and call a `#[server]` function. They own the route-to-type mapping.
- **`features/<feature>/components/`** — UI used only by this feature.
- **`components/`** — UI reusable by more than one feature. Litmus test: if a second feature
  could use it, it lives here; otherwise it lives in the feature.
- **`server/api/<feature>.rs`** — the `#[server]` functions for that feature. The only surface
  the browser calls.
- **`server/fetcher.rs`** — server-only HTTP client used *inside* `#[server]` bodies to reach
  the backend API. Gated `#[cfg(feature = "server")]`.
- **`hooks/`** — generic client hooks shared across features. Reserved.

## Data flow

```
page.rs  ->  feature hook  ->  #[server] fn     ->  fetcher       ->  backend API
(browser)    (browser)         (RPC -> server)      (server only)     (Axum)
```

The browser never calls the backend directly. The `#[server]` function is the boundary: on the
client build it is replaced by a network call; on the server build it runs its body.

## Rules

1. **BFF boundary.** The browser reaches the backend only through a `#[server]` function. No
   direct client-to-backend requests.
2. **Server function placement and shape.** `#[server]` functions live in
   `server/api/<feature>.rs`. They are concrete — no generic type parameters, because each is
   registered as a fixed endpoint — and return `Result<T, ServerFnError>`. `ServerFnError` is
   the only error type permitted to cross the boundary; `reqwest::Error` and other server-only
   errors are mapped to it inside the function and must not appear in a hook or component type.
3. **Server-only imports go inside the function body.** The `#[server]` macro gates only the
   function *body* for the server build — not the `use` statements beside it. A server-only
   import (`fetcher`, `reqwest`, `sqlx`, ...) at module scope in a file that also compiles on
   the client will fail the client build. Put such imports inside the `#[server]` body, or gate
   them with `#[cfg(feature = "server")]`. Types that appear in the function *signature*
   (e.g. `Recipe`, `ServerFnError`) are the exception: they exist in both builds and belong at
   module scope.
4. **Hooks call server functions, not the fetcher.** A feature hook imports its `#[server]`
   function; it must not import `server::fetcher`.
5. **Type safety over generics.** The web app favours explicit per-resource `#[server]`
   functions and typed hooks over a single generic proxy, accepting the extra files as the cost
   of end-to-end type checking. See
   [decisions/0001-typed-server-functions-over-generic-proxy.md](../decisions/0001-typed-server-functions-over-generic-proxy.md).

## Adding a feature

1. Create `features/<feature>/` with `mod.rs`, `page.rs`, and `hooks/`.
2. Add a `#[server]` function per backend call in `server/api/<feature>.rs`.
3. Add a hook per call in `features/<feature>/hooks/` that awaits the server function.
4. Register the page in `router.rs`.
