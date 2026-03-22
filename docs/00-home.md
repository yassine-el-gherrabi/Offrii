# Offrii -- Documentation Technique

## Vision produit

Offrii est une application iOS de **wishlist et d'entraide communautaire**.

Les utilisateurs creent des listes d'envies, les partagent avec leurs proches via des **cercles** (groupes prives), et peuvent demander ou offrir de l'aide via le module **Entraide** (besoins communautaires).

Projet realise par un developpeur solo dans le cadre du titre professionnel **CDA** (Concepteur Developpeur d'Applications).

---

## Stack technique

| Couche | Technologie | Role |
|---|---|---|
| Backend | **Rust / Axum** | API REST, logique metier |
| Frontend | **SwiftUI** | Application iOS native |
| Base de donnees | **PostgreSQL** | Stockage relationnel principal |
| Cache | **Redis** | Sessions, rate limiting, cache |
| Reverse proxy | **Caddy** | TLS automatique, routage |
| Stockage objet | **Cloudflare R2** | Images, fichiers uploades |
| Conteneurisation | **Docker / Docker Compose** | Dev local et deploiement |
| Monitoring | **Prometheus / Grafana / Loki** | Metriques, dashboards, logs |
| CI/CD | **GitHub Actions** | Tests, build, deploiement auto |

---

## Chiffres cles

| Metrique | Valeur |
|---|---|
| Endpoints REST | ~100 |
| Migrations SQL | 8 |
| Tests d'integration | 1 042 (23 500 lignes) |
| Couverture de code | > 75 % |
| Modules iOS | 15 |
| Cles de localisation (FR/EN) | 821 |

---

## Navigation

| Page | Contenu |
|---|---|
| [Architecture Technique](01-architecture.md) | Diagrammes, couches, flux de donnees |
| [Modele de Donnees](02-data-model.md) | Schema PostgreSQL, relations, migrations |
| [API Reference](03-api.md) | Endpoints, requetes/reponses, codes erreur |
| [Regles Metier](04-business-rules.md) | Logique applicative, cycles de vie, validations |
| [Frontend iOS](05-frontend.md) | Architecture SwiftUI, modules, navigation |
| [Infrastructure & Deploiement](06-infrastructure.md) | Docker, CI/CD, monitoring, reseau |
| [Securite & RGPD](07-security.md) | Authentification, chiffrement, conformite |

---

## Environnements

| Environnement | URL / Acces | Hebergement |
|---|---|---|
| **Production** | `api.offrii.com` | Hetzner CX32 |
| **Staging** | `staging.offrii.com` | Hetzner CX32 (meme serveur) |
| **Dev local** | `localhost` | Docker Compose |
