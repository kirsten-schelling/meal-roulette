# Meal Roulette — task runner. Run `just` to list recipes.

set windows-shell := ["cmd.exe", "/c"]

# list all recipes
default:
    @just --list

# start local Postgres (detached)
db:
    docker compose up -d

# stop local Postgres
db-down:
    docker compose down

# run the REST routes on their own (the web server already serves them under /api)  ->  http://localhost:3000
api:
    cargo run -p api

# build + run the C solver standalone with the host harness (see printf output without the API)
solver input="solver/input.txt":
    powershell -NoProfile -ExecutionPolicy Bypass -File solver\run.ps1 "{{input}}"

# run the web app: Dioxus + #[server] functions + REST under /api  ->  http://localhost:8080
web:
    cd web && dx serve

# run the app as a native desktop window (fast loop for the mobile code path; server must be running: just web)
desktop:
    cd web && dx serve --desktop --no-default-features --features desktop

# run the app on an Android emulator or device (see docs/guides/mobile.md)
android:
    cd web && dx serve --android --no-default-features --features mobile

# run pending migrations, then refresh the offline query cache (needs sqlx-cli + DATABASE_URL in .env)
migrate: && prepare
    sqlx migrate run --source api/migrations

# revert the most recent migration (runs its .down.sql)
migrate-down: && prepare
    sqlx migrate revert --source api/migrations

# create a reversible migration:  just new-migration add_servings
new-migration name:
    sqlx migrate add -r {{name}} --source api/migrations

# refresh the sqlx offline query cache in .sqlx (DB must be running)
prepare:
    cargo sqlx prepare --workspace -- --all-targets

# format Rust, then the rsx! blocks in the web app (kept out of lint: dx fmt --check rewrites files)
fmt:
    cargo fmt
    cd web && dx fmt

# lint: fmt check + clippy (fails on warnings, enforces query_as!) + prettier
lint:
    cargo fmt --check
    cargo clippy --all-targets -- -D warnings
    npx prettier@3.3.3 --check .

# type-check everything without producing binaries
check:
    cargo check --workspace
