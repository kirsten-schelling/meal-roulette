# Deploying to production

Production is one Fly.io app (the web server, which also serves the REST routes under `/api`),
a Neon Postgres project, and a Cloudflare DNS record, with Neon and Cloudflare managed by
Pulumi. See [../architecture/stack.md](../architecture/stack.md) and
[../decisions/0002-single-process.md](../decisions/0002-single-process.md).

| Piece | Where it is defined |
| --- | --- |
| Web server image | `Dockerfile` (repo root) |
| Fly app | `fly.toml` (repo root) |
| Neon project, Cloudflare DNS record | `infra/` (Pulumi, TypeScript) |

The Pulumi program is TypeScript because the Cloudflare provider is first-class there and the
Neon provider is a bridged Terraform provider that Pulumi generates a TypeScript SDK for.

## Prerequisites

- Accounts: Neon, Fly.io, Cloudflare (with the domain's zone), and a Pulumi Cloud account or a
  self-managed backend.
- CLIs: `flyctl`, `pulumi`, Node 24, `sqlx-cli`.
- Credentials as environment variables, never committed: `NEON_API_KEY`, `CLOUDFLARE_API_TOKEN`
  (DNS edit on the zone).

## 1. Neon and DNS (Pulumi)

```
cd infra
npm install
pulumi install
pulumi stack init prod
pulumi config set hostname meal.example.com
pulumi config set cloudflareZoneId <zone id>
pulumi preview
pulumi up
```

`pulumi install` generates the Neon SDK into `infra/sdks/` from the `packages` entry in
`Pulumi.yaml`; that folder and the per-stack config files are gitignored. Optional config:
`flyApp` (default `meal-roulette`, must match `app` in `fly.toml`) and `neonRegion` (default
`aws-ap-southeast-2`; keep it near the Fly `primary_region`).

The connection string is exported as a secret: `pulumi stack output databaseUrl --show-secrets`.

## 2. Migrations

The server does not run migrations at startup. Apply them from your machine against Neon
before the first deploy and after any release that adds one:

```
sqlx migrate run --source api/migrations --database-url "<databaseUrl>"
```

## 3. Fly app

```
fly apps create meal-roulette
fly secrets set DATABASE_URL="<databaseUrl>"
fly deploy
fly certs add meal.example.com
```

`DATABASE_URL` is the only required runtime setting. The image sets `IP=0.0.0.0` and
`PORT=8080`, which Dioxus reads to pick its bind address, matching `internal_port` in
`fly.toml`. Machines stop when idle and start on the next request, so the first request after
a quiet period is slow.

The image build runs `dx bundle` with `SQLX_OFFLINE=true`, so the committed `.sqlx` cache must
be current (`just prepare`) before deploying a change to a query.

## Updating

Redeploy with `fly deploy`; change Neon or DNS by editing `infra/index.ts` and running
`pulumi up`.
