# Offrii — Document de Référence Complet

**"N'oublie plus ce que tu veux acheter"**
**Version** : 1.0 — Premier Cercle (MVP)
**Date** : 2026-03-03
**Compliance** : RGPD (UE 2016/679) + OWASP ASVS v4.0.3 (niveau 1) + OWASP Top 10 2021
**Certification** : CDA — Concepteur Développeur d'Applications (RNCP37873)

---

# PARTIE 1 — PRODUCT REQUIREMENTS (PRD)

---

## 1.1 Vision & Problème

### Le Problème
Les achats intentionnels non-urgents sont systématiquement oubliés. Pas les courses du quotidien (lait, pain) — pour ça, des apps existent. Le problème ce sont les achats **réfléchis mais procrastinés** : un écran PC depuis 6 mois, des pantoufles, un câble cassé à remplacer, un livre repéré en passant...

Ces idées surgissent à des moments random (sous la douche, en réunion, en marchant) et disparaissent aussi vite qu'elles sont venues. Le résultat : frustration répétée ("ah merde, j'ai encore oublié"), achats impulsifs de remplacement, et un sentiment permanent d'avoir "un truc à acheter" sans se rappeler quoi.

### Analyse Concurrentielle

| Solution | Problème |
|---|---|
| **Apps wishlist** (Listy, Wisher, Moonsift) | Orientées cadeaux/partage social, nécessitent un lien produit, trop de friction |
| **Apps notes** (Keep, Apple Notes) | Trop généralistes, la liste se perd, pas de structure achat |
| **Apps courses** (Bring!, AnyList) | Pensées pour l'alimentaire récurrent, pas les achats ponctuels |
| **Rappels téléphone** | Pas de contexte (juste "acheter écran"), pas d'enrichissement, pas d'organisation |

### Le Positionnement Offrii
**Offrii = le filet de sécurité pour tes intentions d'achat.**

Ni wishlist sociale, ni liste de courses, ni comparateur de prix. Offrii capture tes idées d'achat à la vitesse de la pensée et te rappelle intelligemment de passer à l'action.

**Différenciation clé** :
- **Capture ultra-rapide** (< 5 sec) — pas besoin de lien, de photo, rien. Juste un nom.
- **Enrichissement progressif** — tu complètes quand tu veux, si tu veux.
- **Rappels intelligents** — pas un spam quotidien, mais des nudges contextuels ("Ça fait 45 jours que tu veux un écran. Tu passes à l'action ?")
- **Cercles de partage** (futur proche) — partager certaines listes avec famille, amis, partenaire.

---

## 1.2 Cible Utilisateur

### Persona Principal — Yassine
- 25 ans, développeur, vie active et occupée
- Pense à des trucs à acheter à des moments random
- N'a jamais le temps/réflexe de noter proprement
- Quand il arrive en magasin ou sur un site, il a oublié
- Utilise principalement son **smartphone**
- Veut une solution aussi rapide que de se parler à soi-même

### Persona Élargi
- Toute personne active (20-45 ans) qui procrastine ses achats non-urgents
- Personnes vivant en couple/coloc qui aimeraient partager certaines listes (cercle futur)

---

## 1.3 User Stories (MVP — Premier Cercle)

### Epic 1 : Capture Ultra-Rapide — CORE

**US-1.1 — Quick Capture**
En tant qu'utilisateur pressé, je veux noter une idée d'achat en **moins de 5 secondes** avec juste un nom.
- AC: L'app s'ouvre directement sur le champ de capture
- AC: Taper un nom + appuyer sur Entrée = item sauvegardé
- AC: Pas de champs obligatoires autres que le nom
- AC: Retour haptique/visuel de confirmation

**US-1.2 — Enrichissement Progressif**
En tant qu'utilisateur, je veux enrichir une note rapide plus tard avec des détails optionnels.
- AC: Cliquer sur un item → écran d'édition
- AC: Champs optionnels : prix estimé, lien URL, catégorie, priorité (1-3), notes libres
- AC: Aucun champ obligatoire à l'enrichissement

