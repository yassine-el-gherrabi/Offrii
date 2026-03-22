#!/usr/bin/env bash
#
# reseed.sh — Clean database and re-seed demo fixtures
#
# Usage: bash backend/rest-api/scripts/reseed.sh
#
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${YELLOW}Cleaning database...${NC}"
docker exec offrii-postgres psql -U offrii -d offrii -c "DROP SCHEMA public CASCADE; CREATE SCHEMA public;"

echo -e "${YELLOW}Restarting backend (re-runs migrations)...${NC}"
docker compose restart backend

# Wait for backend to be ready
for i in $(seq 1 30); do
  if curl -sf http://localhost:3000/health/live > /dev/null 2>&1; then
    echo -e "${GREEN}Backend ready${NC}"
    break
  fi
  if [ "$i" = "30" ]; then
    echo -e "${RED}Backend didn't start in 60s${NC}"
    exit 1
  fi
  sleep 2
done

echo -e "${YELLOW}Seeding demo data...${NC}"
bash "$SCRIPT_DIR/seed_demo.sh"
