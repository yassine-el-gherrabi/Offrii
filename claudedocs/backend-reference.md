# Offrii Backend Reference

> Document auto-généré depuis le code source du backend Rust.
> Objectif : servir de référence complète pour brainstormer la refonte frontend (SwiftUI) et la feature Cercles.

---

## Table des matières

1. [Stack technique](#1-stack-technique)
2. [Schéma BDD (Mermaid ERD)](#2-schéma-bdd)
3. [Règles métier](#3-règles-métier)
4. [État des Cercles](#4-état-des-cercles)
5. [Constantes de référence](#5-constantes-de-référence)

---

## 1. Stack technique

| Composant | Technologie | Détails |
|-----------|-------------|---------|
| Langage | **Rust** | Édition 2021 |
| Framework HTTP | **Axum 0.8** | Async, extractors, State pattern |
| Runtime async | **Tokio** | Multi-threaded runtime |
| Base de données | **PostgreSQL** | Via SQLx (compile-time checked queries) |
| Cache / Sessions | **Redis** | Cache, rate limiting, JTI blacklist, token versioning |
| Auth | **JWT RS256** | Access token 15min, Refresh token 7 jours |
| Hashing MDP | **Argon2id v0x13** | m=19456 KiB, t=2, p=1 — sur blocking thread |
| Email | **Resend API** | Password reset codes |
| Push notifications | **Expo Push API** | Batch sending (chunks de 100) |
| CRON | **tokio-cron-scheduler** | Expression `0 0 * * * *` (toutes les heures, minute 0) |
| Migrations | **SQLx CLI** | 11 migrations SQL |
| Logging | **tracing** + **tracing-subscriber** | JSON structuré, défaut `info` (configurable via `RUST_LOG`) |
| Middleware HTTP | **TraceLayer** | Tracing automatique des requêtes HTTP |

### Variables d'environnement

| Variable | Requis | Défaut | Description |
|----------|--------|--------|-------------|
| `DATABASE_URL` | Oui | — | URL PostgreSQL |
| `REDIS_URL` | Non | `redis://localhost:6379` | URL Redis |
| `API_PORT` | Non | `3000` | Port HTTP |
| `RESEND_API_KEY` | Oui | — | Clé API Resend |
| `EMAIL_FROM` | Non | `Offrii <noreply@offrii.com>` | Adresse expéditeur |
| `JWT_PRIVATE_KEY_FILE` | Prod | — | Chemin PEM clé privée RSA |
| `JWT_PUBLIC_KEY_FILE` | Prod | — | Chemin PEM clé publique RSA |
| `RUST_LOG` | Non | `info` | Niveau de log (fallback gracieux si valeur invalide) |

> En debug, les clés JWT sont auto-générées (éphémères) si les fichiers PEM ne sont pas fournis.

### Architecture en couches

```
Handlers (Axum extractors, validation)
  → Services (logique métier, traits)
    → Repositories (SQL via SQLx, traits)
      → PostgreSQL / Redis
```

Toutes les couches utilisent des **traits** pour l'injection de dépendances (13 traits définis dans `traits.rs`), permettant le mocking en tests.

---

## 2. Schéma BDD

### Diagramme ERD

```mermaid
erDiagram
    users {
        uuid id PK
        varchar email UK "NOT NULL"
        varchar password_hash "NOT NULL"
        varchar display_name "nullable"
        varchar reminder_freq "NOT NULL, default 'weekly' CHECK(never|daily|weekly|monthly)"
        time reminder_time "NOT NULL, default 10:00"
        varchar timezone "NOT NULL, default 'UTC'"
        smallint utc_reminder_hour "NOT NULL, default 10"
        varchar locale "NOT NULL, default 'fr'"
        int token_version "NOT NULL, default 1"
        timestamptz created_at "NOT NULL, default now()"
        timestamptz updated_at "NOT NULL, default now()"
    }

    items {
        uuid id PK
        uuid user_id FK "NOT NULL → users.id CASCADE"
        varchar name "NOT NULL"
        text description "nullable"
        varchar url "nullable"
        decimal estimated_price "nullable"
        smallint priority "NOT NULL, default 2 CHECK(1-3)"
        uuid category_id FK "nullable → categories.id SET NULL"
        varchar status "NOT NULL, default 'active' CHECK(active|purchased|deleted)"
        timestamptz purchased_at "nullable, auto-set par trigger"
        timestamptz created_at "NOT NULL, default now()"
        timestamptz updated_at "NOT NULL, default now()"
    }

    categories {
        uuid id PK
        uuid user_id FK "nullable → users.id CASCADE"
        varchar name "NOT NULL, max 100"
        varchar icon "nullable, max 50"
        boolean is_default "NOT NULL, default false"
        int position "NOT NULL, default 0"
        timestamptz created_at "NOT NULL, default now()"
    }

    push_tokens {
        uuid id PK
        uuid user_id FK "NOT NULL → users.id CASCADE"
        varchar token "NOT NULL"
        varchar platform "NOT NULL CHECK(ios|android)"
        timestamptz created_at "NOT NULL, default now()"
    }

    refresh_tokens {
        uuid id PK
        uuid user_id FK "NOT NULL → users.id CASCADE"
        varchar token_hash "NOT NULL, UNIQUE"
        timestamptz expires_at "NOT NULL"
        timestamptz revoked_at "nullable"
        timestamptz created_at "NOT NULL, default now()"
    }

    circles {
        uuid id PK
        varchar name "NOT NULL, max 100"
        uuid owner_id FK "NOT NULL → users.id CASCADE"
        timestamptz created_at "NOT NULL, default now()"
    }

    circle_members {
        uuid circle_id PK_FK "NOT NULL → circles.id CASCADE"
        uuid user_id PK_FK "NOT NULL → users.id CASCADE"
        varchar role "NOT NULL, default 'member', CHECK(owner|member)"
        timestamptz joined_at "NOT NULL, default now()"
    }

    users ||--o{ items : "possède"
    users ||--o{ categories : "possède"
    users ||--o{ push_tokens : "a des tokens"
    users ||--o{ refresh_tokens : "a des sessions"
    users ||--o{ circles : "est owner"
    users ||--o{ circle_members : "est membre"
    categories ||--o{ items : "contient"
    circles ||--o{ circle_members : "a des membres"
```

### Indexes notables

| Table | Index | Colonnes | Condition |
|-------|-------|----------|-----------|
| `items` | `idx_items_user_status` | `(user_id, status)` | — |
| `items` | `idx_items_user_priority` | `(user_id, priority)` | — |
| `items` | `idx_items_created_at` | `(created_at)` | — |
| `items` | `idx_items_category_id` | `(category_id)` | — |
| `categories` | `uq_categories_user_name` | `(user_id, name)` | `WHERE user_id IS NOT NULL` |
| `categories` | `uq_categories_default_name` | `(name)` | `WHERE user_id IS NULL` |
| `push_tokens` | unique | `(user_id, token)` | — |
| `refresh_tokens` | `idx_refresh_tokens_user_id` | `(user_id)` | — |
| `refresh_tokens` | `idx_refresh_tokens_active_expires` | `(expires_at)` | `WHERE revoked_at IS NULL` |
| `refresh_tokens` | unique | `(token_hash)` | — |
| `users` | idx | `(utc_reminder_hour)` | `WHERE reminder_freq != 'never'` |

### Triggers SQL

| Trigger | Table | Action |
|---------|-------|--------|
| `trg_users_updated_at` | `users` | Auto-update `updated_at` sur UPDATE |
| `trg_items_updated_at` | `items` | Auto-update `updated_at` sur UPDATE |
| `trg_items_set_purchased_at` | `items` | Auto-set `purchased_at` quand status → `purchased`, auto-clear quand status quitte `purchased` |
| `trg_circles_add_owner_member` | `circles` | Auto-insert le owner dans `circle_members` avec role `owner` |

### Catégories par défaut (seed)

6 catégories insérées avec `user_id IS NULL` et `is_default = true` :

| Position | Nom | Icon |
|----------|-----|------|
| 1 | Tech | `laptop` |
| 2 | Mode | `tshirt` |
| 3 | Maison | `home` |
| 4 | Loisirs | `gamepad` |
| 5 | Santé | `heart` |
| 6 | Autre | `tag` |

> À l'inscription, ces catégories sont copiées vers le nouvel utilisateur via `copy_defaults_for_user()`.

---

## 3. Règles métier

### 3.1 Authentification

#### Flow complet

```
Register → Login → [Access Token 15min] → Refresh → [New Pair]
                                        → Logout → [Blacklist JTI + Revoke ALL Refresh]
                                        → Change Password → [Invalidate All Tokens] → 204

Forgot Password → [Rate limit 60s] → [Email code 6 digits, TTL 30min Redis]
               → Reset Password → [Verify SHA256(code) + New password + Invalidate All Tokens] → 204
```

#### Politique de mot de passe (OWASP 2024)

Vérifications dans cet ordre (premier échec retourné) :

1. **Longueur** : 8–128 caractères (comptage Unicode via `chars().count()`, pas bytes)
2. **Mots de passe courants** : rejeté si dans le top 10k+ (liste embarquée dans le binaire via `include_str!`, comparaison case-insensitive, chargée lazily via `LazyLock`)
3. **HIBP** : rejeté si trouvé dans la base de données de brèches (fail-open si API indisponible)
4. Pas d'exigence de complexité (majuscules, chiffres, symboles) — conforme OWASP

#### HIBP (Have I Been Pwned) — détails

- **k-Anonymity** : le mot de passe est hashé en SHA-1, seuls les 5 premiers caractères (prefix) sont envoyés à l'API
- **URL** : `https://api.pwnedpasswords.com/range/{prefix}`
- **Header** : `Add-Padding: true` (protection vie privée)
- **User-Agent** : `offrii-backend/0.1`
- **Timeout** : 5 secondes
- **Comparaison** : case-insensitive du suffix SHA-1
- **Fail-open** : si l'API est indisponible, le mot de passe est accepté (`Ok(false)`)

#### Hashing des mots de passe (Argon2id)

| Paramètre | Valeur |
|-----------|--------|
| Algorithme | Argon2id v0x13 |
| Memory cost | 19 456 KiB (~19 MB) |
| Time cost | 2 itérations |
| Parallelism | 1 |
| Salt | Généré via `OsRng` (cryptographiquement sûr) |
| Format | PHC string : `$argon2id$v=19$m=19456,t=2,p=1$...` |
| Exécution | `spawn_blocking` pour ne pas bloquer le runtime Tokio |

#### Sécurité tokens

| Mécanisme | Détail |
|-----------|--------|
| Algorithme JWT | RS256 (RSA 2048-bit) |
| Access token TTL | 15 minutes (900s) |
| Refresh token TTL | 7 jours (604 800s) |
| Stockage refresh | **SHA-256 hex digest** en BDD (`token_hash`), le token brut est retourné au client |
| Max refresh tokens/user | 5 — à chaque login, les tokens excédentaires les plus anciens sont révoqués |
| Token versioning | Champ `token_version` dans JWT claims, incrémenté à chaque changement/reset de MDP |
| JTI blacklist | Redis `SET NX EX` avec TTL = temps restant du token (clé : `blacklist:{jti}`) |
| Issuer | `offrii-api` |
| Audience | `offrii-app` |

#### Middleware d'authentification (AuthUser extractor)

L'extracteur `AuthUser` est appliqué sur toutes les routes protégées. Il :

1. Parse le header `Authorization: Bearer <token>` (scheme case-insensitive)
2. Valide le JWT (signature RS256, expiration, issuer, audience, token_type = access)
3. Vérifie la révocation via Redis **en pipeline** (une seule requête, 2 commandes) :
   - `EXISTS blacklist:{jti}` → si 1, token révoqué (401)
   - `GET tkver:{user_id}` → si `token_version` dans le JWT < valeur Redis, token révoqué (401)
4. **Fail-open** : si Redis est indisponible, les vérifications de révocation sont sautées (le token reste valide tant que la signature et l'expiration sont OK)

Retourne `AuthUser { user_id, jti, exp }`.

#### Protection anti-timing

À chaque login, le hash Argon2 est vérifié même si l'email n'existe pas (dummy hash) pour empêcher l'énumération d'utilisateurs.

#### Invalidation globale des tokens

Déclenchée par : changement de MDP, reset password.
Mécanisme en 3 étapes :

1. `token_version` incrémenté en BDD (`INCREMENT_TOKEN_VERSION`)
2. Broadcast Redis : `SET tkver:{user_id} {new_version} EX 900` (TTL = ACCESS_TOKEN_TTL)
3. Révocation de tous les refresh tokens en BDD

#### Forgot/Reset password

- **Rate limit** : Redis `SET NX EX 60` sur clé `pwreset:rate:{email}` — max 1 requête/60s par email. Si rate-limited, retourne OK silencieusement (anti-énumération)
- **Code** : 6 chiffres (`format!("{:06}", rand::random_range(0..1_000_000))`)
- **Stockage** : le code est **hashé en SHA-256** avant stockage en Redis (clé `pwreset:{email}`, TTL 30 min)
- **Vérification** : le code soumis est hashé et comparé au hash stocké
- **Email** : envoyé en tâche asynchrone (`tokio::spawn`), ne bloque pas la réponse
  - Subject : `"Your Offrii password reset code"`
  - Body HTML : `<h2>Password Reset</h2><p>Your code: <strong>{code}</strong></p><p>Valid for 30 minutes.</p>`
  - Langue : anglais (non localisé)
- **Cleanup** : après reset réussi, les clés Redis `pwreset:{email}` et `pwreset:rate:{email}` sont supprimées
- Le reset invalide tous les tokens existants (même flow que change-password)
- **Réponses** : `forgot-password` retourne toujours `200 OK` (même si email inexistant). `reset-password` retourne `204 No Content`

### 3.2 Items (Wishlist)

#### CRUD

| Opération | Détails |
|-----------|---------|
| Create | `name` requis (1-255 chars), `description` (max 5000), `url` (max 2048), `estimated_price` (>= 0, validé côté service), `priority` (1-3, défaut 2, validé côté service), `category_id` (optionnel, ownership vérifié via query SQL) |
| Read | Par ID (exclut `deleted`), ou liste paginée |
| Update | Mise à jour partielle, détection de conflit status (409 si déjà dans le status cible) |
| Delete | **Soft delete** : status → `deleted` |

#### Validation de la catégorie (ownership)

À la création/mise à jour d'un item avec `category_id` :
- Requête : `SELECT EXISTS(SELECT 1 FROM categories WHERE id = $1 AND user_id = $2)`
- Si la catégorie n'existe pas ou n'appartient pas à l'utilisateur → `400 BAD_REQUEST`

#### Détection de conflit de status

Quand un changement de status est demandé :
1. `UPDATE ... WHERE status != $new_status` (atomic guard)
2. Si 0 rows affected :
   - Query pour vérifier si l'item existe encore
   - Si existe → `409 CONFLICT` ("item already has that status")
   - Si n'existe pas → `404 NOT_FOUND`

#### Statuts

```
active → purchased (auto-set purchased_at via trigger)
active → deleted (soft delete)
purchased → active (auto-clear purchased_at via trigger)
```

#### Pagination

| Paramètre | Défaut | Min | Max |
|-----------|--------|-----|-----|
| `page` | 1 | 1 | 1000 |
| `per_page` | 50 | 1 | 100 |

#### Tri

| Champ | Ordre possible |
|-------|---------------|
| `created_at` (défaut) | `asc`, `desc` (défaut) |
| `priority` | `asc`, `desc` |
| `name` | `asc`, `desc` |

> Whitelist validation des champs sort/order en double : côté service ET côté repository (defense-in-depth).

#### Filtres

- `status` : `active` (défaut) ou `purchased` — les items `deleted` sont toujours exclus
- `category_id` : UUID d'une catégorie

#### Réponse liste

```json
{
  "items": [...],
  "total": 42,
  "page": 1,
  "per_page": 50
}
```

### 3.3 Catégories

- **6 catégories par défaut** copiées à l'inscription (Tech, Mode, Maison, Loisirs, Santé, Autre)
- **Nom unique** par utilisateur (contrainte PG 23505 → 409 Conflict)
- **Icon** : optionnel, max 50 chars (nom d'icône SF Symbols / Ionicons)
- **Position** : int, auto-calculée à la création (`MAX(position) + 1`), permet le réordonnancement
- **Suppression** : interdit pour les catégories par défaut (`is_default = true`) → `400 BAD_REQUEST`
- **Cascade** : quand une catégorie est supprimée, les items associés ont `category_id` mis à `NULL` (FK SET NULL)
- **Invalidation cache** : la suppression d'une catégorie invalide **à la fois** le cache catégories ET le cache items (car les items voient leur `category_id` mis à NULL)

### 3.4 Rappels (Reminder Service)

#### Déclenchement

- CRON expression : `0 0 * * * *` (toutes les heures, à la seconde 0, minute 0)
- Cible : utilisateurs dont `utc_reminder_hour` = heure UTC actuelle ET `reminder_freq != 'never'` ET ayant au moins un push token

#### Lock distribué (multi-instance)

Le job utilise un verrou distribué Redis pour empêcher l'exécution concurrente :

| Paramètre | Valeur |
|-----------|--------|
| Clé Redis | `reminder_job:lock` |
| TTL | 300 secondes (5 minutes) |
| Valeur | UUID aléatoire (ownership) |
| Acquisition | `SET NX EX 300` |
| Libération | Script Lua CAS : vérifie ownership avant `DEL` |
| Fail-open | Si Redis indisponible, le tick est sauté (retry à l'heure suivante) |

#### Scoring des items

```
score = ((4 - priority) × PRIORITY_WEIGHT) + (days_waiting × AGE_WEIGHT)
```

> **Note** : `4 - priority` signifie que priority 1 (haute) score le plus haut (3×10=30), priority 3 (basse) score le plus bas (1×10=10).

| Constante | Valeur |
|-----------|--------|
| `PRIORITY_WEIGHT` | 10.0 |
| `AGE_WEIGHT` | 1.0 |
| `MIN_AGE_DAYS` | 7 (items dont `created_at` <= now - 7 jours) |
| `TOP_N` | 3 (top 3 items envoyés) |

#### Anti-spam

Redis `SET NX EX` avec clé `last_reminder:{user_id}` :

| Fréquence | TTL Redis | Durée |
|-----------|-----------|-------|
| `daily` | 82 800s | 23 heures |
| `weekly` | 561 600s | 6 jours 12 heures |
| `monthly` | 2 505 600s | 29 jours |

> Si aucun item éligible n'est trouvé, la clé anti-spam est supprimée (l'utilisateur n'est pas pénalisé).
> Si le fetch des items échoue, la clé anti-spam est aussi supprimée (retry au prochain tick).

#### Notification Expo Push

- Titre : `"Tes envies t'attendent !"`
- Body : noms des top 3 items séparés par `, `
- Sound : `default`
- Batch : envoi par chunks de 100 tokens vers `https://exp.host/--/api/v2/push/send`
- Gestion erreur `DeviceNotRegistered` : suppression automatique du token invalide en BDD
- Erreurs réseau : loguées, traitement continue pour les autres tokens

### 3.5 Cache Redis

#### Stratégie de versioning

Chaque entité (items, categories) utilise un compteur de version par utilisateur :

```
{resource}:{user_id}:ver          → compteur INCR
{resource}:{user_id}:v{ver}:{hash} → données cachées
```

| Paramètre | Valeur |
|-----------|--------|
| TTL | 300 secondes (5 minutes) |
| Stratégie | Fail-open (si Redis down, requête directe BDD) |
| Invalidation | INCR du compteur de version à chaque mutation |

#### Cache key items

Le `query_hash` est calculé à partir de `{status, category_id, sort, order, page, per_page}` via `DefaultHasher` de Rust (déterministe, non cryptographique).

#### Clés Redis utilisées

| Pattern | Usage | TTL |
|---------|-------|-----|
| `items:{user_id}:ver` | Version counter items | permanent |
| `items:{user_id}:v{ver}:{query_hash}` | Cache liste items | 300s |
| `categories:{user_id}:ver` | Version counter catégories | permanent |
| `categories:{user_id}:v{ver}` | Cache liste catégories | 300s |
| `pwreset:{email}` | Hash SHA-256 du code reset password | 1800s (30min) |
| `pwreset:rate:{email}` | Rate limit forgot-password | 60s |
| `last_reminder:{user_id}` | Anti-spam rappels | variable (voir 3.4) |
| `blacklist:{jti}` | Blacklist JTI access token | temps restant token |
| `tkver:{user_id}` | Version token pour invalidation globale | 900s (ACCESS_TOKEN_TTL) |
| `reminder_job:lock` | Lock distribué reminder job | 300s |

### 3.6 Push Tokens

- **Plateforme** : `ios` ou `android` uniquement (validation custom dans le DTO)
- **Upsert** : `ON CONFLICT (user_id, token) DO UPDATE` — met à jour la plateforme si le token existe déjà
- **Nettoyage auto** : les tokens invalides (`DeviceNotRegistered`) sont supprimés par le reminder service

### 3.7 Profil utilisateur

#### Champs modifiables

| Champ | Contrainte |
|-------|-----------|
| `display_name` | max 100 chars |
| `reminder_freq` | `never`, `daily`, `weekly`, `monthly` |
| `reminder_time` | `NaiveTime` (heure locale, format `HH:MM:SS`) |
| `timezone` | Validé via `chrono_tz` (IANA timezone database). Erreur : `400 "invalid timezone: {tz}"` |
| `locale` | max 10 chars |

> Quand `reminder_time` ou `timezone` change, `utc_reminder_hour` est recalculé automatiquement.
> En cas d'ambiguïté DST, la conversion utilise `.earliest()` (plus tôt possible).

#### Valeurs par défaut (à l'inscription)

| Champ | Défaut |
|-------|--------|
| `reminder_freq` | `weekly` |
| `reminder_time` | `10:00:00` |
| `timezone` | `UTC` |
| `locale` | `fr` |

#### Export GDPR

`GET /users/me/export` retourne toutes les données de l'utilisateur :

```json
{
  "profile": { ... },
  "items": [ ... ],
  "categories": [ ... ],
  "exported_at": "2026-03-08T12:00:00Z"
}
```

#### Suppression de compte

`DELETE /users/me` → suppression complète (CASCADE sur items, categories, push_tokens, refresh_tokens).

### 3.8 Gestion d'erreurs

Format uniforme pour toutes les erreurs :

```json
{
  "error": {
    "code": "BAD_REQUEST",
    "message": "description lisible"
  }
}
```

| Code HTTP | Code erreur | Usage |
|-----------|------------|-------|
| 400 | `BAD_REQUEST` | Validation échouée, données invalides, timezone invalide, code reset invalide |
| 401 | `UNAUTHORIZED` | Token absent, expiré, révoqué, version invalidée, credentials incorrects |
| 404 | `NOT_FOUND` | Ressource inexistante |
| 409 | `CONFLICT` | Email déjà pris, nom catégorie dupliqué, status déjà atteint |
| 500 | `INTERNAL_ERROR` | Erreur serveur (message générique `"an internal error occurred"`, pas de leak) |
| 503 | `SERVICE_UNAVAILABLE` | BDD ou Redis indisponible (health check) |

> Les erreurs 500 ne leakent jamais le message interne — le message réel est uniquement dans les logs serveur.

### 3.9 Intégrations externes

| Service | Usage | Endpoint | Fail mode |
|---------|-------|----------|-----------|
| **Expo Push API** | Notifications push iOS/Android | `https://exp.host/--/api/v2/push/send` | Log error, continue |
| **Resend** | Envoi email reset password | API Resend | Erreur loguée (envoi asynchrone via `tokio::spawn`) |
| **HIBP** | Vérification MDP breachés (k-Anonymity) | `https://api.pwnedpasswords.com/range/{prefix}` | Fail-open (accepte le MDP si API down, timeout 5s) |

---

## 4. État des Cercles

### Ce qui existe en BDD

Les tables `circles` et `circle_members` sont créées par les migrations 5 et 9 :

```sql
-- Migration 5
CREATE TABLE circles (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name       VARCHAR(100) NOT NULL,
    owner_id   UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE circle_members (
    circle_id  UUID NOT NULL REFERENCES circles(id) ON DELETE CASCADE,
    user_id    UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role       VARCHAR(20) NOT NULL DEFAULT 'member'
               CHECK (role IN ('owner', 'member')),
    joined_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (circle_id, user_id)
);

-- Migration 9
-- Trigger auto-ajout owner comme member
CREATE FUNCTION add_circle_owner_as_member() ...
CREATE TRIGGER trg_circles_add_owner_member
    AFTER INSERT ON circles
    FOR EACH ROW EXECUTE FUNCTION add_circle_owner_as_member();
```

### Ce qui manque

| Couche | État | Détails |
|--------|------|---------|
| **Migrations SQL** | Fait | Tables + trigger auto-owner |
| **Models Rust** | Manquant | Pas de `Circle` ni `CircleMember` dans `src/models/` |
| **Repositories** | Manquant | Pas de `circle_repo.rs` |
| **Services** | Manquant | Pas de `circle_service.rs` |
| **DTOs** | Manquant | Pas de `circle.rs` dans `src/dto/` |
| **Handlers** | Manquant | Pas de `circle.rs` dans `src/handlers/` |
| **Routes** | Manquant | Pas de `/circles` dans le router |
| **Partage items → cercles** | Manquant | Pas de lien entre `items` et `circles` (pas de colonne `circle_id` dans items, pas de table de liaison) |
| **Invitations** | Manquant | Pas de mécanisme d'invitation (email, lien, QR code) |
| **Permissions** | Manquant | Pas de logique de permissions (qui peut voir/modifier quoi dans un cercle) |

### Fonctionnalités Cercles à implémenter

Pour une V1 des Cercles, il faudra au minimum :

1. **API CRUD Cercles** : create, list, get, update, delete
2. **Gestion des membres** : inviter, accepter, quitter, exclure
3. **Partage d'items** : lier des items à un cercle (visible par tous les membres)
4. **Permissions** : owner (admin complet) vs member (lecture + partage de ses propres items)
5. **Notifications** : notifier les membres quand un item est partagé/acheté dans leur cercle

### Questions ouvertes pour le brainstorm

- Un item partagé dans un cercle est-il une **copie** ou une **référence** à l'item original ?
- Un item peut-il être partagé dans **plusieurs cercles** ?
- Les membres d'un cercle peuvent-ils **marquer un item comme acheté** (gift tracking) ?
- Comment gérer les **invitations** ? (email, lien unique, QR code, recherche par nom)
- Faut-il un **feed/timeline** pour les activités du cercle ?
- Les catégories sont-elles **partagées** au niveau du cercle ou restent-elles personnelles ?

---

## 5. Constantes de référence

### Auth & Sécurité

| Constante | Valeur | Source |
|-----------|--------|--------|
| `ACCESS_TOKEN_TTL_SECS` | 900 (15 min) | `utils/jwt.rs` |
| `REFRESH_TOKEN_TTL_SECS` | 604 800 (7 jours) | `utils/jwt.rs` |
| `MAX_REFRESH_TOKENS_PER_USER` | 5 | `services/auth_service.rs` |
| `ARGON2_MEMORY_COST_KIB` | 19 456 | `utils/hash.rs` |
| `ARGON2_TIME_COST` | 2 | `utils/hash.rs` |
| `ARGON2_PARALLELISM` | 1 | `utils/hash.rs` |
| Forgot password code TTL | 1 800s (30 min) | `services/auth_service.rs` |
| Forgot password rate limit | 60s | `services/auth_service.rs` |

### Items & Catégories

| Constante | Valeur | Source |
|-----------|--------|--------|
| `LIST_CACHE_TTL_SECS` | 300 (5 min) | `services/item_service.rs`, `category_service.rs` |
| `DEFAULT_PER_PAGE` | 50 | `services/item_service.rs` |
| `MAX_PER_PAGE` | 100 | `services/item_service.rs` |
| `MAX_PAGE` | 1 000 | `services/item_service.rs` |

### Rappels

| Constante | Valeur | Source |
|-----------|--------|--------|
| `PRIORITY_WEIGHT` | 10.0 | `services/reminder_service.rs` |
| `AGE_WEIGHT` | 1.0 | `services/reminder_service.rs` |
| `MIN_AGE_DAYS` | 7 | `services/reminder_service.rs` |
| `TOP_N` | 3 | `services/reminder_service.rs` |
| Daily anti-spam TTL | 82 800s (23h) | `services/reminder_service.rs` |
| Weekly anti-spam TTL | 561 600s (6j 12h) | `services/reminder_service.rs` |
| Monthly anti-spam TTL | 2 505 600s (29j) | `services/reminder_service.rs` |
| Lock TTL | 300s (5 min) | `jobs/reminder_job.rs` |

### Health

| Constante | Valeur | Source |
|-----------|--------|--------|
| `HEALTH_CHECK_TIMEOUT` | 5s | `handlers/health.rs` |
