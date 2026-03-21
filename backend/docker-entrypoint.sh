#!/bin/sh
set -eu
echo "Running migrations..."
migrate
if [ "${SEED_DEV_DATA:-}" = "true" ]; then
    echo "Seeding dev data..."
    psql "$DATABASE_URL" -f /app/fixtures/dev_seed.sql
    echo "Dev data seeded."
fi
echo "Starting API server..."
exec rest-api
