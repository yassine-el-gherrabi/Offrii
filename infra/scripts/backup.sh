#!/usr/bin/env bash
# Offrii production database backup script
# Usage: ./backup.sh
# Cron:  0 3 * * * /opt/offrii/infra/scripts/backup.sh >> /var/log/offrii-backup.log 2>&1
set -euo pipefail

BACKUP_DIR="/opt/offrii/backups"
RETENTION_DAYS=14
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_FILE="${BACKUP_DIR}/offrii_${TIMESTAMP}.sql.gz"

# Ensure backup directory exists
mkdir -p "$BACKUP_DIR"

echo "=== Offrii Backup — $(date) ==="

# Dump database from running container
echo "Dumping database..."
docker exec offrii-postgres pg_dump \
  -U "${POSTGRES_USER:-offrii}" \
  -d "${POSTGRES_DB:-offrii}" \
  --format=plain \
  --no-owner \
  --no-privileges \
  | gzip > "$BACKUP_FILE"

BACKUP_SIZE=$(du -h "$BACKUP_FILE" | cut -f1)
echo "Backup created: $BACKUP_FILE ($BACKUP_SIZE)"

# Prune old backups
echo "Pruning backups older than ${RETENTION_DAYS} days..."
DELETED=$(find "$BACKUP_DIR" -name "offrii_*.sql.gz" -mtime +"$RETENTION_DAYS" -print -delete | wc -l)
echo "Deleted $DELETED old backup(s)."

# Summary
TOTAL_COUNT=$(find "$BACKUP_DIR" -name "offrii_*.sql.gz" | wc -l)
TOTAL_SIZE=$(du -sh "$BACKUP_DIR" | cut -f1)
echo "Backups on disk: $TOTAL_COUNT files, $TOTAL_SIZE total"

# Uncomment when rclone is configured for off-site backup:
# echo "Uploading to remote storage..."
# rclone copy "$BACKUP_FILE" offrii-backup:offrii-backups/ --progress
# echo "Upload complete."

echo "=== Backup finished ==="
