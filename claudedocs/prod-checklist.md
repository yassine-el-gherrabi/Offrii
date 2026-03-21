# Offrii Production Checklist

**Serveur** : Hetzner CX32 — 167.235.193.237
**Domaine** : api.offrii.com
**Stack** : Rust/Axum + PostgreSQL 18 + Redis 8 + Caddy + Prometheus/Grafana/Loki
**Deploiement** : 2026-03-20

---

## Prochaines actions

### Cloudflare Migration ✅
- [x] Créer compte Cloudflare (free) + ajouter domaine `offrii.com`
- [x] Vérifier import DNS (api, staging, grafana → 167.235.193.237 en DNS only)
- [x] Changer nameservers dans Namecheap → Cloudflare
- [x] Propagation NS Cloudflare confirmée
- [x] Activer custom domain R2 : `cdn.offrii.com` (proxied)
- [x] Activer Email Routing : `contact@offrii.com` → Gmail perso
- [x] Fusionner SPF records (Resend + Cloudflare)
- [x] Mettre à jour LOGO_URL dans le backend → `https://cdn.offrii.com/branding/logo-1024.png`
- [x] Mettre à jour R2_PUBLIC_URL dans les secrets GitHub (prod → cdn.offrii.com)
- [x] Testé `https://cdn.offrii.com/branding/logo-1024.png` → 200 OK
- [x] Testé réception email `contact@offrii.com` → forwarding Gmail OK

### R2 Bucket séparation staging/prod ✅
- [x] Créer bucket `offrii-media-staging` dans Cloudflare R2
- [x] Mettre à jour variable GitHub staging : `R2_BUCKET_NAME=offrii-media-staging`
- [x] Bucket prod reste `offrii-media` avec custom domain `cdn.offrii.com`

---

## Etat actuel

### Infrastructure ✅
- [x] Serveur provisionné (Hetzner CX32, 4 vCPU, 8GB RAM)
- [x] SSH hardened (key-only, root disabled, fail2ban)
- [x] UFW firewall (22/80/443)
- [x] Docker + Compose installé
- [x] DNS configuré (api.offrii.com → serveur)
- [x] Secrets copiés (/opt/offrii/.env, JWT keys, APNs key)

