# Offrii

[![CI](https://github.com/yassine-el-gherrabi/offrii/actions/workflows/ci.yml/badge.svg)](https://github.com/yassine-el-gherrabi/offrii/actions/workflows/ci.yml)

**"N'oublie plus ce que tu veux acheter"**

Offrii est une application mobile qui capture tes intentions d'achat à la vitesse de la pensée et te rappelle intelligemment de passer à l'action. Ni wishlist sociale, ni liste de courses, ni comparateur de prix — juste un filet de sécurité pour les achats que tu procrastines.

## Structure monorepo

```
offrii/
├── backend/             # Backend Rust (Axum + PostgreSQL)
├── frontend/            # App mobile React Native (Expo)
├── docs/                # Documentation projet
├── .github/workflows/   # CI/CD GitHub Actions
└── ai-agent-guidelines/ # Guidelines agents IA
```

## Stack technique

| Composant | Technologie |
|-----------|-------------|
| **Backend** | Rust, Axum, SQLx, PostgreSQL |
| **Mobile** | React Native, Expo, TypeScript |
| **Auth** | JWT (access + refresh tokens) |
| **Infra** | Docker, GitHub Actions |

## Démarrage

### Prérequis

- [Docker](https://docs.docker.com/get-docker/) et Docker Compose

### Lancer les services

```bash
# Copier les variables d'environnement
cp .env.example .env

# Lancer PostgreSQL + Redis
docker compose up -d

# Vérifier que les services sont up
docker compose ps

# Voir les logs
docker compose logs -f

# Arrêter les services
docker compose down

# Arrêter et supprimer les volumes (reset complet)
docker compose down -v
```

> Les instructions pour l'API (Rust) et l'app mobile (Expo) seront ajoutées avec DEV-3 et DEV-4.

### Git hooks

```bash
git config core.hooksPath .githooks
```

Le pre-commit hook lance automatiquement les checks (fmt, sort, clippy, lint) sur les fichiers staged, et exécute une vérification de types TypeScript (`npx tsc --noEmit`) lorsque des fichiers frontend sont stagés.

## Licence

[MIT](LICENSE)
