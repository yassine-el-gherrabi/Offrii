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
- [x] Datasources provisioning (Prometheus + Loki) — auto-configured on deploy
- [x] Dashboard host (CPU, RAM, disk, network) via node-exporter — ID 1860
- [ ] Dashboard backend (requests, latency, errors) — nécessite backend /metrics
- [ ] Dashboard PostgreSQL — nécessite postgres_exporter
- [ ] Dashboard Redis — nécessite redis_exporter

### Loki log streaming
- [x] Promtail container déployé — collecte logs de tous les containers via Docker socket

### Backend /metrics
- [ ] Ajouter endpoint Prometheus /metrics dans le backend Rust (axum-prometheus)
- [ ] Configurer Prometheus pour scraper backend:3000/metrics
- [ ] Métriques : request count, latency histogram, error rate, DB pool

### Alertes
- [x] Grafana unified alerting activé
- [x] Contact point : email → yassineelgherrabi@gmail.com
- [x] Alerte disk > 80%
- [x] Alerte CPU > 90% pendant 5min
- [ ] Alerte backend down (nécessite backend /metrics)
- [ ] Alerte error rate > 5% (nécessite backend /metrics)

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

### Low ✅
- [x] ~~S7: ngrok URL~~ → déjà supprimé du code
- [x] **P3: Index `fulfilled_at`** → migration 20260321000002
- [x] **P4: Cleanup `refresh_tokens`** → migration 20260321000003 (index existait déjà)
- [x] **Q3: Email retry** → 3 attempts, 1s/2s backoff via send_with_retry
- [x] **Q4: Cache keys** → format!() déterministe au lieu de DefaultHasher
- [x] **A1: HTML templates** → extract to templates/ dir, include_str!
- [ ] ~~A2: AppState god object~~ → skip, refacto post-Series A

---

## P3 — Hardening ✅

### Exporters
- [x] postgres_exporter v0.16.0 + dashboard Grafana (ID 9628)
- [x] redis_exporter v1.67.0 + dashboard Grafana (ID 763)
- [x] Prometheus config mis à jour pour scraper les 2

### Securite
- [x] Rotation JWT keys → procédure documentée dans RUNBOOK.md
- [x] Rotation API keys → procédure documentée dans RUNBOOK.md
- [x] DMARC/SPF vérifié : SPF ✅, DKIM ✅, DMARC ✅ (p=none, OK pour le launch)

### Ops
- [ ] Test de restore backup (snapshot Hetzner → nouveau serveur) — à faire manuellement
- [x] Runbook complet : infra/docs/RUNBOOK.md (SSH, logs, restart, rollback, DB, Redis, diagnostic, rotation clés)

---

## Audit final (2026-03-21)

- [x] Trivy container image scanning dans CD (advisory)
- [x] PR review requirement retiré (solo dev)
- [ ] Mettre à jour privacy policy avec liste sous-traitants RGPD (Resend, OpenAI, Cloudflare, Hetzner, APNs)
- [x] Uptime monitoring externe — UptimeRobot
  - Prod : `https://api.offrii.com/health/ready` (5 min interval)
  - Status page : `https://stats.uptimerobot.com/J7vbVoj5h1`
  - [ ] Ajouter monitor staging : `https://staging.offrii.com/health/ready` (quand staging déployé)

---

## Schema + API Audit (2026-03-21)

### Critical (à faire)
- [ ] **Drop `items.url` column** — data migrée vers `links[]`, colonne morte
  - Backend : migration SQL + retirer du model Rust `Item` + retirer de `ITEM_COLS`
  - Frontend : vérifier que `url` n'est pas utilisé dans `Item.swift` (probable, à checker)
- [ ] **Drop 3 index redondants** — doublons des UNIQUE constraints
  - `idx_circle_invites_token`, `idx_verification_token`, `idx_email_change_token`
  - Backend only, aucun impact frontend
- [ ] **`PUT /items/{id}` → `PATCH`** — accepte des updates partielles, devrait être PATCH
  - Backend : changer `put(update_item)` → `patch(update_item)` dans le router
  - Frontend : changer `APIEndpoint` de `.put` à `.patch` pour updateItem
- [ ] **Protéger `/metrics`** — actuellement public, expose les métriques internes
  - Backend : restreindre à localhost ou ajouter un bearer token
  - Caddy : bloquer `/metrics` de l'extérieur (seul Prometheus interne y accède)
  - Aucun impact frontend

### Important (avant scale)
- [ ] **Pagination manquante sur 11 endpoints** — OK < 1000 users, problème après
  - `/circles`, `/me/friends`, `/share-links`, `/community/wishes/mine`,
    `/community/wishes/my-offers`, `/community/wishes/recent-fulfilled`,
    `/circles/my-reservations`, `/circles/my-share-rules`, `/circles/{id}/items`,
    `/circles/{id}/invites`, `/me/friend-requests`
  - Backend : ajouter `page`/`limit` query params + `PaginatedResponse`
  - Frontend : adapter les appels pour gérer la pagination (ou garder tel quel si liste courte)
- [ ] **Missing CHECK constraints DB** — validation app-only, pas DB
  - `notifications.type`, `circle_share_rules.share_mode`, `items.claimed_via`
  - Backend only, aucun impact frontend
- [ ] **Namespace `/me` inconsistant** — `/me/friends` mais `/users/me`
  - Cosmétique, breaking change si on renomme → reporter post-launch
- [ ] **Raw SQL dans handlers** — `circles.rs`, `notifications.rs` bypass le repo pattern
  - Backend refacto, aucun impact frontend

### Backlog (post-launch)
- [ ] API versioning (`/v1/` prefix)
- [ ] ETags / conditional requests pour réduire la bande passante mobile
- [ ] Staging environment complet (docker-compose.staging.yml)
- [ ] Canary deployments / blue-green
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
