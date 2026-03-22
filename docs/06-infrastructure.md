# Infrastructure & Deploiement

Ce document decrit l'ensemble de l'infrastructure Offrii : serveur, conteneurs, pipeline CI/CD, monitoring, reverse proxy, gestion des secrets et sauvegardes. Il permet de reconstruire l'infrastructure complete a partir de zero.

---

## Table des matieres

1. [Environnements](#1-environnements)
2. [Serveur](#2-serveur)
3. [Stack Docker](#3-stack-docker)
4. [Pipeline CI/CD](#4-pipeline-cicd)
5. [Gestion des secrets](#5-gestion-des-secrets)
6. [Monitoring](#6-monitoring)
7. [Reverse proxy (Caddy)](#7-reverse-proxy-caddy)
8. [Sauvegardes](#8-sauvegardes)

---

## 1. Environnements

Offrii dispose de trois environnements, tous heberges sur le meme serveur Hetzner (isolation par Docker Compose projects).

### Vue d'ensemble

| Parametre | Dev (local) | Staging | Production |
|---|---|---|---|
| **URL** | `localhost` | `staging.offrii.com` | `api.offrii.com` |
| **Branche Git** | toutes | `develop` | `master` |
| **RUST_LOG** | `rest_api=debug,tower_http=debug` | `rest_api=debug,tower_http=debug` | `rest_api=info,tower_http=info` |
| **APNS_SANDBOX** | `true` | `true` | `false` |
| **MODERATION_ENABLED** | non defini | `false` | `true` |
| **SEED_DEV_DATA** | `true` (optionnel) | configurable via variable | non |
| **CORS_ORIGIN** | `http://localhost:19006,http://localhost:8081` | `https://staging.offrii.com` | `https://offrii.com` |
| **POSTGRES_DB** | `offrii` | `offrii` (volume isole) | `offrii` |
| **R2_BUCKET_NAME** | `offrii-media` | `offrii-media-staging` | `offrii-media` |
| **API_PORT** | `3000` | `3001` | `3000` |
| **TLS** | non (HTTP) | oui (Let's Encrypt) | oui (Let's Encrypt) |
| **Deploiement** | `docker compose up` | push sur `develop` | push sur `master` |
| **Image backend** | build local | `ghcr.io/.../offrii-api:staging` | `ghcr.io/.../offrii-api:latest` |
| **Limite memoire backend** | aucune | 256 Mo | 512 Mo |
| **Limite memoire PostgreSQL** | aucune | 256 Mo | 1 Go |

### Isolation staging / production

Staging et production tournent sur le meme serveur mais sont entierement isoles :

- **Docker Compose projects** distincts : `offrii` (prod) et `offrii-staging` (staging)
- **Volumes Docker** separes : `postgres_data` (prod) vs `staging_postgres_data` (staging)
- **Fichiers secrets** separes : `/opt/offrii/secrets/` (prod) vs `/opt/offrii/secrets/staging/` (staging)
- **Fichiers .env** separes : `.env` (prod) vs `.env.staging` (staging)
- **Ports** : backend prod sur `3000`, staging sur `3001`
- **Reseaux Docker** : chaque project cree son propre reseau `offrii-net`

---

## 2. Serveur

### Specifications materielles

| Parametre | Valeur |
|---|---|
| **Fournisseur** | Hetzner Cloud |
| **Modele** | CX32 |
| **vCPU** | 4 |
| **RAM** | 8 Go |
| **Stockage** | 80 Go SSD NVMe |
| **OS** | Ubuntu 24.04 LTS |
| **Localisation** | Europe (Hetzner) |

### Logiciels installes

| Logiciel | Role |
|---|---|
| Docker + Docker Compose | Orchestration des conteneurs |
| Caddy (conteneurise) | Reverse proxy, TLS, rate limiting |
| cron | Planification des sauvegardes |

### Arborescence serveur

```
/opt/offrii/
  docker-compose.yml
  docker-compose.prod.yml
  docker-compose.staging.yml
  .env                          # Genere par GitHub Actions (prod)
  .env.staging                  # Genere par GitHub Actions (staging)
  infra/
    caddy/
      Caddyfile.prod
    monitoring/
      prometheus.yml
      loki.yml
      promtail.yml
      grafana/
        provisioning/
        dashboards/
    scripts/
      backup.sh
  secrets/
    jwt_private.pem             # Cle RSA privee (prod)
    jwt_public.pem              # Cle RSA publique (prod)
    apns_key.p8                 # Cle APNs Apple (prod)
    staging/
      jwt_private.pem           # Cle RSA privee (staging)
      jwt_public.pem            # Cle RSA publique (staging)
      apns_key.p8               # Cle APNs Apple (staging)
  backups/
    offrii_20260322_030000.sql.gz
    ...
```

### Repartition memoire (8 Go)

```
+----------------------------------------------+
|  PostgreSQL          1 024 Mo                 |
|  Grafana               512 Mo                 |
|  Prometheus            512 Mo                 |
|  Backend (prod)        512 Mo                 |
|  Backend (staging)     256 Mo                 |
|  Loki                  256 Mo                 |
|  Redis (prod)          256 Mo                 |
|  PostgreSQL (staging)  256 Mo                 |
|  Caddy                 128 Mo                 |
|  Promtail              128 Mo                 |
|  Node Exporter         128 Mo                 |
|  Redis (staging)       128 Mo                 |
|  Postgres Exporter      64 Mo                 |
|  Redis Exporter         64 Mo                 |
+----------------------------------------------+
|  OS + marge         ~1 900 Mo                 |
+----------------------------------------------+
|  TOTAL               ~6 200 Mo / 8 192 Mo    |
+----------------------------------------------+
```

---

## 3. Stack Docker

### Services de production

L'infrastructure est definie dans deux fichiers composes en overlay :

- `docker-compose.yml` : configuration de base (dev + prod)
- `docker-compose.prod.yml` : surcharges production (images GHCR, limites memoire, monitoring)

Commande de lancement : `docker compose -f docker-compose.yml -f docker-compose.prod.yml up -d`

| Service | Image | Role | Port interne | Limite memoire |
|---|---|---|---|---|
| **postgres** | `postgres:18-alpine` | Base de donnees relationnelle principale | 5432 | 1 Go |
| **redis** | `redis:8-alpine` | Cache, sessions, rate limiting | 6379 | 256 Mo |
| **backend** | `ghcr.io/yassine-el-gherrabi/offrii-api:latest` | API REST Rust/Axum | 3000 | 512 Mo |
| **caddy** | `caddy:2.11.1-alpine` + plugin rate_limit | Reverse proxy, TLS, securite | 80, 443 | 128 Mo |
| **prometheus** | `prom/prometheus:v3.3.0` | Collecte de metriques | 9090 | 512 Mo |
| **grafana** | `grafana/grafana:11.6.0` | Dashboards et alertes | 3000 | 512 Mo |
| **loki** | `grafana/loki:3.5.0` | Agregation de logs | 3100 | 256 Mo |
| **promtail** | `grafana/promtail:3.5.0` | Agent de collecte de logs Docker | - | 128 Mo |
| **node-exporter** | `prom/node-exporter:v1.9.0` | Metriques systeme (CPU, RAM, disque) | 9100 | 128 Mo |
| **postgres-exporter** | `prometheuscommunity/postgres-exporter:v0.16.0` | Metriques PostgreSQL | 9187 | 64 Mo |
| **redis-exporter** | `oliver006/redis_exporter:v1.67.0` | Metriques Redis | 9121 | 64 Mo |

### Services de staging (stack isolee)

Definis dans `docker-compose.staging.yml`, lances avec le project name `offrii-staging`.

| Service | Image | Port interne | Limite memoire |
|---|---|---|---|
| **staging-postgres** | `postgres:18-alpine` | 5432 | 256 Mo |
| **staging-redis** | `redis:8-alpine` | 6379 | 128 Mo |
| **staging-backend** | `ghcr.io/yassine-el-gherrabi/offrii-api:staging` | 3001 | 256 Mo |

Staging partage le Caddy de production (le Caddyfile.prod contient le bloc `staging.offrii.com`).

### Configuration PostgreSQL (production)

Le serveur PostgreSQL est demarre avec des parametres optimises pour le CX32 :

| Parametre | Valeur | Justification |
|---|---|---|
| `shared_buffers` | 256 Mo | 25 % de la memoire allouee au PG |
| `effective_cache_size` | 512 Mo | Estimation du cache OS disponible |
| `work_mem` | 4 Mo | Par operation de tri/hash |
| `maintenance_work_mem` | 64 Mo | VACUUM, CREATE INDEX |
| `max_connections` | 100 | Suffisant pour le backend mono-instance |
| `log_min_duration_statement` | 500 ms | Journalise les requetes lentes |
| `data-checksums` | active | Detection de corruption de donnees |

### Image Docker du backend

Le Dockerfile utilise un build multi-stage pour minimiser la taille de l'image finale :

```
Stage 1 : Builder (rust:1.93-slim-bookworm)
  - Copie Cargo.toml + Cargo.lock
  - Build "vide" pour mettre en cache les dependances
  - Copie du code source reel
  - Build release

Stage 2 : Runtime (debian:bookworm-slim)
  - Installe ca-certificates, curl, postgresql-client
  - Copie les binaires : rest-api, migrate, seed
  - Utilisateur non-root (appuser, UID 1000)
  - Expose le port 3000
  - HEALTHCHECK integre : curl http://localhost:3000/health/live
```

### Diagramme reseau Docker

```
                        Internet
                           |
                    [ Hetzner CX32 ]
                      80/443 (TCP+UDP)
                           |
                       [ caddy ]
                      /    |     \
                     /     |      \
          api.offrii.com   |   grafana.offrii.com
          staging.offrii.com
                     |     |      |
              [ backend ] [ staging-backend ] [ grafana ]
                  |    \        |    \            |
           [ postgres ] [ redis ]    [ staging-postgres ]   [ prometheus ]
                                     [ staging-redis ]      /    |     \
                                                     [ node ]  [ pg ]  [ redis ]
                                                     [ exp. ]  [ exp.] [ exp. ]
                           |
                        [ loki ] <-- [ promtail ] (lit /var/run/docker.sock)
```

Tous les services communiquent via le reseau Docker bridge `offrii-net`. Seul Caddy expose les ports 80/443 a l'exterieur.

---

## 4. Pipeline CI/CD

### Vue d'ensemble des workflows

| Workflow | Fichier | Declencheur | Environnement |
|---|---|---|---|
| **CI** | `ci.yml` | Push/PR sur `master` ou `develop` | - |
| **Deploy Staging** | `deploy-staging.yml` | Push sur `develop` | staging |
| **Deploy Production** | `deploy-prod.yml` | Push sur `master` | prod |

### CI (`ci.yml`)

Ce workflow s'execute sur chaque push et pull request. Il detecte les fichiers modifies pour ne lancer que les jobs necessaires.

**Branch gate** : les PRs vers `master` ne sont acceptees que depuis `develop`.

**Job backend** (si fichiers `backend/**` modifies) :

1. Checkout du code
2. Installation de la toolchain Rust stable + composants (rustfmt, clippy, llvm-tools-preview)
3. Installation des outils : cargo-sort, cargo-machete, cargo-nextest, cargo-llvm-cov
4. Cache Rust (Swatinem/rust-cache)
5. Verification du formatage (`cargo fmt --check`)
6. Verification du tri des Cargo.toml (`cargo sort --workspace --check`)
7. Analyse statique (`cargo clippy -- -D warnings`)
8. Detection des dependances inutilisees (`cargo machete`)
9. Pre-pull des images testcontainers (postgres:16-alpine, redis:7)
10. Tests avec couverture (`cargo llvm-cov nextest --fail-under-lines 75`)
11. Audit de securite (`cargo audit`, mode consultatif)

**Job iOS** (si fichiers `frontend/**` modifies) :

1. Machine macOS 15, Xcode 16.4
2. SwiftLint strict
3. Build + tests avec couverture

### Deploy Production (`deploy-prod.yml`)

Declenche par un push sur `master` affectant : `backend/**`, `docker-compose*.yml`, `infra/**`, ou le workflow lui-meme.

**Concurrence** : un seul deploiement a la fois (`cancel-in-progress: false`).

```
 push master
     |
     v
 +-------+     +-------+     +--------+
 | test  | --> | build | --> | deploy |
 +-------+     +-------+     +--------+
```

#### Etape 1 : Test (`test`)

- Checkout, toolchain Rust, cache
- Pre-pull images testcontainers
- `cargo nextest run --workspace`

#### Etape 2 : Build (`build`)

- Login au registre GHCR (GitHub Container Registry)
- Setup Docker Buildx
- Generation des tags : `latest` + SHA du commit
- Build et push de l'image Docker
- Cache GHA pour les layers Docker
- Scan de vulnerabilites Trivy (CRITICAL + HIGH, mode consultatif)

#### Etape 3 : Deploy (`deploy`)

Cette etape utilise l'environnement GitHub `prod` (protection requise).

```
1. SCP : Synchronisation des fichiers de configuration
   - infra/, docker-compose.yml, docker-compose.prod.yml
   -> /opt/offrii/ sur le serveur

2. SSH : Execution du script de deploiement
   |
   +-- Generation du fichier .env
   |   (toutes les variables d'environnement depuis GitHub)
   |   chmod 600 .env
   |
   +-- Decodage des secrets en base64
   |   JWT_PRIVATE_KEY_BASE64 -> /opt/offrii/secrets/jwt_private.pem
   |   JWT_PUBLIC_KEY_BASE64  -> /opt/offrii/secrets/jwt_public.pem
   |   APNS_KEY_BASE64        -> /opt/offrii/secrets/apns_key.p8
   |   chmod 600 secrets/*
   |
   +-- Pull de la nouvelle image backend depuis GHCR
   |
   +-- Sauvegarde de l'image courante (pour rollback)
   |
   +-- Migrations de base de donnees
   |   docker compose run --rm --entrypoint migrate backend
   |   (echec -> abort immediat)
   |
   +-- Demarrage des services
   |   docker compose up -d --no-build
   |
   +-- Health check (12 tentatives, 5s d'intervalle = 60s max)
   |   curl http://localhost/health/ready
   |   OU curl https://api.offrii.com/health/ready
   |
   +-- Si OK :
   |   +-- Annotation Grafana (tag deploy + SHA)
   |   +-- Nettoyage des images Docker inutilisees
   |   +-- Exit 0
   |
   +-- Si ECHEC :
       +-- Rollback vers l'image precedente
       +-- docker compose down backend
       +-- docker tag (ancienne image) -> latest
       +-- docker compose up -d
       +-- Exit 1
```

### Deploy Staging (`deploy-staging.yml`)

Meme structure que la production avec les differences suivantes :

| Aspect | Staging | Production |
|---|---|---|
| Branche | `develop` | `master` |
| cancel-in-progress | `true` | `false` |
| Tag image | `:staging` | `:latest` + `:sha` |
| Scan Trivy | non | oui |
| Project Docker | `offrii-staging` | defaut (`offrii`) |
| Fichier .env | `.env.staging` | `.env` |
| Repertoire secrets | `/opt/offrii/secrets/staging/` | `/opt/offrii/secrets/` |
| Rollback automatique | non | oui |

---

## 5. Gestion des secrets

### Strategie

Les secrets sont stockes dans GitHub et injectes au deploiement. Aucun secret n'est commite dans le depot Git. Le fichier `.env` est genere dynamiquement par le workflow de deploiement.

Sur le serveur, les fichiers sensibles (cles, .env) ont les permissions `600` (lecture/ecriture proprietaire uniquement).

### Niveaux de stockage GitHub

| Niveau | Portee | Utilisation |
|---|---|---|
| **Repository secrets** | Tous les workflows | Secrets partages (DEPLOY_SSH_KEY) |
| **Environment secrets** | Un environnement (`prod` ou `staging`) | Secrets specifiques a un environnement |
| **Environment variables** | Un environnement | Valeurs non sensibles specifiques |

### Inventaire des secrets

#### Secrets au niveau du repository

| Secret | Description |
|---|---|
| `DEPLOY_SSH_KEY` | Cle SSH pour acceder au serveur Hetzner |

#### Secrets d'environnement (prod et staging)

| Secret | Description |
|---|---|
| `POSTGRES_PASSWORD` | Mot de passe de la base de donnees |
| `RESEND_API_KEY` | Cle API pour l'envoi d'emails (Resend) |
| `GOOGLE_CLIENT_ID` | Client ID pour l'authentification Google |
| `R2_ACCOUNT_ID` | Identifiant du compte Cloudflare R2 |
| `R2_ACCESS_KEY_ID` | Cle d'acces R2 |
| `R2_SECRET_ACCESS_KEY` | Cle secrete R2 |
| `OPENAI_API_KEY` | Cle API OpenAI (moderation de contenu) |
| `GF_SECURITY_ADMIN_PASSWORD` | Mot de passe admin Grafana |
| `GRAFANA_BASIC_AUTH_HASH_B64` | Hash bcrypt encode en base64 pour l'auth basique Caddy |
| `JWT_PRIVATE_KEY_BASE64` | Cle RSA privee encodee en base64 |
| `JWT_PUBLIC_KEY_BASE64` | Cle RSA publique encodee en base64 |
| `APNS_KEY_BASE64` | Cle APNs Apple encodee en base64 |

Note : staging utilise `JWT_PRIVATE_KEY_BASE64_STAGING` et `JWT_PUBLIC_KEY_BASE64_STAGING` pour avoir des cles JWT distinctes.

#### Variables d'environnement (non sensibles)

| Variable | Exemple prod | Exemple staging |
|---|---|---|
| `HETZNER_HOST` | IP du serveur | IP du serveur |
| `POSTGRES_DB` | `offrii` | `offrii` |
| `POSTGRES_USER` | `offrii` | `offrii` |
| `RUST_LOG` | `rest_api=info,tower_http=info` | `rest_api=debug,tower_http=debug` |
| `R2_BUCKET_NAME` | `offrii-media` | `offrii-media-staging` |
| `R2_PUBLIC_URL` | URL publique R2 | URL publique R2 |
| `APNS_KEY_ID` | `LTCYFUC4WT` | `LTCYFUC4WT` |
| `APNS_TEAM_ID` | `6SMPW6NUXP` | `6SMPW6NUXP` |
| `APNS_BUNDLE_ID` | `com.offrii.app` | `com.offrii.app` |
| `CORS_ORIGIN` | `https://offrii.com` | `https://staging.offrii.com` |
| `SEED_DEV_DATA` | - | `true` (optionnel) |

### Secrets sur le serveur

Les fichiers sensibles sont stockes en dehors du depot Git :

```
/opt/offrii/secrets/
  jwt_private.pem     # 600, genere par le workflow
  jwt_public.pem      # 600, genere par le workflow
  apns_key.p8         # 600, genere par le workflow
  staging/
    jwt_private.pem   # 600, genere par le workflow
    jwt_public.pem    # 600, genere par le workflow
    apns_key.p8       # 600, genere par le workflow
```

Les fichiers de cles sont montes en lecture seule dans les conteneurs via des volumes Docker :

```yaml
volumes:
  - /opt/offrii/secrets/jwt_private.pem:/run/secrets/jwt_private_key:ro
  - /opt/offrii/secrets/jwt_public.pem:/run/secrets/jwt_public_key:ro
  - /opt/offrii/secrets/apns_key.p8:/run/secrets/apns_key:ro
```

---

## 6. Monitoring

### Architecture de monitoring

```
                   [ Grafana :3000 ]
                   /       |       \
                  /        |        \
        [ Prometheus ]  [ Loki ]    Alertes
         :9090          :3100       (email)
        /  |  \           ^
       /   |   \          |
  [node] [pg] [redis]  [ promtail ]
  [exp ] [exp] [exp ]     |
    |      |     |    Docker socket
    v      v     v        |
  Hetzner  PG   Redis   Logs conteneurs
```

### Prometheus

**Configuration** : `infra/monitoring/prometheus.yml`

| Parametre | Valeur |
|---|---|
| Intervalle de scrape | 15 secondes |
| Intervalle d'evaluation | 15 secondes |
| Retention des donnees | 15 jours |

**Cibles de scrape** :

| Job | Cible | Port | Metriques collectees |
|---|---|---|---|
| `node-exporter` | `node-exporter:9100` | 9100 | CPU, memoire, disque, reseau du serveur |
| `offrii-backend` | `backend:3000` | 3000 | Requetes HTTP (axum), latence, erreurs |
| `postgres` | `postgres-exporter:9187` | 9187 | Connexions, transactions, taille des tables |
| `redis` | `redis-exporter:9121` | 9121 | Memoire utilisee, commandes, cles |

### Grafana

**Acces** : `https://grafana.offrii.com` (protege par basic auth Caddy + auth Grafana)

**Datasources provisionnees** :

| Datasource | Type | URL interne | UID |
|---|---|---|---|
| Prometheus | prometheus | `http://prometheus:9090` | `prometheus` |
| Loki | loki | `http://loki:3100` | `loki` |

**Dashboards provisionnes** (4) :

| Dashboard | Fichier | Contenu principal |
|---|---|---|
| **Node Exporter** | `node-exporter.json` | CPU, memoire, disque, reseau du serveur |
| **Backend** | `backend.json` | Requetes/s, latence P50/P95/P99, taux d'erreurs, endpoints |
| **PostgreSQL** | `postgresql.json` | Connexions actives, transactions/s, taille de la base |
| **Redis** | `redis.json` | Memoire utilisee, commandes/s, cles, hit ratio |

### Alertes

**Point de contact** : email vers `yassineelgherrabi@gmail.com`

**Politique** : groupement par `alertname`, attente de 30s, intervalle de 5min, repetition toutes les 4h.

| Alerte | UID | Condition | Duree | Severite |
|---|---|---|---|---|
| **Disk usage > 80%** | `disk-usage-high` | Espace disque utilise > 80 % sur `/` | 5 min | warning |
| **CPU usage > 90%** | `high-cpu` | Usage CPU moyen > 90 % | 5 min | critical |
| **Backend down** | `backend-down` | `up{job="offrii-backend"} < 1` | 2 min | critical |
| **Error rate > 5%** | `high-error-rate` | Taux de reponses 5xx > 5 % | 5 min | critical |
| **P99 latency > 2s** | `high-latency-p99` | Percentile 99 de la latence > 2 secondes | 5 min | warning |

**Expressions PromQL des alertes** :

```promql
# Disk usage
100 - ((node_filesystem_avail_bytes{mountpoint="/"} / node_filesystem_size_bytes{mountpoint="/"}) * 100)

# CPU usage
100 - (avg(rate(node_cpu_seconds_total{mode="idle"}[5m])) * 100)

# Backend down
up{job="offrii-backend"}

# Error rate
(sum(rate(axum_http_requests_total{status=~"5.."}[5m])) / sum(rate(axum_http_requests_total[5m]))) * 100

# P99 latency
histogram_quantile(0.99, sum(rate(axum_http_requests_duration_seconds_bucket[5m])) by (le))
```

### Loki (logs)

| Parametre | Valeur |
|---|---|
| Image | `grafana/loki:3.5.0` |
| Configuration | `infra/monitoring/loki.yml` |
| Stockage | Volume Docker `loki_data` |
| Collecteur | Promtail (lit le Docker socket) |

Promtail collecte automatiquement les logs de tous les conteneurs Docker via `/var/run/docker.sock` et les pousse vers Loki. Les logs sont consultables dans Grafana via la datasource Loki.

---

## 7. Reverse proxy (Caddy)

### Image personnalisee

Caddy est construit avec le plugin `caddy-ratelimit` via xcaddy :

```dockerfile
FROM caddy:2.11.1-builder AS builder
RUN xcaddy build --with github.com/mholt/caddy-ratelimit@v0.1.0

FROM caddy:2.11.1-alpine
COPY --from=builder /usr/bin/caddy /usr/bin/caddy
COPY Caddyfile /etc/caddy/Caddyfile
```

### TLS automatique

Caddy gere automatiquement les certificats TLS via Let's Encrypt pour les trois domaines :

- `api.offrii.com`
- `staging.offrii.com`
- `grafana.offrii.com`

Aucune configuration manuelle n'est necessaire. Caddy renouvelle les certificats avant expiration.

### Domaines et routage

| Domaine | Backend cible | Protection | Notes |
|---|---|---|---|
| `api.offrii.com` | `backend:3000` | Rate limiting, security headers | API principale |
| `staging.offrii.com` | `staging-backend:3001` | Rate limiting, security headers | API de test |
| `grafana.offrii.com` | `grafana:3000` | Basic auth Caddy + auth Grafana | Double authentification |

### En-tetes de securite

Appliques sur tous les domaines :

| En-tete | Valeur | Objectif |
|---|---|---|
| `X-Content-Type-Options` | `nosniff` | Empeche le MIME sniffing |
| `X-Frame-Options` | `DENY` (`SAMEORIGIN` pour Grafana) | Protection contre le clickjacking |
| `X-XSS-Protection` | `0` | Desactive le filtre XSS du navigateur (obsolete) |
| `Referrer-Policy` | `strict-origin-when-cross-origin` | Controle les informations referrer |
| `Permissions-Policy` | `camera=(), microphone=(), geolocation=()` | Desactive les APIs sensibles |
| `Content-Security-Policy` | `default-src 'none'; frame-ancestors 'none'` | Politique de securite du contenu (API) |
| `Strict-Transport-Security` | `max-age=63072000; includeSubDomains; preload` | Force HTTPS (2 ans) |
| `Server` | supprime | Cache l'identite du serveur |

### Zones de rate limiting

| Zone | Chemin | Limite | Cle |
|---|---|---|---|
| `auth_zone` | `/auth/*` | 10 requetes / minute | IP source |
| `messages_zone` | `/community/wishes/*/messages` | 20 requetes / minute | IP source |
| `global_zone` | toutes les requetes | 100 requetes / minute | IP source |

L'ordre d'evaluation est : auth -> messages -> global. Les rate limits s'appliquent par IP source (`{remote_host}`).

### Protection de /metrics

Le endpoint `/metrics` (expose par le backend pour Prometheus) est bloque au niveau de Caddy pour les requetes externes :

```
@metrics path /metrics
respond @metrics 404
```

Prometheus accede aux metriques via le reseau Docker interne, sans passer par Caddy.

### CORS

La politique CORS est configuree par domaine :

| Domaine | Access-Control-Allow-Origin | Methodes |
|---|---|---|
| `api.offrii.com` | `https://offrii.com` | GET, POST, PUT, PATCH, DELETE, OPTIONS |
| `staging.offrii.com` | `https://staging.offrii.com` | idem |
| Dev local | `*` (ou configurable via `$CORS_ORIGIN`) | idem |

Les requetes OPTIONS sont traitees avec une reponse `204` directe.

### Health checks internes

Caddy effectue des health checks actifs sur les backends :

| Parametre | Valeur |
|---|---|
| URI | `/health/ready` |
| Intervalle | 30 secondes |
| Timeout | 5 secondes |

---

## 8. Sauvegardes

### Strategie de sauvegarde

Deux niveaux de sauvegarde complementaires :

| Niveau | Methode | Frequence | Retention | Contenu |
|---|---|---|---|---|
| **Serveur complet** | Hetzner Snapshots | Manuel / planifiable | Selon politique Hetzner | Disque complet (OS + donnees + config) |
| **Base de donnees** | `pg_dump` via script | Quotidien (cron 03:00) | 14 jours | Dump SQL compresse (gzip) |

### Script de sauvegarde (`infra/scripts/backup.sh`)

Le script effectue les operations suivantes :

1. Creation du repertoire `/opt/offrii/backups/` si inexistant
2. Dump de la base via `docker exec offrii-postgres pg_dump`
   - Format plain text
   - Options `--no-owner --no-privileges`
   - Compression gzip
3. Suppression des sauvegardes de plus de 14 jours
4. Affichage du rapport (nombre de fichiers, taille totale)

**Planification cron** :

```
0 3 * * * /opt/offrii/infra/scripts/backup.sh >> /var/log/offrii-backup.log 2>&1
```

**Fichiers generes** :

```
/opt/offrii/backups/
  offrii_20260320_030000.sql.gz
  offrii_20260321_030000.sql.gz
  offrii_20260322_030000.sql.gz
  ...
```

### Restauration

Pour restaurer une sauvegarde :

```bash
# Decompresser et restaurer
gunzip -c /opt/offrii/backups/offrii_20260322_030000.sql.gz | \
  docker exec -i offrii-postgres psql -U offrii -d offrii

# OU restaurer sur une base vierge
docker exec offrii-postgres createdb -U offrii offrii_restore
gunzip -c /opt/offrii/backups/offrii_20260322_030000.sql.gz | \
  docker exec -i offrii-postgres psql -U offrii -d offrii_restore
```

### Sauvegarde hors site (prevu)

Le script contient une section commentee pour l'upload vers un stockage distant via `rclone`. Cette fonctionnalite est prevue pour une activation future.

```bash
# A decommenter quand rclone est configure :
# rclone copy "$BACKUP_FILE" offrii-backup:offrii-backups/ --progress
```

---

## Annexe : Reconstruire l'infrastructure de zero

Procedure complete pour reconstruire l'infrastructure sur un nouveau serveur Hetzner CX32 :

### 1. Provisionner le serveur

- Commander un CX32 chez Hetzner (4 vCPU, 8 Go RAM, Ubuntu 24.04)
- Configurer le DNS : `api.offrii.com`, `staging.offrii.com`, `grafana.offrii.com` vers l'IP du serveur
- Creer un utilisateur `deploy` avec acces sudo

### 2. Installer Docker

```bash
curl -fsSL https://get.docker.com | sh
usermod -aG docker deploy
```

### 3. Preparer l'arborescence

```bash
mkdir -p /opt/offrii/secrets/staging
mkdir -p /opt/offrii/backups
chown -R deploy:deploy /opt/offrii
```

### 4. Configurer GitHub

- Ajouter le secret `DEPLOY_SSH_KEY` au niveau du repository
- Creer les environnements `prod` et `staging` dans GitHub
- Renseigner tous les secrets et variables d'environnement (voir section 5)

### 5. Deployer

- Merger dans `develop` pour deployer le staging
- Merger `develop` dans `master` pour deployer la production
- Le workflow CI/CD gere automatiquement : envoi des fichiers, generation du .env, decodage des secrets, pull de l'image, migrations, demarrage, health check

### 6. Configurer les sauvegardes

```bash
crontab -e
# Ajouter :
0 3 * * * /opt/offrii/infra/scripts/backup.sh >> /var/log/offrii-backup.log 2>&1
```

### 7. Verifier

- `https://api.offrii.com/health/ready` retourne 200
- `https://staging.offrii.com/health/ready` retourne 200 (si staging deploye)
- `https://grafana.offrii.com` affiche le dashboard Grafana
- Les 4 dashboards sont visibles dans Grafana
- Les 5 alertes sont configurees dans Grafana Alerting
