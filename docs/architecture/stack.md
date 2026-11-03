# Meal Roulette: Stack & Engineering Notes

Companion to [the roadmap](../product/roadmap.md).

## Guiding principles

- Learning project: deepen Rust on the path to embedded. UI styling is delegated; frontend code is part of the learning.
- Prefer compile-time errors.

## Backend

- **Rust**: Axum 0.8 + SQLx 0.9 (features: `postgres`, `macros`, `migrate`, `uuid`, `time`, `rust_decimal`, `json`, `runtime-tokio`, `tls-rustls`).
- One Cargo **workspace**:
  - `api`: library (state, error mapping, repositories, REST router) plus a thin standalone binary.
  - `domain`: wire types shared by api, web and the future device.
  - `web`: Dioxus app; its server build embeds `api`.
  - `solver`: meal roller in **C** via FFI.
- Modules: `features/<x>/{ mod.rs = routes, handlers.rs, repository.rs }`; request/response types live in `domain`. `roll` has `mod.rs` (`POST /roll`, `POST /roll/preview`, `POST /roll/preview/reroll`, `POST /roll/keep`, `POST /roll/fit`, `POST /planned-meals/{id}/reroll`), `handlers.rs`, `service.rs`, no repository; the service orchestrates other repositories and the solver.
- Logic shared by REST handlers and `#[server]` functions (multi-table transactions, inline lookups) lives in the repository.
- **UUIDv7** primary keys (`DEFAULT uuidv7()`, needs Postgres 18), plus `created_at` / `updated_at` with a shared `set_updated_at` trigger.
- **Migrations** via `sqlx-cli` (`api/migrations`, reversible with `-r`) are the schema source of truth. Never edit an applied one; add a new one.
- Only the checked macros `query_as!` / `query_scalar!` are allowed; `clippy.toml` bans `sqlx::query`, `sqlx::query_as` and `query!`.

## Database

- **Postgres 18** locally via `docker compose`; its image mounts the volume at `/var/lib/postgresql` (not `/data`).
- **Neon** in prod (serverless, scale-to-zero).
- `DATABASE_URL` is in the root `.env`, feeding `sqlx-cli` and the compile-time macros.

## Data pattern: one process

- The Dioxus server (Axum, `:8080`) is the only server. `#[server]` functions call `api::features::<x>::repository` directly; no HTTP hop.
- The REST router (`api::app::build`) is mounted under `/api` in the same process for curl and the device. The browser never calls it.
- **Reads** wrap a server function in `use_resource`; **writes** call one, then refresh.
- Splitting criteria: [0002](../decisions/0002-single-process.md).

## Auth (planned): session cookie

- The Dioxus server is the **OIDC client**: it runs login, holds tokens, and sets an **httpOnly session cookie**. Tokens never reach JS.
- Server functions run behind that cookie, passing the household scope to repositories.
- JWT validation against the issuer's JWKS (`jsonwebtoken`, `jwt-authorizer`) is needed only if the device calls `/api`.

## Infra and tooling

- **Neon** (DB) + **Fly.io** (one app: the web server) + **Pulumi** (IaC, against Neon), reachable only over Tailscale ([deploy guide](../guides/deploy.md)).
- Rust (rustup, MSVC), Docker, **just** (`just` lists recipes), **sqlx-cli**, **dioxus-cli** (`dx`).

## Frontend

**Decided: Dioxus 0.7** (fullstack, Rust to WASM) in **`web/`**, the only frontend (the `web2/` and `web3/` spikes were deleted). Alternatives built or evaluated: SvelteKit (shadcn-svelte, DaisyUI), Next 16 (Chakra v3), Leptos (signals model, non-React). No ready-made components; it uses `web/assets/primitives.css` ("Warm" tokens) and primitives in `web/src/components/ui/`. Module structure: [web.md](web.md). Run with `dx serve`.

## C/C++ side-projects (embedded practice)

1. **Wire-format codec in portable C** (best starting point). Encodes/decodes the "week display" payload for the ESP32; the same C file serves the Rust backend (FFI) and the firmware. Teaches fixed buffers, no `malloc`, struct packing, endianness, bounds checks and FFI; TDD-able on the host. Tooling: `cc` crate, `bindgen`, `cbindgen`.
2. **Kitchen-screen firmware in C++ / ESP-IDF**, with a host-side simulator.
3. **Meal-roulette solver in C** (built). In the `solver` crate, called from `/roll` via FFI. Recipes cross as plain arrays; chosen indices come back. Server-only; never in the wasm build.
   ```
   solver/
     build.rs               # compiles csrc/solver.c via the `cc` crate
     csrc/{solver.c, solver.h}
     src/lib.rs             # unsafe extern "C" bindings + safe roll() wrapper + tests
   ```
   `solver.c` is a backtracking CSP search with a seeded PRNG (splitmix64), so a seed reproduces a week on any platform. Per-day limits (`SlotLimit`) restrict which recipes each day may take; the most restricted days fill first. Rust owns every buffer crossing the boundary; C allocates only internal scratch (`calloc`, freed on every path). Client-side C-to-WASM was dropped: it would ship all recipes to the browser.
4. **(Stretch)** C++ + OpenCV receipt-OCR preprocessing CLI.

## Kitchen screen (future)

- Firmware in C/C++ (ESP-IDF or Arduino); needs hardware and the partner's (electronics engineer) design input.
- Display: e-ink (passive "this week") vs small TFT / repurposed tablet (cook-mode).
- Decodes the payload with the wire-format codec.

## Open decisions

- OIDC client crate (`openidconnect`, `axum-oidc`, `oauth2`); when to start auth. Full queue: [pending.md](../decisions/pending.md).
