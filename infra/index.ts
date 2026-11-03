import * as pulumi from "@pulumi/pulumi";
import * as neon from "@pulumi/neon";

const config = new pulumi.Config();
const neonRegion = config.get("neonRegion") ?? "aws-ap-southeast-2";
// Neon refuses to create a project without the organization that will own it.
const neonOrgId = config.require("neonOrgId");

const project = new neon.Project("meal-roulette", {
  name: "meal-roulette",
  orgId: neonOrgId,
  // The provider asks for 24 hours of point-in-time history by default; the free plan allows 6.
  historyRetentionSeconds: 21600,
  regionId: neonRegion,
  pgVersion: 18,
  branch: {
    name: "main",
  },
});

// Neon's pooler runs PgBouncer in transaction mode, which suits the app's many short
// connections but not `sqlx migrate`, which holds a session-level advisory lock.
export const databaseUrl = pulumi.secret(project.connectionUriPooler);
export const migrateDatabaseUrl = pulumi.secret(project.connectionUri);
