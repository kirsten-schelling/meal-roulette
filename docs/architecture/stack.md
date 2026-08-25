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

- **Rust** — Axum 0.8 + SQLx 0.9 (features: `postgres`, `macros`, `migrate`, `uuid`, `time`, `runtime-tokio`, `tls-rustls`).
- Cargo **workspace**: `crates/api` (HTTP server) + `crates/domain` (types shared server ↔ future device) + _(planned)_ `crates/solver` (the meal roller written in **C**, called via FFI — see C/C++ projects).
- Feature-oriented modules: `features/<x>/{ mod.rs = routes, handlers, service, repo, model }`.
- **UUIDv7** primary keys (`DEFAULT uuidv7()`, needs Postgres 18), plus `created_at` / `updated_at` with a shared `set_updated_at` trigger.
- **SQL migrations** via `sqlx-cli` (`crates/api/migrations`, reversible with `-r`). Migrations are the schema source of truth — never edit one that's been applied; add a new one.
- **Only** `query_as!` **allowed**, enforced by `clippy.toml` (`disallowed-methods` + `disallowed-macros`) → no unchecked or anonymous queries.

### Database

- **Postgres 18** locally via Docker (`docker compose`). Note: PG18's image mounts the volume at `/var/lib/postgresql` (not `/data`).
- **Neon** intended for prod (serverless, scale-to-zero, branch-per-PR).
- `DATABASE_URL` lives in the root `.env` — feeds both `sqlx-cli` and the compile-time query macros.

### Data pattern — BFF, no form actions

- **Reads:** server-side (SvelteKit `load` / Next server component / Dioxus `#[server]`) fetch the Rust API **server-to-server**. The browser never calls the API directly.
- **Writes:** a **BFF endpoint** (SvelteKit `+server.ts`, Next route handler, or Dioxus `#[server]`) that the client calls with `fetch`, then refresh (`invalidateAll` / `router.refresh`).
- Deliberately **not** using SvelteKit form actions or client data libs (SWR/React-Query) — though TanStack Query stays an option if client caching is wanted later.

### Auth (planned) — BFF / token-handler pattern

- The web server is the **BFF** and the **OIDC client**: it runs the login flow, holds the tokens, and sets an **httpOnly session cookie**. Tokens never reach JS.
- Every read/write runs server-side and attaches the **access token**, calling the Rust API server-to-server.
- The **Rust API is the resource server**: it validates the JWT against the issuer's JWKS (`jsonwebtoken` + JWKS, or a crate like `jwt-authorizer`).
- OIDC client libs to evaluate: `openid-client` (most standards-compliant), **Auth.js** (`@auth/sveltekit`), **Better Auth**.
- This architecture is identical under Svelte / React / Dioxus, so it doesn't gate the frontend choice.

### Infra (intended)

- **Neon** (DB) + **Fly.io** (API) + **Pulumi** (IaC, against Neon + Cloudflare). Cheap and transferable.

### Tooling installed on this machine

- Rust (rustup, MSVC toolchain), Node 24 via **fnm** (pinned in `.node-version`), **yarn**, Docker, **just**, **sqlx-cli**, **dioxus-cli** (`dx`).
- `just` recipes: `db`, `migrate`, `new-migration <name>`, `api`, `web`, `fmt`, `lint`, `check`.

---

## Frontend exploration

Same recipes page (table + add form) built several ways to decide by _taste_:

| Where                                 | Stack                          | Verdict / notes                                                                                                                                     |
| ------------------------------------- | ------------------------------ | --------------------------------------------------------------------------------------------------------------------------------------------------- |
| `web/` → `/shadcn`                    | SvelteKit + shadcn-svelte      | Neutral, composable, you own the components. Plain out of the box.                                                                                  |
| `web/` → `/daisy`                     | SvelteKit + DaisyUI            | Opinionated, themed, **nice out of the box**; one `data-theme` swaps the whole look.                                                                |
| `web2/` → `/chakra`                   | Next 16 + Chakra v3            | Built. Renders, but has an SSR hydration warning to fix. The work-familiar stack.                                                                   |
| `web2/` → `/mantine`, `/antd`, `/mui` | Next 16 + Mantine / AntD / MUI | **Planned, not built** — each needs its own Next-16 SSR wiring; paused as low-value grind.                                                          |
| `web3/`                               | **Dioxus 0.7** (Rust → WASM)   | **Current lean.** React-like, one codebase → web/desktop/mobile, `#[server]` BFF. No OOB components (used Pico.css + hand-built `DifficultyBadge`). |

**Status: DECIDED → Dioxus** (fullstack, `#[server]` BFF), now the main app in **`web/`**. Trade-off accepted: **no ready-made component library → build your own** (which is wanted anyway). _(The `Where` paths in the table above are pre-decision; the folders were rotated afterwards — final layout below.)_

**Final layout:** `web/` = **Dioxus** (the app, committed) · `web2/` = SvelteKit · `web3/` = Next/React. `web2/` and `web3/` are kept as **gitignored reference spikes** (not built or committed) — delete anytime.

`web/` module structure: `main.rs` (launch + `App` composition) · `models.rs` (types) · `server.rs` (`#[server]` fns) · `components/` (`difficulty_badge`, `recipe_table`, `add_recipe_form`). Run with `dx serve` → localhost:8080.

### Leptos vs Dioxus (if the frontend goes all-Rust)

- Both compile Rust → WASM, both have `#[server]` functions = an elegant **all-Rust BFF on Axum** with natively shared types (no OpenAPI codegen).
- **Dioxus** — React-like (hooks, `rsx!`, components), **one codebase → web/desktop/mobile**, VDOM. Best fit given the React background + multi-surface roadmap. ← current lean.
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
   Write the roller/constraint-solver in **portable C** in a dedicated `crates/solver` crate, called from `crates/api`'s `/roll` endpoint via FFI. Recipes come from Postgres, cross the boundary as plain arrays, chosen indices come back. **Lives in the API backend (`:3000`), NOT the Dioxus web/BFF server.**
   ```
   crates/solver/
     build.rs               # compiles csrc/solver.c via the `cc` crate
     csrc/{solver.c, solver.h}
     src/lib.rs             # unsafe extern "C" bindings + safe roll() wrapper + tests
   ```
   Plan: **I scaffold the `cc`/`build.rs`/FFI plumbing + a passing test harness; you write `solver.c`.** Teaches C discipline (fixed buffers, no `malloc`, struct layout) + the C↔Rust ABI + host-side tests.
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

- OIDC client library (`openid-client` / Auth.js / Better Auth).
- When to start the **BFF + auth** layer.

_(Resolved: frontend = **Dioxus fullstack** (`#[server]` BFF); meal-roller = **C in the API** via FFI, not client-side WASM.)_
