# Offrii -- Documentation Technique

## Vision produit

Offrii est une application iOS de **wishlist et d'entraide communautaire**.

Les utilisateurs créent des listes d'envies, les partagent avec leurs proches via des **cercles** (groupes privés), et peuvent demander ou offrir de l'aide via le module **Entraide** (besoins communautaires).

Projet réalisé par un développeur solo dans le cadre du titre professionnel **CDA** (Concepteur Développeur d'Applications).

---

## Stack technique

| Couche | Technologie | Rôle |
|---|---|---|
| Backend | **Rust / Axum** | API REST, logique métier |
| Frontend | **SwiftUI** | Application iOS native |
| Base de données | **PostgreSQL** | Stockage relationnel principal |
| Cache | **Redis** | Sessions, rate limiting, cache |
| Reverse proxy | **Caddy** | TLS automatique, routage |
| Stockage objet | **Cloudflare R2** | Images, fichiers uploadés |
| Conteneurisation | **Docker / Docker Compose** | Dev local et déploiement |
| Monitoring | **Prometheus / Grafana / Loki** | Métriques, dashboards, logs |
| CI/CD | **GitHub Actions** | Tests, build, déploiement auto |

---

## Chiffres clés

| Métrique | Valeur |
|---|---|
| Endpoints REST | ~100 |
| Migrations SQL | 8 |
| Tests d'intégration | 1 042 (23 500 lignes) |
| Couverture de code | > 75 % |
| Modules iOS | 15 |
| Clés de localisation (FR/EN) | 821 |

---

## Navigation

| Page | Contenu |
|---|---|
| [Architecture Technique](01-architecture.md) | Diagrammes, couches, flux de données |
| [Modèle de Données](02-data-model.md) | Schéma PostgreSQL, relations, migrations |
| [API Reference](03-api.md) | Endpoints, requêtes/réponses, codes erreur |
| [Règles Métier](04-business-rules.md) | Logique applicative, cycles de vie, validations |
| [Frontend iOS](05-frontend.md) | Architecture SwiftUI, modules, navigation |
| [Infrastructure & Déploiement](06-infrastructure.md) | Docker, CI/CD, monitoring, réseau |
| [Sécurité & RGPD](07-security.md) | Authentification, chiffrement, conformité |

---

## Environnements

| Environnement | URL / Accès | Hébergement |
|---|---|---|
| **Production** | `api.offrii.com` | Hetzner CX32 |
| **Staging** | `staging.offrii.com` | Hetzner CX32 (même serveur) |
| **Dev local** | `localhost` | Docker Compose |
