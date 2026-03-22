#!/bin/sh
set -eu

# Start infra only (Postgres + Redis)
docker compose up -d postgres redis

# Wait for Postgres to be ready
printf "Waiting for Postgres..."
until docker compose exec -T postgres pg_isready -U "${POSTGRES_USER:-offrii}" > /dev/null 2>&1; do
  printf "."
  sleep 1
done
echo " ready"

# Run migrations then start backend with hot-reload
cd backend
cargo run --bin migrate
exec cargo watch -x 'run --bin rest-api'
