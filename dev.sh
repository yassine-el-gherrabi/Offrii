#!/usr/bin/env bash
#
# dev.sh — Start Offrii dev environment with auto-seed
#
# Usage: bash dev.sh
#
set -euo pipefail

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${CYAN}Starting Offrii dev environment...${NC}"

# Start all services
docker compose -f docker-compose.yml -f docker-compose.override.yml -f docker-compose.dev.yml up -d

# Wait for backend to be ready
echo -e "${YELLOW}Waiting for backend...${NC}"
for i in $(seq 1 30); do
  if curl -sf http://localhost:3000/health/live > /dev/null 2>&1; then
    echo -e "${GREEN}Backend ready${NC}"
    break
  fi
  if [ "$i" = "30" ]; then
    echo -e "${RED}Backend didn't start in 60s${NC}"
    echo "Check logs: docker compose logs backend"
    exit 1
  fi
  sleep 2
done

# Auto-seed if DB is empty
USER_COUNT=$(docker exec offrii-postgres psql -U offrii -d offrii -tAc "SELECT COUNT(*) FROM users;" 2>/dev/null | tr -d ' ' || echo "0")

if [ "$USER_COUNT" = "0" ]; then
  echo -e "${YELLOW}Empty database detected — seeding demo data...${NC}"
  bash "$(dirname "$0")/backend/rest-api/scripts/seed_demo.sh"
else
  echo -e "${GREEN}Database has $USER_COUNT users — skipping seed${NC}"
fi

echo ""
echo -e "${GREEN}Offrii dev ready!${NC}"
echo -e "  API: http://localhost:3000"
echo -e "  Login: yassine@demo.com / DemoPass123x"
echo -e "  Cold-start: newuser@demo.com / DemoPass123x"
echo ""
echo -e "  Re-seed: bash backend/rest-api/scripts/reseed.sh"
