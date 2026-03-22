#!/bin/sh
set -eu
echo "Running migrations..."
migrate
if [ "${SEED_DEV_DATA:-}" = "true" ]; then
    echo "Seeding dev data..."
    seed
    echo "Dev data seeded."
fi
echo "Starting API server..."
exec rest-api
