import * as pulumi from "@pulumi/pulumi";
import * as cloudflare from "@pulumi/cloudflare";
import * as neon from "@pulumi/neon";

const config = new pulumi.Config();
const flyApp = config.get("flyApp") ?? "meal-roulette";
const hostname = config.require("hostname");
const zoneId = config.require("cloudflareZoneId");
const neonRegion = config.get("neonRegion") ?? "aws-ap-southeast-2";

const project = new neon.Project("meal-roulette", {
  name: "meal-roulette",
  regionId: neonRegion,
  pgVersion: 18,
});

// Fly terminates TLS with its own certificate, so the record must not be proxied.
new cloudflare.DnsRecord("app", {
  zoneId,
  name: hostname,
  type: "CNAME",
  content: `${flyApp}.fly.dev`,
  ttl: 1,
  proxied: false,
});

export const databaseUrl = pulumi.secret(project.connectionUri);