**US-1.3 — Capture Détaillée**
En tant qu'utilisateur qui a le temps, je veux pouvoir remplir directement tous les détails à la création.
- AC: Option "+ Détails" visible mais non imposée sur l'écran de capture
- AC: Formulaire extensible sans quitter le flow de capture

### Epic 2 : Organisation & Visualisation

**US-2.1 — Liste Active**
En tant qu'utilisateur, je veux voir ma liste d'achats en attente clairement.
- AC: Vue liste par défaut, triable par : date d'ajout, priorité, catégorie
- AC: Chaque item affiche : nom, catégorie (si définie), badge "depuis X jours", priorité

**US-2.2 — Catégorisation**
En tant qu'utilisateur, je veux catégoriser mes items.
- AC: Catégories prédéfinies : Tech, Maison, Vêtements, Santé, Loisirs, Autre
- AC: Possibilité de créer des catégories custom
- AC: Filtrage par catégorie

**US-2.3 — Marquer comme Acheté**
En tant qu'utilisateur, je veux marquer un item comme acheté.
- AC: Swipe ou bouton pour marquer acheté
- AC: Item déplacé dans un historique consultable
- AC: Possibilité de "désacheter" (remettre dans la liste active)

**US-2.4 — Suppression**
En tant qu'utilisateur, je veux supprimer un item que je ne veux plus acheter.
- AC: Swipe inverse ou bouton supprimer
- AC: Confirmation avant suppression définitive

### Epic 3 : Rappels Intelligents

**US-3.1 — Nudges Temporels**
En tant qu'utilisateur, je veux être rappelé intelligemment de mes items en attente.
- AC: Notification type "Ça fait 30 jours que tu veux acheter un écran. On passe à l'action ?"
- AC: Fréquence configurable par l'utilisateur (quotidien, hebdo, mensuel)
- AC: Les items haute priorité sont rappelés plus souvent
- AC: Les nudges incluent le nom de l'item et le nombre de jours d'attente

**US-3.2 — Badge d'Ancienneté**
En tant qu'utilisateur, je veux voir depuis combien de temps chaque item attend.
- AC: Badge visuel coloré : vert (< 7j), orange (7-30j), rouge (> 30j)
- AC: Le nombre de jours est affiché sur chaque item

### Epic 4 : Compte & Sécurité

**US-4.1 — Inscription/Connexion**
En tant qu'utilisateur, je veux un compte sécurisé pour retrouver mes données.
- AC: Inscription par email + mot de passe
- AC: Authentification sécurisée (JWT RS256 + refresh tokens)
- AC: Validation email

**US-4.2 — RGPD**
En tant qu'utilisateur, je veux le contrôle sur mes données personnelles.
- AC: Mentions légales et politique de confidentialité accessibles
- AC: Possibilité de supprimer son compte et toutes ses données
- AC: Export des données au format JSON

---

## 1.4 Métriques de Succès (MVP)

| Métrique | Cible |
|---|---|
| Temps de capture rapide | < 5 secondes |
| Temps de réponse API | < 200ms |
| Onboarding complet | < 2 minutes |
| Couverture tests | > 70% |
| Compétences CDA démontrables | 11/11 |
| App installable | iOS + Android via Expo |
| Notifications push fonctionnelles | Oui |

---

## 1.5 Hors Scope (Premier Cercle)

- Partage social / cercles (UI) — prévu cercle 1.5
- Capture vocale / photo — prévu cercle 2
- Widget écran d'accueil — prévu cercle 2
- Intégration e-commerce / scraping de prix
- Comparateur de prix
- Intelligence artificielle / suggestions
- Géolocalisation / rappels par lieu
- Système de paiement
- Version web standalone

---

## 1.6 Concept de Partage (Cercle 1.5 — Futur Proche)

Le nom "Offrii" porte en lui l'idée d'**offrir**, de **partager**. Pour le MVP on se concentre sur l'usage personnel, mais l'architecture prévoit :
- Un item appartient à un **utilisateur**
- Un utilisateur pourra (futur) créer des **cercles** (famille, amis, couple)
- Un item pourra (futur) être **partagé** avec un cercle = visible par les membres

**Pour le MVP** : les tables `circles` et `circle_members` sont créées en base mais l'UI de partage n'est pas implémentée.

