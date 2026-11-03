# Deploying (tailnet-only hosting)

One always-on Fly.io app, reachable only from devices on your Tailscale network (tailnet). No
login, no public IP, no public DNS. Database: Neon Postgres managed by Pulumi.

## How it works

- The `Dockerfile` ships the server, `tailscaled`, `sqlx`, `api/migrations`.
- `tailscaled` (userspace mode) keeps its identity on a Fly volume at `/data`.
- `deploy/start.sh` joins the tailnet as `meal-roulette` with `TS_AUTHKEY`, then runs
  `tailscale serve --bg 8080`, publishing `https://meal-roulette.<tailnet>.ts.net`.
- The web server binds `127.0.0.1:8080`; only `tailscale serve` reaches it.
- `fly.toml` has no `http_service` or `services`: no public IP, and the machine is never auto-stopped.
- Migrations run as the release command in a temporary machine.

## 1. Accounts

1. **Fly.io**: sign up at https://fly.io, add a payment method, install `flyctl`, `fly auth login`.
2. **Neon**: sign up at https://neon.tech, create an API key.
3. **Tailscale**: enable MagicDNS and "HTTPS Certificates" (admin console, DNS page).
4. **Pulumi**: sign up for Pulumi Cloud. Install `pulumi` and Node 24.

## 2. Tailscale auth key

Tag the node so its key does not expire with a user:

1. Access controls: add `"tagOwners": { "tag:meal-roulette": ["autogroup:admin"] }`.
2. Settings, Keys, "Generate auth key": tick **Reusable**, leave **Ephemeral** off, tick
   **Pre-approved** if device approval is on, add tag `tag:meal-roulette`.
3. Copy the key (`tskey-auth-...`). Secret.

Reusable survives volume recreation; ephemeral deletes the node offline.

## 3. Neon (Pulumi)

```
cd infra
npm install
pulumi install
pulumi stack init prod
pulumi config set neonOrgId <org id from Neon: Organization settings>
export NEON_API_KEY=<the Neon API key>
pulumi preview
pulumi up
pulumi stack output databaseUrl --show-secrets          # pooled, for the app
pulumi stack output migrateDatabaseUrl --show-secrets   # direct, for migrations
```

`infra/sdks/` and `Pulumi.<stack>.yaml` are gitignored.

Config `neonRegion` (default `aws-ap-southeast-2`) should be near the Fly
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

Volume name and region must match `fly.toml` (`[mounts].source`, `primary_region`). Use
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

A fresh app lists none; release any listed. Confirm `https://meal-roulette.fly.dev` does not load off the tailnet.

## 7. Open on the phone

1. Connect Tailscale.
2. Find the tailnet name (admin console, DNS page).
3. Open `https://meal-roulette.<tailnet>.ts.net`.

If it fails to load, check `fly logs --app meal-roulette` for `tailscale up` and `tailscale serve` output, and the node in the admin console Machines list.

## Updating

- App: `fly deploy`. A page left open loads the new version on reload.
- Neon: edit `infra/index.ts`, run `pulumi up`.
- Tailscale key: `fly secrets set TS_AUTHKEY=...`.

## Local development

`just tailnet-on` ([tailscale.md](tailscale.md)) serves the PC's app to the tailnet under its
own name.
