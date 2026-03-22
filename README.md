# Offrii

[![CI](https://github.com/yassine-el-gherrabi/Offrii/actions/workflows/ci.yml/badge.svg)](https://github.com/yassine-el-gherrabi/Offrii/actions/workflows/ci.yml)

**Offre, partage, fais plaisir.**

Offrii est une application iOS de wishlist et d'entraide communautaire. Les utilisateurs créent des listes d'envies, les partagent avec leurs proches via des cercles, et peuvent demander ou offrir de l'aide via l'Entraide.

## Stack

| Composant | Technologie |
|-----------|-------------|
| **Backend** | Rust, Axum, SQLx, PostgreSQL 18, Redis 8 |
| **Frontend** | SwiftUI (iOS 17+), Swift Concurrency |
| **Auth** | JWT RS256 (access 15min, refresh 7j), Google & Apple SSO |
| **Storage** | Cloudflare R2 (media), Resend (email), APNs (push) |
| **Infra** | Docker Compose, Caddy, Hetzner, GitHub Actions |
| **Monitoring** | Prometheus, Grafana, Loki |

## Structure

```
offrii/
├── backend/rest-api/       # API Rust/Axum (~100 endpoints)
│   ├── src/handlers/       # Couche HTTP
│   ├── src/services/       # Logique métier
│   ├── src/repositories/   # Accès données (SQLx)
│   ├── src/models/         # Entités domaine
│   ├── src/dto/            # Request/Response DTOs
│   ├── migrations/         # Schéma PostgreSQL (8 migrations)
│   └── tests/              # Tests d'intégration (1042 tests)
├── frontend/ios/           # App SwiftUI
│   ├── Offrii/Features/    # 15 modules fonctionnels
│   ├── Offrii/DesignSystem/ # Composants, thème, localisation
│   └── Offrii/Networking/  # Client API
├── infra/                  # Caddy, Prometheus, Grafana, scripts
├── docs/                   # Documentation projet (8 pages)
├── .github/workflows/      # CI + CD (prod & staging)
└── docker-compose*.yml     # Dev, prod, staging, monitoring
```

## Démarrage rapide

### Prérequis

- [Docker](https://docs.docker.com/get-docker/) et Docker Compose
- [Rust](https://rustup.rs/) (stable) — pour le dev backend
- [Xcode](https://developer.apple.com/xcode/) 16+ — pour le dev iOS
- [cargo-nextest](https://nexte.st/) — pour les tests

### 1. Cloner et configurer

```bash
git clone git@github.com:yassine-el-gherrabi/Offrii.git
cd Offrii

# Configurer les git hooks (fmt, clippy, sort, lint automatiques)
git config core.hooksPath .githooks

# Copier les variables d'environnement
cp .env.example .env
# → Remplir les clés manquantes (RESEND_API_KEY, R2_*, etc.)
```

### 2. Lancer le backend

```bash
# Démarrer tous les services (DB, Redis, backend, Caddy)
docker compose up -d

# Vérifier que tout est healthy
docker compose ps

# Voir les logs du backend
docker compose logs -f backend
```

Le backend est accessible sur `http://localhost:3000`. Le health check :

```bash
curl http://localhost:3000/health/ready
# → {"status":"ok","db":"connected","redis":"connected"}
```

### 3. Lancer les tests

```bash
cd backend
cargo nextest run --workspace
# → 1042 tests, ~10 minutes (utilise testcontainers, pas de mocks)
```

### 4. Lancer l'app iOS

1. Ouvrir `frontend/ios/Offrii.xcodeproj` dans Xcode
2. Sélectionner un simulateur iPhone (iOS 17+)
3. Build & Run (⌘R)

> L'app pointe vers `http://localhost:3000` en dev. Pour pointer vers la prod, configurer `API_BASE_URL` dans le build scheme.

### 5. Monitoring local (optionnel)

```bash
# Ajouter Prometheus, Grafana, Loki, exporters
docker compose -f docker-compose.yml -f docker-compose.local-monitoring.yml up -d

# Grafana sur http://localhost:3001 (admin/admin)
```

## Déploiement

### Production

Le déploiement est automatisé via GitHub Actions. Workflow : `push master → tests → build Docker → push GHCR → deploy sur Hetzner`.

```
master push → ci.yml (tests + lint) → deploy-prod.yml
                                        ├── Build image Docker → GHCR
                                        ├── SCP configs → serveur
                                        ├── Génère .env depuis GitHub Secrets
                                        ├── Run migrations
                                        ├── docker compose up -d
                                        ├── Health check (60s timeout)
                                        └── Rollback automatique si échec
```

**Environnements GitHub** : `prod` et `staging` avec secrets/variables séparés.

| Environnement | Branche | URL | APNS |
|---------------|---------|-----|------|
| **Production** | `master` | `api.offrii.com` | Production |
| **Staging** | `develop` | `staging.offrii.com` | Sandbox |
| **Dev** | local | `localhost:3000` | Sandbox |

### Déploiement manuel (si nécessaire)

```bash
# Build et push l'image
docker buildx build --platform linux/amd64 \
  -t ghcr.io/yassine-el-gherrabi/offrii-api:latest \
  -f backend/Dockerfile backend/ --push

# Deploy sur le serveur
ssh deploy@<SERVER_IP> "cd /opt/offrii && \
  docker compose -f docker-compose.yml -f docker-compose.prod.yml pull backend && \
  docker compose -f docker-compose.yml -f docker-compose.prod.yml up -d --no-build"
```

## Documentation

La documentation complète est dans [`docs/`](docs/) :

| Page | Contenu |
|------|---------|
| [Home](docs/00-home.md) | Vue d'ensemble du projet |
| [Architecture](docs/01-architecture.md) | Patterns, middleware, DI, décisions techniques |
| [Modèle de données](docs/02-data-model.md) | Schéma BDD, migrations, diagramme ER |
| [API Reference](docs/03-api.md) | ~100 endpoints, auth, pagination, erreurs |
| [Règles métier](docs/04-business-rules.md) | State machines, modération, anti-spam |
| [Frontend iOS](docs/05-frontend.md) | SwiftUI, modules, design system |
| [Infrastructure](docs/06-infrastructure.md) | Docker, CI/CD, monitoring, secrets |
| [Sécurité & RGPD](docs/07-security.md) | Auth, OWASP, conformité RGPD |

## Licence

Propriétaire — Tous droits réservés.