---

## 1.7 Risques Projet

| Risque | Probabilité | Impact | Mitigation |
|---|---|---|---|
| Rust learning curve pour le backend | Moyenne | Élevé | Commencer par l'API tôt, utiliser Axum (bon écosystème) |
| Notifications push complexité | Moyenne | Moyen | Utiliser Expo Notifications (simplifie énormément) |
| Scope creep (tentation d'ajouter le partage) | Élevée | Élevé | Discipline stricte sur le premier cercle, backlog priorisé |
| Double plateforme iOS/Android | Faible | Moyen | React Native + Expo = un seul build |
| Jury CDA sceptique sur Rust | Faible | Moyen | Préparer argumentaire solide (sécurité, performance, modernité) |

---

# PARTIE 2 — ARCHITECTURE TECHNIQUE

---

## 2.1 Principes d'Architecture

1. **Capture-First Design** : Chaque décision optimise la vitesse de capture (< 5 sec UX, < 200ms API)
2. **Layered Architecture** : Séparation stricte handlers → services → repositories (compétence CDA)
3. **Security by Design (OWASP 2026)** : Argon2id, JWT RS256, rate limiting, input validation, secure headers
4. **RGPD by Design** : Données en Europe (Hetzner), chiffrement, droit à l'oubli, export, consentement
5. **Observable by Default** : OpenTelemetry traces/metrics/logs → Grafana dashboards
6. **Progressive Enhancement** : Modèle de données prêt pour les cercles futurs sans migration breaking

---

## 2.2 Stack Technique

### Backend
| Composant | Technologie | Justification |
|---|---|---|
| Language | **Rust 1.75+** (edition 2021) | Sécurité mémoire, performance, différenciation CDA |
| Framework | **Axum 0.7** | Modern, async (Tokio), excellent écosystème |
| ORM/Queries | **SQLx 0.7** | Compile-time checked SQL, migrations intégrées |
| Database | **PostgreSQL 16** | Relationnelle, compétence CDA SQL |
| Cache/Sessions | **Redis 7** | NoSQL, compétence CDA, sessions, rate limiting |
| Auth | **JWT RS256** (jsonwebtoken) | Asymétrique = plus sécurisé que HS256 |
| Hashing | **Argon2id** (argon2 crate) | OWASP 2026 recommended |
| Validation | **validator** crate | DTO validation en entrée |
| Serialization | **serde** + serde_json | Standard Rust |
| API Docs | **utoipa** | Génération OpenAPI/Swagger automatique |
| Telemetry | **tracing** + **opentelemetry** + **opentelemetry-otlp** | Traces, métriques, logs structurés |
| CRON Jobs | **tokio-cron-scheduler** | Rappels intelligents planifiés |
| Testing | **cargo test** + sqlx fixtures | Tests unitaires + intégration |

### Frontend
| Composant | Technologie | Justification |
|---|---|---|
| Framework | **React Native 0.73+** / **Expo SDK 50+** | Cross-platform, un seul codebase |
| Language | **TypeScript 5.x** | Type-safety |
| Navigation | **Expo Router** | File-based routing, convention over config |
| State | **Zustand** | Léger, simple, pas de boilerplate |
| HTTP Client | **Axios** + interceptors | JWT auto-refresh |
| UI | **React Native Paper** | Material Design, accessible, RGAA compatible |
| Notifications | **expo-notifications** | Push notifications simplifiées |
| Secure Storage | **expo-secure-store** | Stockage sécurisé des tokens |
| Animations | **react-native-reanimated** | Swipe gestures, retour haptique |
| Testing | **Jest** + React Native Testing Library | Tests composants + logique |

### Infrastructure & Observabilité
| Composant | Technologie | Justification |
|---|---|---|
| Hosting | **Hetzner VPS** (Europe, Falkenstein/Nuremberg) | RGPD (données EU), économique (~€4-8/mois) |
| Containerisation | **Docker** + **Docker Compose** | Reproductible, isolation |
| CI/CD | **GitHub Actions** | Lint, test, build, deploy automatisé |
| Reverse Proxy | **Caddy** | HTTPS automatique (Let's Encrypt), HTTP/2, simple |
| Traces/Metrics | **OpenTelemetry** (OTLP) | Standard ouvert, vendor-agnostic |
| Collector | **OpenTelemetry Collector** | Agrège traces, métriques, logs |
| Dashboards | **Grafana** | Visualisation, alerting |
| Metrics Store | **Prometheus** | Stockage métriques |
| Traces Store | **Grafana Tempo** | Stockage traces distribué |
| Logs Store | **Grafana Loki** | Agrégation logs structurés |
| Mobile Build | **Expo EAS Build** | Cloud builds iOS + Android |
| Backup | **pg_dump** + cron → Hetzner Storage Box | Backup automatisé quotidien |

### Pourquoi Rust pour le CDA
- Montre de l'**ambition technique** au jury
- **Sécurité mémoire** = argument béton pour le bloc "application sécurisée"
- **Performance** = API ultra-rapide, pertinent pour la capture < 5 sec
- Peu de candidats CDA utilisent Rust → **différenciation forte**

### Pourquoi React Native plutôt que SwiftUI/Kotlin
- **Un seul codebase** pour iOS et Android → réaliste en solo pour un CDA
- **SwiftUI + Kotlin = 2 apps** → trop ambitieux pour le scope
- **L'écosystème JS/TS** complète bien un backend Rust (diversité technique au CDA)
- **Expo** simplifie le build, les notifications push, et le déploiement

---

## 2.3 Architecture Haut Niveau

```
┌─────────────────────────────────────────────────────────────────┐
│                        Mobile Client                            │
│                 React Native / Expo / TypeScript                 │
│   ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐      │
│   │ Capture  │  │  Liste   │  │  Detail  │  │ Profile  │      │
│   │ Screen   │  │  Screen  │  │  Screen  │  │ Screen   │      │
│   └────┬─────┘  └────┬─────┘  └────┬─────┘  └────┬─────┘      │
│        └──────────────┴──────────────┴──────────────┘           │
│                          │ Zustand + Axios                      │
└──────────────────────────┼──────────────────────────────────────┘
                           │ HTTPS (TLS 1.3)
                           ▼
┌──────────────────────────────────────────────────────────────────┐
│                     Hetzner VPS (Europe)                         │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │                    Caddy (Reverse Proxy)                   │ │
│  │              HTTPS auto (Let's Encrypt)                    │ │
│  │              Security Headers (OWASP)                      │ │
│  └──────────────────────┬─────────────────────────────────────┘ │
│                         ▼                                       │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │                  Offrii API (Axum/Rust)                    │ │
│  │  ┌─────────────────────────────────────────────────────┐  │ │
│  │  │  Middleware: [CORS] [JWT Auth] [Rate Limit] [OTEL]  │  │ │
│  │  └─────────────────────┬───────────────────────────────┘  │ │
│  │                        ▼                                   │ │
│  │  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐       │ │
│  │  │   Handlers   │ │   Services   │ │ Repositories │       │ │
│  │  │ (Controllers)│→│(Biz Logic)   │→│ (Data Access) │       │ │
│  │  └──────────────┘ └──────────────┘ └──────┬───────┘       │ │
│  │                                           │                │ │
│  │  ┌──────────────────────┐ ┌──────────────────────┐        │ │
│  │  │  CRON: Reminder Job  │ │  OpenTelemetry SDK   │        │ │
│  │  │  (tokio-cron)        │ │  (traces+metrics)    │        │ │
│  │  └──────────────────────┘ └──────────┬───────────┘        │ │
│  └───────────────────────────────────────┼────────────────────┘ │
│           │               │              │                      │
│           ▼               ▼              ▼                      │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────────────┐    │
│  │ PostgreSQL 16│ │   Redis 7    │ │  OTEL Collector      │    │
│  │ (SQL Data)   │ │ (Cache/NoSQL)│ │  → Prometheus        │    │
│  │              │ │              │ │  → Loki              │    │
│  └──────────────┘ └──────────────┘ │  → Tempo             │    │
│                                    └──────────┬───────────┘    │
│                                               ▼                 │
│                                    ┌──────────────────────┐    │
│                                    │      Grafana         │    │
│                                    │   (Dashboards)       │    │
│                                    │   (Alerting)         │    │
│                                    └──────────────────────┘    │
└──────────────────────────────────────────────────────────────────┘
```

---

## 2.4 Structure Projet Backend

```
offrii-api/
├── Cargo.toml
├── Dockerfile
├── .env.example
├── migrations/
│   ├── 001_create_users.sql
│   ├── 002_create_categories.sql
│   ├── 003_create_items.sql
│   ├── 004_create_push_tokens.sql
│   └── 005_create_circles.sql           # Préparation cercle 1.5
├── src/
│   ├── main.rs                           # Entry point, server + telemetry setup
│   ├── lib.rs
│   ├── config/
│   │   ├── mod.rs
│   │   ├── app.rs                        # App config (env vars)
│   │   ├── database.rs                   # PgPool + Redis connection
│   │   └── telemetry.rs                  # OpenTelemetry setup (OTLP exporter)
│   ├── handlers/                         # HTTP handlers (controllers)
│   │   ├── mod.rs
│   │   ├── auth.rs                       # register, login, refresh, logout
│   │   ├── items.rs                      # CRUD + purchase + restore
│   │   ├── categories.rs                # CRUD
│   │   ├── users.rs                      # profile, settings, delete, export
│   │   ├── push_tokens.rs               # register/unregister device
│   │   └── health.rs                     # GET /health (DB + Redis check)
│   ├── services/                         # Business logic
│   │   ├── mod.rs
│   │   ├── auth.rs
│   │   ├── items.rs
│   │   ├── categories.rs
│   │   ├── users.rs
│   │   └── reminder.rs                   # Logique nudges intelligents
│   ├── repositories/                     # Data access (SQL queries)
│   │   ├── mod.rs
│   │   ├── user_repo.rs
│   │   ├── item_repo.rs
│   │   ├── category_repo.rs
│   │   └── push_token_repo.rs
│   ├── models/                           # Domain models & DTOs
│   │   ├── mod.rs
│   │   ├── user.rs                       # User, CreateUser, UpdateUser
│   │   ├── item.rs                       # Item, CreateItem, UpdateItem, ItemStatus
│   │   ├── category.rs
│   │   ├── auth.rs                       # LoginRequest, RegisterRequest, TokenPair
│   │   └── reminder.rs                   # ReminderSettings, NudgePayload
│   ├── middleware/
│   │   ├── mod.rs
│   │   ├── auth.rs                       # JWT extraction + validation RS256
│   │   ├── rate_limit.rs                # Redis sliding window
│   │   ├── security_headers.rs          # OWASP secure headers
│   │   └── request_tracing.rs           # OpenTelemetry span injection
│   ├── jobs/
│   │   ├── mod.rs
│   │   └── reminder_job.rs              # CRON: calcule + envoie nudges
│   ├── errors/
│   │   ├── mod.rs
│   │   └── app_error.rs                 # Error enum + IntoResponse
│   └── utils/
│       ├── mod.rs
│       ├── jwt.rs                        # RS256 sign/verify, key management
│       ├── hash.rs                       # Argon2id hash/verify
│       ├── redis.rs                      # Redis helpers
│       └── expo_push.rs                  # Expo Push API client
└── tests/
    ├── common/mod.rs
    ├── auth_tests.rs
    ├── item_tests.rs
    ├── reminder_tests.rs
    └── integration/
        └── api_tests.rs
```

---

## 2.5 Structure Projet Frontend

```
offrii-mobile/
├── package.json
├── tsconfig.json
├── app.json                              # Expo config
├── eas.json                              # EAS Build config
├── app/                                  # Expo Router (file-based)
│   ├── _layout.tsx                       # Root layout (auth guard)
│   ├── index.tsx                         # → Redirect to capture or login
│   ├── (auth)/
│   │   ├── _layout.tsx
│   │   ├── login.tsx
│   │   └── register.tsx
│   └── (tabs)/
│       ├── _layout.tsx                   # Tab navigator
│       ├── capture.tsx                   # Quick capture (home tab)
│       ├── list.tsx                      # Items list
│       ├── list/[id].tsx                 # Item detail/edit
│       └── profile.tsx                   # Settings & profile
├── src/
│   ├── api/
│   │   ├── client.ts                     # Axios instance + JWT interceptors
│   │   ├── auth.ts
│   │   ├── items.ts
│   │   └── categories.ts
│   ├── stores/
│   │   ├── authStore.ts                  # Zustand
│   │   ├── itemStore.ts
│   │   └── categoryStore.ts
│   ├── components/
│   │   ├── QuickCaptureInput.tsx         # Core capture component
│   │   ├── ItemCard.tsx
│   │   ├── AgeBadge.tsx                  # Vert/Orange/Rouge
│   │   ├── PriorityIndicator.tsx
│   │   ├── CategoryChip.tsx
│   │   ├── SwipeableRow.tsx
│   │   └── EmptyState.tsx
│   ├── hooks/
│   │   ├── useAuth.ts
│   │   ├── useItems.ts
│   │   └── useCategories.ts
│   ├── theme/
│   │   ├── colors.ts
│   │   ├── typography.ts
│   │   └── spacing.ts
│   ├── types/
│   │   ├── item.ts
│   │   ├── user.ts
│   │   └── category.ts
│   └── utils/
│       ├── dates.ts
│       └── notifications.ts
└── __tests__/
    ├── components/
    └── stores/
```

---

## 2.6 Modèle de Données (PostgreSQL)

```sql
-- ============================================
-- USERS
-- ============================================
CREATE TABLE users (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email            VARCHAR(255) UNIQUE NOT NULL,
    password_hash    VARCHAR(255) NOT NULL,           -- Argon2id
    display_name     VARCHAR(100),
    reminder_freq    VARCHAR(20) DEFAULT 'weekly',    -- daily, weekly, monthly
    reminder_time    TIME DEFAULT '10:00:00',
    created_at       TIMESTAMPTZ DEFAULT NOW(),
    updated_at       TIMESTAMPTZ DEFAULT NOW()
);

-- ============================================
-- CATEGORIES
-- ============================================
CREATE TABLE categories (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id          UUID REFERENCES users(id) ON DELETE CASCADE,
    name             VARCHAR(100) NOT NULL,
    icon             VARCHAR(50),
    is_default       BOOLEAN DEFAULT FALSE,
    position         INTEGER DEFAULT 0,
    created_at       TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(user_id, name)
);

-- ============================================
-- ITEMS (coeur de l'app)
-- ============================================
CREATE TABLE items (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id          UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name             VARCHAR(255) NOT NULL,            -- seul champ obligatoire
    description      TEXT,
    url              VARCHAR(2048),
    estimated_price  DECIMAL(10,2),
    priority         SMALLINT DEFAULT 2 CHECK (priority BETWEEN 1 AND 3),
    category_id      UUID REFERENCES categories(id) ON DELETE SET NULL,
    status           VARCHAR(20) DEFAULT 'active',     -- active, purchased, deleted
    purchased_at     TIMESTAMPTZ,
    created_at       TIMESTAMPTZ DEFAULT NOW(),
    updated_at       TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_items_user_status ON items(user_id, status);
CREATE INDEX idx_items_user_priority ON items(user_id, priority);
CREATE INDEX idx_items_created_at ON items(created_at);

-- ============================================
-- PUSH TOKENS
-- ============================================
CREATE TABLE push_tokens (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id          UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token            VARCHAR(500) NOT NULL,
    platform         VARCHAR(10) NOT NULL,             -- ios, android
    created_at       TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(user_id, token)
);

-- ============================================
-- CIRCLES (Cercle 1.5 — tables prêtes, pas d'UI MVP)
-- ============================================
CREATE TABLE circles (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name             VARCHAR(100) NOT NULL,
    owner_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at       TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE circle_members (
    circle_id        UUID NOT NULL REFERENCES circles(id) ON DELETE CASCADE,
    user_id          UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role             VARCHAR(20) DEFAULT 'member',
    joined_at        TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (circle_id, user_id)
);
```

### Redis (NoSQL — compétence CDA)
```
refresh:{user_id}:{token_jti}     → "valid"      TTL 7 days
rate:{ip}:{endpoint}              → counter       TTL 1 min
cache:items:{user_id}             → JSON          TTL 5 min
cache:categories:{user_id}        → JSON          TTL 1 hour
```

---

## 2.7 API REST

### Base URL: `https://api.offrii.app/v1`

### Authentication
```
POST   /auth/register       { email, password, display_name? }     → 201
POST   /auth/login           { email, password }                    → 200
POST   /auth/refresh         { refresh_token }                      → 200
POST   /auth/logout          (Bearer)                               → 204
```

### Items (Bearer required)
```
GET    /items                ?status=active&sort=created_at&order=desc&category_id=xxx&page=1&per_page=50
POST   /items                { name }                               ← Quick capture
POST   /items                { name, url?, price?, priority?, category_id?, description? }
GET    /items/:id
PUT    /items/:id            { ...partial update }                  ← Enrichissement
PATCH  /items/:id/purchase                                          ← Marquer acheté
PATCH  /items/:id/restore                                           ← Remettre actif
DELETE /items/:id
```

### Categories (Bearer required)
```
GET    /categories
POST   /categories           { name, icon? }
PUT    /categories/:id       { name?, icon?, position? }
DELETE /categories/:id
```

### User (Bearer required)
```
GET    /users/me
PUT    /users/me             { display_name?, reminder_freq?, reminder_time? }
DELETE /users/me                                                    ← RGPD suppression
GET    /users/me/export                                             ← RGPD export
```

### Push Tokens / Health
```
POST   /push-tokens          { token, platform }
DELETE /push-tokens/:token
GET    /health
```

### Standards
- **Auth** : Bearer JWT RS256
- **Errors** : `{ error: { code, message, details? } }`
- **Dates** : ISO 8601 UTC
- **IDs** : UUID v4
- **Pagination** : `?page=1&per_page=50`
- **Versioning** : `/v1`

---

## 2.8 Sécurité (OWASP 2026 + RGPD)

### Authentication
- **JWT RS256** : Clés asymétriques RSA 2048-bit minimum
- **Access token** : TTL 15 min, en mémoire (Zustand)
- **Refresh token** : TTL 7 jours, expo-secure-store + Redis
- **Token rotation** : Nouveau refresh à chaque refresh (revoke ancien)

### Password (Argon2id — OWASP 2026)
- Params : m=19456 (19MB), t=2, p=1
- Min 8 chars, pas de max restrictif

### Secure Headers (Caddy)
```
Strict-Transport-Security: max-age=31536000; includeSubDomains; preload
X-Content-Type-Options: nosniff
X-Frame-Options: DENY
Content-Security-Policy: default-src 'self'
Referrer-Policy: strict-origin-when-cross-origin
Permissions-Policy: camera=(), microphone=(), geolocation=()
```

### Rate Limiting (Redis)
- Global : 100 req/min par IP
- Auth : 10 req/min par IP

### RGPD
- Hébergement EU (Hetzner Allemagne)
- `DELETE /users/me` → CASCADE suppression totale
- `GET /users/me/export` → JSON complet
- Minimisation des données, pas de tracking tiers
- TLS 1.3 transit + disk encryption repos

---

## 2.9 Observabilité (OpenTelemetry + Grafana)

```
Offrii API (Rust)
  │ tracing + opentelemetry-otlp
  ▼
OpenTelemetry Collector
  ├──→ Prometheus (métriques) ──→ Grafana
  ├──→ Grafana Tempo (traces)  ──→ Grafana
  └──→ Grafana Loki (logs)     ──→ Grafana
```

### Métriques
- `http_request_duration_seconds`, `http_requests_total`
- `items_created_total`, `items_purchased_total`, `auth_login_total`, `reminder_sent_total`
- CPU, RAM, disk (node_exporter), PostgreSQL (pg_exporter), Redis (redis_exporter)

### Dashboards
1. API Overview : req/sec, latence p50/p95/p99, error rate
2. Business Metrics : items/jour, users actifs, achats
3. Infrastructure : CPU, RAM, disk, DB connections
4. Alerting : error > 5%, latence p95 > 500ms, disk > 80%

---

## 2.10 Rappels Intelligents (Nudges)

### Fonctionnement
1. CRON Job (tokio-cron-scheduler) tourne toutes les heures
2. Sélectionne les users dont `reminder_time` ≈ heure actuelle
3. Calcule un **score de nudge** par item :

```
score = (priority_weight × priority) + (age_weight × jours_attente)

priority_weight: haute=×3, moyenne=×2, basse=×1
age_weight: <7j=×0, 7-30j=×1, >30j=×2
```

4. Top 3 items envoyés en notification push (Expo Push API)
5. Message : "Ça fait {n} jours que tu veux acheter {nom}. On passe à l'action ?"
6. Respecte la fréquence user (daily/weekly/monthly)

---

## 2.11 Déploiement

### Hetzner VPS
- CX21 (~€4.50/mois) ou CX31 (~€8.50/mois)
- 2-4 vCPU, 4-8 GB RAM, 40-80 GB SSD
- Falkenstein (Allemagne) → RGPD EU
- Ubuntu 24.04 LTS

### Docker Compose Production
Services : Caddy, API Rust, PostgreSQL 16, Redis 7, OTEL Collector, Prometheus, Loki, Tempo, Grafana

### CI/CD (GitHub Actions)
```
Push any branch → lint + test
Push develop    → build + deploy staging + smoke tests
Tag vX.Y.Z     → build + deploy prod + EAS Build mobile
```

### Backup
- pg_dump quotidien 3h00 → Hetzner Storage Box (~€3/mois)
- Rétention 7 jours glissants
- Test restauration mensuel

---

## 2.12 Tests

| Couche | Type | Outil | Cible |
|---|---|---|---|
| Backend | Unit | cargo test | Services, utils (Argon2, JWT, nudge) > 80% |
| Backend | Integration | sqlx::test | Tous les endpoints API |
| Backend | Coverage | cargo tarpaulin | Rapport de couverture |
| Frontend | Unit | Jest | Stores, utils > 70% |
| Frontend | Component | React Native Testing Library | QuickCapture, ItemCard, AgeBadge |
| E2E | Scénarios | Detox ou Maestro | Inscription → capture → liste → achat |

---

# PARTIE 3 — COUVERTURE CDA

---

## 3.1 Compétences CDA (RNCP37873)

### Bloc 1 — Développer une application sécurisée
| Compétence | Implémentation | Preuve |
|---|---|---|
| Installer/configurer environnement | Docker, Rust toolchain, Expo, GitHub Actions | docker-compose.yml, CI config |
| Développer interfaces utilisateur | React Native + Expo Router + Paper | 4 écrans, composants réutilisables |
| Développer composants métier | Capture, catégorisation, nudges intelligents | Services Rust, scoring algorithm |
| Gestion de projet informatique | Jira Kanban, Git flow, documentation | Board Jira, commits, PRD |

### Bloc 2 — Concevoir et développer une application sécurisée organisée en couches
| Compétence | Implémentation | Preuve |
|---|---|---|
| Analyser besoins / maquetter | PRD + maquettes Figma | Ce document + Figma |
| Architecture logicielle en couches | Handlers → Services → Repositories | Structure projet documentée |
| BDD relationnelle | PostgreSQL : 6 tables, index, FK, CASCADE | Migrations SQL |
| Accès données SQL et NoSQL | SQLx (PostgreSQL) + Redis (cache, sessions) | Code repositories + redis utils |

### Bloc 3 — Préparer le déploiement d'une application sécurisée
| Compétence | Implémentation | Preuve |
|---|---|---|
| Plans de tests | Unit + Integration + E2E | cargo test, jest, Detox |
| Documenter déploiement | README, OpenAPI/Swagger, ADRs | utoipa auto-doc, scripts deploy |
| Démarche DevOps | Docker, CI/CD, OpenTelemetry, Grafana | Pipeline complète, dashboards |

**Verdict : 11/11 compétences CDA couvertes.**

---

## 3.2 Gestion de Projet

**Méthodologie** : Kanban (projet solo — plus adapté que Scrum)
**Outil** : Jira (board Kanban)
**Colonnes** : Backlog → To Do → In Progress → Review → Done
**Git** : Git flow (main, develop, feature/*)