### Services ✅
- [x] Backend Rust/Axum — healthy, migrations OK
- [x] PostgreSQL 18 — healthy, tuned (256MB shared_buffers)
- [x] Redis 8 — healthy, LRU 128MB
- [x] Caddy — HTTPS auto (Let's Encrypt), rate limiting, security headers
- [x] Prometheus — scraping node-exporter
- [x] Grafana — running (admin password set)
- [x] Loki — running
- [x] Node Exporter — running

### Backups ✅
- [x] Hetzner snapshots automatiques (niveau serveur)
- [ ] ~Backup cron pg_dump~ — pas nécessaire pour le moment (Hetzner couvre)

---

## P0 — Bloquants / Securite

### GitHub Actions CD
- [x] Environments créés (prod + staging)
- [x] 10 secrets repo-level (DEPLOY_SSH_KEY, RESEND_API_KEY, R2_*, JWT_*, APNS_*, OPENAI_API_KEY, GOOGLE_CLIENT_ID)
- [x] 5 variables repo-level (APNS_KEY_ID, APNS_TEAM_ID, APNS_BUNDLE_ID, R2_BUCKET_NAME, R2_PUBLIC_URL)
- [x] 2 secrets par env (POSTGRES_PASSWORD, GF_SECURITY_ADMIN_PASSWORD)
- [x] 6 variables par env (HETZNER_HOST, POSTGRES_DB, POSTGRES_USER, RUST_LOG, APNS_SANDBOX, CORS_ORIGIN)
- [x] Workflow prod mis à jour — génère .env + décode clés au deploy
- [x] Workflow staging mis à jour — même pattern avec .env.staging
- [ ] Tester un deploy complet via `git push master` — CD en cours

### CI/CD Hardening (audit 2026-03-20)

#### Critical (avant launch)
- [x] `cargo audit` dans le CI (advisory, non-bloquant via continue-on-error)
- [x] TODO/FIXME check dans CI iOS (advisory, non-bloquant)
- [x] ~~Secret Scanning~~ → couvert par GitGuardian
- [x] Migration DB dans le deploy script (avant docker compose up, prod + staging)

#### Recommended (semaine 1)
- [x] SwiftLint : force_unwrap/force_cast/implicitly_unwrapped_optional (advisory CI check)
- [x] SwiftLint : seuils resserrés (line:140/200, file:400/800, type:400/700)
- [x] Dependabot auto-merge grouping (minor+patch regroupés)
- [x] Branch protection master (CI required, no force push, no PR review)
- [x] Branch protection develop (CI required, force push OK)
- [x] iOS coverage reporting (advisory)
- [ ] ~~Notifications deploy~~ — pas nécessaire en solo dev

#### Nice to have (mois 1-3)
- [ ] iOS matrix testing (multi-device/OS)
- [ ] Codecov badge README
- [ ] SARIF output cargo-audit → GitHub Security tab
- [ ] Performance benchmarks (cargo-criterion)
- [ ] API documentation validation (OpenAPI spec drift)

### Restreindre /docs (Swagger UI) ✅
- [x] basic_auth sur /docs et /api-doc → user: `admin` / pass: `OffriiDocs2026Prod`
- [x] CSP relaxé uniquement pour /docs (unsafe-inline pour Swagger UI JS)
- [x] API endpoints restent accessibles sans auth

### Restreindre Grafana ✅
- [x] grafana.offrii.com avec basic_auth + TLS (Let's Encrypt)
- [x] user: `admin` / pass: même que GF_SECURITY_ADMIN_PASSWORD
- [x] Double auth : Caddy basic_auth + Grafana login

---

## P1 — Observabilite (cette semaine)

### Grafana dashboards
- [ ] Dashboard host (CPU, RAM, disk, network) via node-exporter
- [ ] Dashboard backend (requests, latency, errors) — nécessite P1.3
- [ ] Dashboard PostgreSQL — nécessite postgres_exporter
- [ ] Dashboard Redis — nécessite redis_exporter

### Loki log streaming
- [ ] Configurer Docker logging driver → Loki
- [ ] Ou : deployer Promtail pour collecter les logs containers
- **Actuellement** : Loki tourne mais ne collecte rien

### Backend /metrics
- [ ] Ajouter endpoint Prometheus /metrics dans le backend Rust
- [ ] Configurer Prometheus pour scraper backend:3000/metrics
- [ ] Métriques : request count, latency histogram, error rate, DB pool

### Alertes
- [ ] Alertmanager ou Grafana alerting
- [ ] Alerte disk > 80%
- [ ] Alerte backend down (health check fail)
- [ ] Alerte error rate > 5%
- [ ] Alerte PostgreSQL connections > 80

---

## P2 — Security Audit Fixes (code analysis 2026-03-21)

### Critical ✅
- [x] **S1: CORS** → restreint à offrii.com, api, cdn, staging
- [x] **S2: Login rate limit** → 10 attempts/5min par identifiant via Redis
- [x] **S3: SSRF** → blocage IPs privées + validation scheme http/https

### High
- [x] **P1: Index `circle_members(user_id)`** → migration 20260321000001
- [x] ~~S5: Register rate limit~~ → couvert par Caddy IP rate limit + unique email constraint

### Medium
- [x] **S6: OG fetch** → content-length check avant download
- [x] **P2: `list_items` enrichissement** → tokio::join! (4→3 round-trips)
- [x] **Q2: `list_recent_fulfilled`** → moved to service/repo layer
- [x] **Q1: Admin `is_admin`** → JWT claims (no more DB query per admin request)

### Low
- [ ] **S7: ngrok URL hardcodée dans debug** → utiliser localhost
- [ ] **P3: Index manquant `community_wishes(fulfilled_at)`** → migration
- [ ] **P4: Index/cleanup `refresh_tokens`** → job de nettoyage
- [ ] **Q3: Emails fire-and-forget sans retry** → retry avec backoff
- [ ] **Q4: `DefaultHasher` non-déterministe cache keys** → utiliser xxhash/fnv
- [ ] **A1: HTML templates inline dans handlers** → include_str! ou askama
- [ ] **A2: AppState god object (17 services)** → sous-states groupés

---

## P3 — Hardening (ce sprint)

### Exporters
- [ ] postgres_exporter (connexions, queries, locks, replication lag)
- [ ] redis_exporter (memory, evictions, connections)

### Securite
- [ ] Rotation JWT keys (versionning, grace period)
- [ ] Documenter procédure rotation API keys
- [ ] Vérifier DMARC/SPF pour emails Resend

### Ops
- [ ] Test de restore backup (snapshot Hetzner → nouveau serveur)
- [ ] Runbook : comment redémarrer, rollback, debug
- [ ] Documenter procédure d'incident

---

## P4 — Nice to have (backlog)

- [ ] Staging environment complet (docker-compose.staging.yml)
- [ ] Canary deployments / blue-green
- [ ] postgres_exporter + redis_exporter dans le compose
- [ ] OpenTelemetry tracing
- [ ] DDoS protection (Cloudflare proxy)
- [ ] Multi-instance backend (horizontal scaling)
- [ ] Database replication (read replica)

---

## Notes de deploy

### 2026-03-20 — Premier deploy
- Image GHCR : `ghcr.io/yassine-el-gherrabi/offrii-api:latest` (private)
- Fixes appliqués pendant le deploy :
  - GHCR username corrigé (yassinelechef → yassine-el-gherrabi)
  - Image rebuilt pour linux/amd64 (serveur x86, Mac = arm64)
  - APNs key path corrigé (AuthKey.p8 → apns_key.p8)
  - Caddy : retiré staging/grafana blocks (basicauth env vars manquantes)
  - Caddy : CSP relaxé pour /docs (default-src 'none' bloquait Swagger UI)
  - Docker auth configuré sur serveur pour pull GHCR privé
