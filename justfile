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

# run the API  ->  http://localhost:3000
api:
    cargo run -p api

# run the web app  ->  http://localhost:8080
web:
    cd web && dx serve

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

# format Rust
fmt:
    cargo fmt

# lint: fmt check + clippy (fails on warnings, enforces query_as!)
lint:
    cargo fmt --check
    cargo clippy --all-targets -- -D warnings

# type-check everything without producing binaries
check:
    cargo check --workspace
