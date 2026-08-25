# 1. Typed server functions over a generic proxy

- Status: Accepted
- Date: 2026-08-25

## Context

The web app is a backend-for-frontend (BFF): the browser calls the app's own server, which in
turn calls the Rust API. Dioxus exposes this boundary through `#[server]` functions, which must
be concrete — a `#[server]` function cannot be generic over its argument or return type, because
each one is registered as a fixed RPC endpoint with a fixed (de)serialisation.

Two ways to structure the boundary were considered:

1. **A single generic proxy.** One `#[server]` function that forwards an arbitrary route
   (concrete `String` in, raw JSON `String` out), with a generic client-side hook
   (`use_get<T>`) deserialising the response. One server function serves every read.
2. **Per-resource typed functions.** One concrete `#[server]` function per backend call
   (e.g. `get_recipes() -> Result<Vec<Recipe>, ServerFnError>`), each with a matching typed
   hook.

## Decision

Use **per-resource typed `#[server]` functions** with typed hooks. The generic proxy is not
used.

## Consequences

- **Gained: end-to-end type checking.** Each endpoint's request and response types are known at
  compile time on both sides; a shape change is caught by the compiler, not at runtime.
- **Gained: per-endpoint control.** Auth scopes, validation, and caching can be added to an
  individual function without a shared choke point.
- **Gained: no open-proxy surface.** A generic proxy that forwards a client-supplied route is a
  server-side request forgery (SSRF) risk and would need a route allowlist. Per-resource
  functions expose only the calls that exist.
- **Accepted cost: boilerplate.** Every backend call needs a server function and a hook. This is
  the deliberate trade — type safety is valued over brevity (see
  [../architecture/stack.md](../architecture/stack.md), "Guiding principles").
- Server functions and their layout follow [../architecture/web.md](../architecture/web.md).
