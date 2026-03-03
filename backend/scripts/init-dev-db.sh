#!/bin/sh
set -e

# Apply schema migrations in order
for f in /migrations/*.sql; do
    echo "Applying migration: $f"
    psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$POSTGRES_DB" -f "$f"
done

# Apply dev seed data
for f in /seeds/*.sql; do
    echo "Applying seed: $f"
    psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$POSTGRES_DB" -f "$f"
done
