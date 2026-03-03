# Offrii

**"N'oublie plus ce que tu veux acheter"**

Offrii est une application mobile qui capture tes intentions d'achat à la vitesse de la pensée et te rappelle intelligemment de passer à l'action. Ni wishlist sociale, ni liste de courses, ni comparateur de prix — juste un filet de sécurité pour les achats que tu procrastines.

## Structure monorepo

```
offrii/
├── offrii-api/       # Backend Rust (Axum + PostgreSQL)
├── offrii-mobile/    # App mobile React Native (Expo)
├── docker/           # Configuration Docker (PostgreSQL, etc.)
├── docs/             # Documentation projet
├── .github/workflows # CI/CD GitHub Actions
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

> Les instructions de setup seront ajoutées avec les tickets DEV-3 (API) et DEV-4 (Mobile).

## Licence

[MIT](LICENSE)
