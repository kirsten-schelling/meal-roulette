# Deploying (tailnet-only hosting)

One always-on Fly.io app, reachable only from devices on your Tailscale network (tailnet). No
login, no public IP, no public DNS. Database: Neon Postgres managed by Pulumi. See
[stack.md](../architecture/stack.md) and [0002](../decisions/0002-single-process.md).

## How it works

- `tailscaled` (userspace mode) keeps its identity on a Fly volume at `/data`.
- Start joins the tailnet as `meal-roulette` with `TS_AUTHKEY`, then runs
  `tailscale serve --bg 8080`, publishing `https://meal-roulette.<tailnet>.ts.net`.
- The web server (and REST under `/api`) binds `127.0.0.1:8080`; only `tailscale serve` reaches it.
- `fly.toml` has no `http_service` or `services`: no public IP, and the machine is never auto-stopped.
- Migrations run as the release command in a temporary machine with no network exposure.

## 1. Accounts

1. **Fly.io**: sign up at https://fly.io, add a payment method, install `flyctl`, `fly auth login`.
2. **Neon**: sign up at https://neon.tech, create an API key (Account settings, API keys).
3. **Tailscale**: enable MagicDNS and "HTTPS Certificates" (admin console, DNS page).
4. **Pulumi**: sign up for Pulumi Cloud (or self-manage a backend). Install `pulumi` and Node 24.

## 2. Tailscale auth key

1. Access controls: add `"tagOwners": { "tag:meal-roulette": ["autogroup:admin"] }`.
2. Settings, Keys, "Generate auth key": tick **Reusable**, leave **Ephemeral** off, tick
   **Pre-approved** if device approval is on, add tag `tag:meal-roulette`.
3. Copy the key (`tskey-auth-...`). Treat it as a password.

Reusable is required (reused if the volume is recreated). Ephemeral deletes the node when offline.

## 3. Neon (Pulumi)

```
cd infra
npm install
pulumi install
pulumi stack init prod
export NEON_API_KEY=<the Neon API key>
pulumi preview
pulumi up
pulumi stack output databaseUrl --show-secrets          # pooled, for the app
pulumi stack output migrateDatabaseUrl --show-secrets   # direct, for migrations
```

Optional config `neonRegion` (default `aws-ap-southeast-2`) should be near the Fly
`primary_region`. Migrations need the direct string: `sqlx migrate` takes a session-level
advisory lock that Neon's pooler does not support.

## 4. Fly app, volume, secrets

```
fly apps create meal-roulette
fly volumes create tailscale_state --size 1 --region syd --app meal-roulette
fly secrets set --app meal-roulette \
  DATABASE_URL="<databaseUrl>" \
  MIGRATE_DATABASE_URL="<migrateDatabaseUrl>" \
  TS_AUTHKEY="<tskey-auth-...>"
```

Volume name must match `[mounts].source` in `fly.toml`; region must match `primary_region`. Use
one machine: two would fight over one tailnet identity.

## 5. Deploy

```
fly deploy
```

The build runs `dx bundle` with `SQLX_OFFLINE=true`: run `just prepare` first when queries changed.

## 6. Confirm nothing is public

```
fly ips list --app meal-roulette
fly ips release <address> --app meal-roulette
```

Release any address listed. Confirm `https://meal-roulette.fly.dev` does not load from a device
off the tailnet.

## 7. Open on the phone

1. Connect the Tailscale app.
2. Find the tailnet name (admin console, DNS page, for example `tail1234.ts.net`).
3. Open `https://meal-roulette.<tailnet>.ts.net`.

If it fails to load, check `fly logs --app meal-roulette`.

## Updating

- App: `fly deploy`. Close and reopen a running installed web app to load it.
- Neon: edit `infra/index.ts`, run `pulumi up`.
- Tailscale key: `fly secrets set TS_AUTHKEY=...`.

## Local development

`just tailnet-on` ([tailscale.md](tailscale.md)) serves the PC's app to the tailnet under the
PC's own name, separate from the hosted node.
