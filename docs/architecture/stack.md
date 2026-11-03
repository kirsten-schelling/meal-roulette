# Meal Roulette — Stack & Engineering Notes

> Companion to [the roadmap](../product/roadmap.md) (which covers _features_). This captures the
> **tech-stack decisions, the frontend exploration, the data/auth architecture, and the
> "for fun" C/C++ side-projects** — so nothing from the planning discussions gets lost.

## Guiding principles

- Personal, learning-oriented project. The real goal is to **deepen Rust on the path to embedded**.
- **UI design stays low-effort** — visual styling and CSS are not of interest and are delegated rather than studied. "Chosen by taste, not treated as a learning vehicle" applies to _how the app looks_, not to the frontend code.
- Type safety is non-negotiable; prefer compile-time errors over runtime.

---

## Current stack (decided)

### Backend

- **Rust** — Axum 0.8 + SQLx 0.9 (features: `postgres`, `macros`, `migrate`, `uuid`, `time`, `rust_decimal`, `json`, `runtime-tokio`, `tls-rustls`).
- One root Cargo **workspace**: `api` (a library: state, error mapping, repositories and the REST
  router, plus a thin standalone binary) + `domain` (wire types shared api ↔ web ↔ future device)
  - `web` (the Dioxus app, whose server build embeds `api`) + `solver` (the meal roller written
    in **C**, called via FFI — see C/C++ projects).
- Feature-oriented modules: `features/<x>/{ mod.rs = routes, handlers.rs, repository.rs }`;
  request/response types live in the `domain` crate. `roll` is the exception: `mod.rs` (routes:
  `POST /roll`, `POST /planned-meals/{id}/reroll`), `handlers.rs` and `service.rs`, with no
  repository of its own. The service orchestrates other features' repositories and the solver.
- Logic that both the REST handlers and the web app's `#[server]` functions need (a transaction
  spanning tables, resolving an inline lookup value) lives in the repository, so both stay thin
  adapters over the same functions.
- **UUIDv7** primary keys (`DEFAULT uuidv7()`, needs Postgres 18), plus `created_at` / `updated_at` with a shared `set_updated_at` trigger.
- **SQL migrations** via `sqlx-cli` (`api/migrations`, reversible with `-r`). Migrations are the schema source of truth — never edit one that's been applied; add a new one.
- **Only the checked macros** `query_as!` / `query_scalar!` are allowed — `clippy.toml` (`disallowed-methods` + `disallowed-macros`) bans `sqlx::query`, `sqlx::query_as` and `query!`.

### Database

- **Postgres 18** locally via Docker (`docker compose`). Note: PG18's image mounts the volume at `/var/lib/postgresql` (not `/data`).
- **Neon** intended for prod (serverless, scale-to-zero, branch-per-PR).
- `DATABASE_URL` lives in the root `.env` — feeds both `sqlx-cli` and the compile-time query macros.

### Data pattern — one process, server functions over the data layer

- The Dioxus server (Axum, `:8080`) is the only server. Its `#[server]` functions call
  `api::features::<x>::repository` directly; there is no HTTP hop to a separate API.
- The REST router (`api::app::build`) is mounted in the same process under `/api` for curl and
  the device / simulator. The browser never calls it; it goes through server functions only.
- **Reads** wrap a server function in `use_resource`; **writes** call a server function, then
  refresh. No form actions, no client data libs (TanStack Query stays an option if client
  caching is wanted later).
- Rationale, and the criteria for splitting into a second process again:
  [decisions/0002-single-process.md](../decisions/0002-single-process.md).

### Auth (planned) — one layer, session cookie

- The Dioxus server is the **OIDC client**: it runs the login flow, holds the tokens, and sets an
  **httpOnly session cookie**. Tokens never reach JS.
- Every server function runs behind that cookie; the server validates the session and passes the
  household scope down into the repositories.
- The resource-server half of the token-handler pattern (JWT validation against the issuer's
  JWKS via `jsonwebtoken` + JWKS, or a crate like `jwt-authorizer`) is only needed if an outside
  client — the device — calls the REST routes under `/api`.
- OIDC client libs to evaluate: Rust crates, since the client is the Rust server (e.g. `openidconnect`, `axum-oidc`, `oauth2`).

### Infra (intended)

- **Neon** (DB) + **Fly.io** (one app: the web server) + **Pulumi** (IaC, against Neon + Cloudflare). Cheap and transferable.

### Tooling installed on this machine

- Rust (rustup, MSVC toolchain), Docker, **just**, **sqlx-cli**, **dioxus-cli** (`dx`).
- Task recipes live in the `justfile` — run `just` to list them.

---

## Frontend exploration

Same recipes page (table + add form) built several ways to decide by _taste_:

