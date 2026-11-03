# Reaching the app from your phone (Tailscale)

Temporary setup for using Meal Roulette from a phone while the app runs on the home PC. The app
is reachable only from devices signed in to your own tailnet. It is not exposed to the internet.

This uses `tailscale serve`, which proxies a port on the PC to the tailnet. It does not use
`tailscale funnel` (the public-internet variant) and nothing in the repo changes the tailnet's
access controls.

## Install

1. Install Tailscale on the home PC (https://tailscale.com/download) and sign in.
2. Install Tailscale on the phone and sign in to the same account.
3. In the Tailscale admin console, check that HTTPS certificates are enabled for the tailnet
   (DNS page, "HTTPS Certificates"). `tailscale serve` prints a link to enable it on first use.

## Turn it on

Start the database and the web app as usual (`docker compose up -d`, then `cd web && dx serve`),
then in another terminal:

```
just tailnet-on
```

This runs `tailscale serve --bg 8080`, which forwards `https://<pc-name>.<tailnet-name>.ts.net`
to `http://localhost:8080`, and prints the status including that URL.

The web server keeps listening on localhost only. `tailscale serve` runs on the same machine
and connects to it locally, so no listen-address change is needed.

## Open it on the phone

1. Make sure the Tailscale app on the phone is connected.
2. Open the `https://<pc-name>.<tailnet-name>.ts.net` URL from `just tailnet-on` in the phone's
   browser. Optionally add it to the home screen.

The PC must be on, awake, and running `dx serve` for the page to load.

## Turn it off

```
just tailnet-off
```

This runs `tailscale serve --https=443 off`, which stops forwarding. Check with
`tailscale serve status`.

## Removing this

When the setup is no longer needed:

1. Run `just tailnet-off` (or `tailscale serve reset` to clear all serve config).
2. Delete `docs/guides/tailscale.md`.
3. Delete the `tailnet-on` and `tailnet-off` recipes (and their comments) from the `justfile`.
4. Optionally uninstall Tailscale from the PC and phone.

Nothing else in the repo references this setup.
