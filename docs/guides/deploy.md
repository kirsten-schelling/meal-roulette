# Deploying (tailnet-only hosting)

Production is one always-on Fly.io app that is reachable only from devices signed in to your
Tailscale network (tailnet). There is no login, no public IP and no public DNS record: if a
device is not on the tailnet, the app does not exist for it. The database is a Neon Postgres
project managed by Pulumi. See [../architecture/stack.md](../architecture/stack.md) and
[../decisions/0002-single-process.md](../decisions/0002-single-process.md).

| Piece                            | Where it is defined           |
| -------------------------------- | ----------------------------- |
| Image (server, tailscaled, sqlx) | `Dockerfile` (repo root)      |
| Container startup                | `deploy/start.sh`             |
| Fly app and volume               | `fly.toml` (repo root)        |
| Neon project and branch          | `infra/` (Pulumi, TypeScript) |

## How it fits together

- The container runs `tailscaled` in userspace networking mode (no TUN device or extra
  privileges needed). Its identity is stored on a small Fly volume mounted at `/data`, so the
  machine keeps the same tailnet node across restarts and deploys.
- On start it joins the tailnet as `meal-roulette` using the `TS_AUTHKEY` secret, then runs
  `tailscale serve --bg 8080`, which publishes `https://meal-roulette.<tailnet>.ts.net` to the
  tailnet using the tailnet's HTTPS certificates.
- The web server (which also serves the REST routes under `/api`) binds `127.0.0.1:8080`
  inside the machine. `tailscale serve` connects to it locally, so nothing else can reach it.
- `fly.toml` declares no `http_service` or `services`, so Fly has nothing to route to the
  machine and allocates no public IP for the app. Without a service the machine is not
  auto-stopped, which is what keeps it always on.
- Migrations run as Fly's release command before each deploy goes live, in a temporary
  machine that has no network exposure. The image carries `sqlx` and `api/migrations` for this.
- The Pulumi TypeScript program exists because the Neon provider is a bridged Terraform
  provider and Pulumi generates a TypeScript SDK for it.

## 1. Create the accounts

1. **Fly.io**: sign up at https://fly.io and add a payment method. Install `flyctl` and run
   `fly auth login`.
2. **Neon**: sign up at https://neon.tech. In the console, create an API key
   (Account settings, API keys).
3. **Tailscale**: you already have a tailnet from the phone setup. Check in the admin console
   (DNS page) that MagicDNS and "HTTPS Certificates" are enabled.
4. **Pulumi**: sign up for Pulumi Cloud (free for individuals) or use a self-managed backend,
   and install the `pulumi` CLI and Node 24.

## 2. Tailscale auth key

The app joins the tailnet as a tagged device so its key does not expire with a person.

1. In the admin console, Access controls: add a tag owner for the tag, for example
   `"tagOwners": { "tag:meal-roulette": ["autogroup:admin"] }`.
2. Settings, Keys, "Generate auth key": tick **Reusable**, leave **Ephemeral** off, tick
   **Pre-approved** if device approval is on, and add the tag `tag:meal-roulette`.
3. Copy the key (starts with `tskey-auth-`). Treat it as a password.

Reusable matters because the key is used again whenever the machine's stored state is lost
(for example if the volume is recreated). Do not mark it ephemeral, or the node is deleted from
the tailnet whenever it goes offline.

## 3. Neon project and branch (Pulumi)

```
cd infra
npm install
pulumi install
pulumi stack init prod
export NEON_API_KEY=<the Neon API key>
pulumi preview
pulumi up
```

`pulumi install` generates the Neon SDK into `infra/sdks/` from the `packages` entry in
`Pulumi.yaml`; that folder and the per-stack config files are gitignored. Optional config:
`neonRegion` (default `aws-ap-southeast-2`; keep it near the Fly `primary_region`).

Two connection strings are exported as secrets:

```
pulumi stack output databaseUrl --show-secrets          # pooled, for the running app
pulumi stack output migrateDatabaseUrl --show-secrets   # direct, for migrations
```

The app uses the pooled string. Migrations use the direct string because `sqlx migrate` holds
a session-level advisory lock, which Neon's transaction-mode pooler does not support.

## 4. Fly app, volume and secrets

```
fly apps create meal-roulette
fly volumes create tailscale_state --size 1 --region syd --app meal-roulette
fly secrets set --app meal-roulette \
  DATABASE_URL="<databaseUrl>" \
  MIGRATE_DATABASE_URL="<migrateDatabaseUrl>" \
  TS_AUTHKEY="<tskey-auth-...>"
```

The volume name must match `[mounts].source` in `fly.toml`, and its region must match
`primary_region`. Use a single machine: a volume belongs to one machine, and two machines would
fight over one tailnet identity.

## 5. Deploy

```
fly deploy
```

The first build compiles the whole workspace and takes a while. When the release command
(migrations) succeeds, the machine starts, joins the tailnet and serves.

The image build runs `dx bundle` with `SQLX_OFFLINE=true`, so the committed `.sqlx` cache must
be current (`just prepare`) before deploying a change to a query.

## 6. Make sure nothing is public

Fly may have allocated public IPs for the app (for instance if an earlier version of this app
had a public service). List them and release any:

```
fly ips list --app meal-roulette
fly ips release <address> --app meal-roulette
```

With no services in `fly.toml`, a fresh app shows no addresses. Also check that
`https://meal-roulette.fly.dev` does not load from a device that is off the tailnet.

## 7. Open it on the phone

1. Connect the Tailscale app on the phone.
2. Find the tailnet name in the admin console (DNS page, for example `tail1234.ts.net`).
3. Open `https://meal-roulette.<tailnet>.ts.net` in the phone's browser. Optionally add it to
   the home screen.

If the page does not load, run `fly logs --app meal-roulette` and look for the `tailscale up`
and `tailscale serve` output, and check the device appears in the Tailscale machines list.

## Updating

Run `fly deploy` again from the repository root. Pending migrations run first, then the machine
is replaced and rejoins the tailnet with the same identity (stored on the volume). The
installed web app picks up the new version the next time it loads the page; close and reopen
it if it was left running.

To change Neon, edit `infra/index.ts` and run `pulumi up`. To rotate the Tailscale key, set a
new `TS_AUTHKEY` with `fly secrets set`.

## Local development

The local `just tailnet-on` setup ([tailscale.md](tailscale.md)) can stay for development: it
serves the app running on your PC to the tailnet under the PC's own name, separate from the
hosted `meal-roulette` node.