| Stack                          | Verdict / notes                                                                                                                                                                                                              |
| ------------------------------ | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| SvelteKit + shadcn-svelte      | Neutral, composable, you own the components. Plain out of the box.                                                                                                                                                           |
| SvelteKit + DaisyUI            | Opinionated, themed, **nice out of the box**; one `data-theme` swaps the whole look.                                                                                                                                         |
| Next 16 + Chakra v3            | Built. Renders, but has an SSR hydration warning to fix. The work-familiar stack.                                                                                                                                            |
| Next 16 + Mantine / AntD / MUI | **Planned, not built** — each needs its own Next-16 SSR wiring; paused as low-value grind.                                                                                                                                   |
| **Dioxus 0.7** (Rust → WASM)   | **Chosen.** React-like, one codebase → web/desktop/mobile, `#[server]` fns. No OOB components: the app uses its own `web/assets/primitives.css` ("Warm" theme tokens) and hand-built primitives in `web/src/components/ui/`. |

**Status: DECIDED → Dioxus** (fullstack, `#[server]` functions), now the main app in **`web/`**. Trade-off accepted: **no ready-made component library → build your own** (which is wanted anyway).

**Final layout:** `web/` = **Dioxus** is the only frontend. The SvelteKit and Next spikes (`web2/`, `web3/`) were deleted.

`web/` module structure: see [web.md](web.md) (normative). Run with `dx serve` → localhost:8080.

### Leptos vs Dioxus (if the frontend goes all-Rust)

- Both compile Rust → WASM, both have `#[server]` functions = an elegant **all-Rust BFF on Axum** with natively shared types (no OpenAPI codegen).
- **Dioxus** — React-like (hooks, `rsx!`, components), **one codebase → web/desktop/mobile**, VDOM. Best fit given the React background + multi-surface roadmap. ← chosen.
- **Leptos** — fine-grained signals (SolidJS-like), fastest, most mature fullstack — but a _new, non-React_ reactivity model to learn.
- Shared downside: immature component ecosystems → build your own, or lean on CSS frameworks (Pico, DaisyUI) / headless kits (`dioxus-primitives`).

---

## "For fun" C/C++ side-projects (embedded practice)

Goal: get real C/C++ reps (embedded hiring still expects them), done _coherently_ inside this project rather than as throwaway toys.

1. **Wire-format codec in portable C** — _(best starting point)_
   Encode/decode the "week display" byte payload sent to the ESP32. The **same C file** is used by the **Rust backend (via FFI)** and the **firmware**. Teaches embedded-C discipline (fixed `uint8_t` buffers, no `malloc`, struct packing, endianness, bounds checks) **plus** the C↔Rust FFI boundary. Fully TDD-able on the host (no hardware).
   Tooling: `cc` crate (build C from `build.rs`), `bindgen` (C header → Rust), `cbindgen` (Rust → C header).
2. **Kitchen-screen firmware in C++ / ESP-IDF** — _the résumé-credible embedded C++._
   Write portable display/render logic that compiles for the ESP32 **and** a host-side simulator (fake framebuffer → terminal) so it can be TDD'd without hardware. Real hardware + partner's electronics input; a fair way off (see below).
3. **Meal-roulette solver in C — server-side, in the API** _(decided direction)._
   Write the roller/constraint-solver in **portable C** in a dedicated solver crate, called from a `/roll` route via FFI. Recipes come from Postgres, cross the boundary as plain arrays, chosen indices come back. **Server-gated inside the one process, as a dependency of the `api` library; it never reaches the wasm build.**
   ```
   solver/
     build.rs               # compiles csrc/solver.c via the `cc` crate
     csrc/{solver.c, solver.h}
     src/lib.rs             # unsafe extern "C" bindings + safe roll() wrapper + tests
   ```
   Status: built. `solver.c` is a backtracking CSP search with its own seeded PRNG (splitmix64), so a seed reproduces a week on any platform. Per-day limits (`SlotLimit`: day, column, low, high) restrict which recipes a given day may take; the solver fills the most restricted days first and only shuffles days that allow the same recipes. Rust owns every buffer that crosses the boundary; C allocates only internal scratch space (`calloc`, freed on every path). Teaches C discipline + struct layout + the C↔Rust ABI + host-side tests.
   _(A client-side `C → WASM` version was considered but **dropped** — it'd force shipping the whole recipe dataset to the browser. Only worth it if offline rolls ever become a goal.)_
4. **(Stretch) C++ + OpenCV CLI** for receipt-OCR preprocessing (deskew / threshold before OCR) — ties into a ROADMAP feature.

---

## Kitchen screen (future)

- Firmware will be **C/C++ (ESP-IDF or Arduino)** — confirmed.
- Needs hardware + design input from partner (electronics engineer). **A fair way off** — not a near-term item.
- Display choice (from ROADMAP): e-ink for a passive "this week" panel vs a small TFT / repurposed tablet for interactive cook-mode.
- Decodes the backend payload using the shared **wire-format codec** (side-project #1).

---

## Open decisions

- OIDC client library (a Rust crate: `openidconnect` / `axum-oidc` / `oauth2`).
- When to start the **auth** layer.
- The full queue lives in [../decisions/pending.md](../decisions/pending.md).

_(Resolved: frontend = **Dioxus fullstack** (`#[server]` functions over the in-process data layer, [0002](../decisions/0002-single-process.md)); meal-roller = **C in the `api` library** via FFI, not client-side WASM.)_
