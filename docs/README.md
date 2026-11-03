# Documentation

Reference and planning docs for Meal Roulette, organised by purpose.

| Area                             | What lives here                                                                 |
| -------------------------------- | ------------------------------------------------------------------------------- |
| [`architecture/`](architecture/) | How the system is built — stack decisions, data model, module structure.        |
| [`product/`](product/)           | What is being built and why — the feature roadmap.                              |
| [`decisions/`](decisions/)       | Architecture Decision Records (ADRs): one file per significant, dated decision. |
| [`guides/`](guides/)             | How-to guides for working in the repo.                                          |

## Index

- **Architecture**
  - [Stack & engineering notes](architecture/stack.md)
  - [Data model](architecture/data-model.md)
  - [Web app module structure](architecture/web.md)
- **Product**
  - [Roadmap & feature ideas](product/roadmap.md)
- **Decisions**
  - [0001 — Typed server functions over a generic proxy](decisions/0001-typed-server-functions-over-generic-proxy.md)
  - [0002 — One process for the web app and the data layer](decisions/0002-single-process.md)
  - [0003 — Simple units: one unit per ingredient, no conversions](decisions/0003-simple-units.md)
  - [Pending decisions](decisions/pending.md) — queue of decisions to make / to record
- **Guides**
  - [Local development](guides/local-dev.md)
  - [Deploying (tailnet-only hosting)](guides/deploy.md)

## Conventions

- One ADR per decision, numbered and dated. An accepted record is never rewritten — a later
  record supersedes it.
- Docs are standalone reference: written to be read on their own, not as a narrative of how
  they came to be.
