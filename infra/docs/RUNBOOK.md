# Runbook Offrii

## Accès

| Service | URL | Auth |
|---------|-----|------|
| API prod | `https://api.offrii.com` | JWT Bearer |
| Swagger UI | `https://api.offrii.com/docs/` | `admin` / `OffriiDocs2026Prod` |
| Grafana | `https://grafana.offrii.com` | `admin` / (GF_SECURITY_ADMIN_PASSWORD) |
| Serveur SSH | `ssh deploy@167.235.193.237` | Clé SSH |

## Commandes courantes

### SSH au serveur

```bash
ssh deploy@167.235.193.237
cd /opt/offrii
```

### Voir les logs

```bash
# Logs backend (live)
docker logs -f offrii-backend

# Logs avec filtre
docker logs offrii-backend 2>&1 | grep "ERROR"

# Logs Caddy
docker logs -f offrii-caddy

# Logs PostgreSQL
docker logs -f offrii-postgres
```

### Statut des services

```bash
docker ps --format 'table {{.Names}}\t{{.Status}}\t{{.Ports}}'
```

### Health checks

```bash
# Backend
curl -sf http://localhost:3000/health/ready

# Depuis l'extérieur
curl -sf https://api.offrii.com/health/live
```

## Restart

### Restart un service

```bash
docker compose -f docker-compose.yml -f docker-compose.prod.yml restart backend
```

### Restart tout

```bash
docker compose -f docker-compose.yml -f docker-compose.prod.yml down
docker compose -f docker-compose.yml -f docker-compose.prod.yml up -d --no-build
```

## Rollback

### Rollback au deploy précédent

```bash
# Voir les images disponibles
docker images ghcr.io/yassine-el-gherrabi/offrii-api

# Retag l'ancienne image
docker tag ghcr.io/yassine-el-gherrabi/offrii-api:<sha-ancien> ghcr.io/yassine-el-gherrabi/offrii-api:latest

# Restart
docker compose -f docker-compose.yml -f docker-compose.prod.yml up -d --no-build
```

### Rollback une migration

```bash
# Accéder au container
docker exec -it offrii-backend sh

# Voir les migrations appliquées
migrate --source file:///app/migrations --database $DATABASE_URL version

# Rollback la dernière
migrate --source file:///app/migrations --database $DATABASE_URL down 1
```

## Base de données

### Accès psql

```bash
docker exec -it offrii-postgres psql -U offrii -d offrii
```

### Requêtes utiles

```sql
-- Nombre d'utilisateurs
SELECT COUNT(*) FROM users;

-- Connexions actives
SELECT count(*) FROM pg_stat_activity WHERE state = 'active';

-- Requêtes lentes (> 500ms, logguées par PostgreSQL)
-- Voir docker logs offrii-postgres | grep "duration"

-- Taille des tables
SELECT tablename, pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename))
FROM pg_tables WHERE schemaname = 'public' ORDER BY pg_total_relation_size(schemaname||'.'||tablename) DESC;
```

## Redis

### Accès redis-cli

```bash
docker exec -it offrii-redis redis-cli
```

### Commandes utiles

```bash
# Mémoire utilisée
INFO memory

# Nombre de clés
DBSIZE

# Evictions
INFO stats | grep evicted

# Vider le cache (ATTENTION)
FLUSHALL
```

## Diagnostic

### Checklist "le site est down"

1. **SSH marche ?** → `ssh deploy@167.235.193.237`
2. **Containers up ?** → `docker ps` — vérifier que backend, postgres, redis, caddy sont "healthy"
3. **Health check ?** → `curl http://localhost:3000/health/ready` — si DB ou Redis down, le health check le dit
4. **Logs ?** → `docker logs offrii-backend --tail 50` — chercher les erreurs
5. **Disk plein ?** → `df -h` — si > 90%, nettoyer les images Docker : `docker image prune -f`
6. **RAM ?** → `free -h` — si < 500MB libre, un container consomme trop
7. **CPU ?** → `top` ou `htop` — vérifier quel process consomme
8. **DNS ?** → `dig api.offrii.com` — doit pointer vers 167.235.193.237

### Checklist "les emails n'arrivent pas"

1. **Resend dashboard** → vérifier les envois récents
2. **SPF/DKIM** → `dig TXT offrii.com +short` — doit inclure `send.resend.com`
3. **Logs backend** → chercher "failed to send" dans les logs
4. **Spam** → vérifier le dossier spam du destinataire

### Checklist "l'app est lente"

1. **Grafana** → `https://grafana.offrii.com` → Dashboard host → CPU/RAM/disk
2. **PostgreSQL** → Dashboard PostgreSQL → connexions, cache hit ratio, requêtes lentes
3. **Redis** → Dashboard Redis → mémoire, evictions
4. **Logs** → chercher les requêtes > 1s dans les logs backend

## Rotation des clés

### JWT keys (urgence : clé compromise)

```bash
# 1. Générer une nouvelle paire
openssl genpkey -algorithm RSA -out new_private.pem -pkeyopt rsa_keygen_bits:2048
openssl rsa -in new_private.pem -pubout -out new_public.pem

# 2. Copier sur le serveur
scp new_private.pem deploy@167.235.193.237:/opt/offrii/secrets/jwt_private.pem
scp new_public.pem deploy@167.235.193.237:/opt/offrii/secrets/jwt_public.pem

# 3. Mettre à jour les secrets GitHub (base64)
base64 < new_private.pem | gh secret set JWT_PRIVATE_KEY_BASE64
base64 < new_public.pem | gh secret set JWT_PUBLIC_KEY_BASE64

# 4. Restart le backend (tous les users seront déconnectés)
docker compose -f docker-compose.yml -f docker-compose.prod.yml restart backend

# 5. Nettoyer
rm new_private.pem new_public.pem
```

### API keys (Resend, OpenAI, R2)

1. Régénérer la clé dans le dashboard du service
2. Mettre à jour dans GitHub Secrets
3. Relancer le deploy CD ou restart manuellement

## Contacts

- **Serveur** : Hetzner Cloud (dashboard.hetzner.cloud)
- **DNS** : Cloudflare (dash.cloudflare.com)
- **Emails** : Resend (resend.com/emails)
- **Domaine** : Namecheap (namecheap.com)
- **Images** : Cloudflare R2 (dash.cloudflare.com → R2)
