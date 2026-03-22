# Architecture Technique - Offrii Backend

> Document technique pour le dossier CDA.
> Dernière mise à jour : mars 2026.

---

## 1. Vue d'ensemble

L'infrastructure Offrii suit un modèle client-serveur classique avec un backend API REST monolithique modulaire.

```
┌──────────────┐
│  Client iOS   │
│  (SwiftUI)    │
└──────┬───────┘
       │ HTTPS
       ▼
┌──────────────┐
│    Caddy      │  ← TLS automatique (Let's Encrypt)
│  (reverse     │  ← Rate limiting
│   proxy)      │  ← Compression
└──────┬───────┘
       │ HTTP (interne)
       ▼
┌──────────────────────────────────────────────────┐
│              Rust / Axum API                      │
│  ┌──────────┐  ┌──────────┐  ┌───────────────┐  │
│  │ Handlers │→ │ Services │→ │ Repositories  │  │
│  └──────────┘  └──────────┘  └───────┬───────┘  │
│                                      │           │
│  Middleware : Prometheus │ CORS │ Security │ JWT  │
└──────────────────────────┬───────────┼───────────┘
                           │           │
          ┌────────────────┼───────────┼────────────┐
          │                │           │            │
          ▼                ▼           ▼            ▼
   ┌────────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐
   │ PostgreSQL │  │  Redis   │  │   R2    │  │  APNs   │
   │ (données)  │  │ (cache)  │  │ (media) │  │ (push)  │
   └────────────┘  └─────────┘  └─────────┘  └─────────┘
                                      │
                               ┌──────┴──────┐
                               │   Resend    │  ← Emails transactionnels
                               └─────────────┘
                               ┌─────────────┐
                               │   OpenAI    │  ← Modération de contenu
                               └─────────────┘
```

**Flux d'une requête typique** : le client iOS envoie une requête HTTPS. Caddy termine le TLS, applique le rate limiting, puis proxifie vers l'API Axum en HTTP interne. L'API traverse la pile de middlewares, authentifie le JWT, puis route vers le handler concerné. Le handler délègue au service, qui orchestre la logique métier et appelle un ou plusieurs repositories pour l'accès aux données.

---

## 2. Architecture backend : Handler → Service → Repository

Le backend est structuré en trois couches avec une séparation stricte des responsabilités.

### Responsabilités par couche

| Couche | Rôle | Dépend de |
|---|---|---|
| **Handler** | Désérialisation HTTP, validation des entrées, appel au service, sérialisation de la réponse | Service (via trait) |
| **Service** | Logique métier, orchestration multi-repo, règles de gestion, cache | Repository (via trait) |
| **Repository** | Accès aux données (SQL), mapping modèle, aucune logique métier | Base de données (PgPool) |

### Pourquoi ce pattern

**Testabilité** : chaque couche peut être testée isolément. Un service peut être testé avec un mock de repository sans base de données. Un handler peut être testé avec un mock de service.

**Principe SOLID respecté** :
- **S** (Single Responsibility) : le handler ne fait que du HTTP, le service ne fait que de la logique, le repo ne fait que du SQL.
- **D** (Dependency Inversion) : les couches hautes dépendent d'abstractions (traits), pas d'implémentations concrètes.
- **O** (Open/Closed) : on peut ajouter une implémentation `RedisItemRepo` sans modifier `PgItemService`.

### Exemple concret : le flux d'un item

```
POST /v1/items
    │
    ▼
items::handler::create_item(AuthUser, State<AppState>, Json<CreateItemRequest>)
    │  ← extrait le user_id du JWT, valide le body
    ▼
state.items.create_item(user_id, name, ...)        // appel via Arc<dyn ItemService>
    │  ← vérifie les règles métier (limites, permissions)
    ▼
item_repo.create(user_id, name, ...)               // appel via Arc<dyn ItemRepo>
    │  ← INSERT INTO items ... RETURNING *
    ▼
ItemResponse (DTO) ← sérialise en JSON → 201 Created
```

---

## 3. Pile de middlewares

Les middlewares Axum sont appliqués en ordre LIFO (le dernier `.layer()` s'exécute en premier). Voici l'ordre d'exécution réel sur chaque requête entrante :

```
Requête entrante
    │
    ▼
1. Prometheus Metrics     ← compteurs request_count, request_duration par méthode/path/status
    │
    ▼
2. Security Headers       ← X-Content-Type-Options: nosniff
    │                       X-Frame-Options: DENY
    │                       Strict-Transport-Security: max-age=31536000
    ▼
3. CORS                   ← origines autorisées : offrii.com, api.offrii.com, cdn.offrii.com
    │                       méthodes : GET, POST, PUT, PATCH, DELETE, OPTIONS
    │                       headers : Authorization, Content-Type, Accept
    ▼
4. Trace                  ← logs structurés (JSON) via tracing, corrélation par requête
    │
    ▼
5. Timeout                ← 30 secondes max par requête, retourne 408 Request Timeout
    │
    ▼
6. Body Limit             ← 10 Mo max (upload d'images)
    │
    ▼
7. Auth (extracteur)      ← pas un layer Tower, mais un extracteur Axum (FromRequestParts)
    │                       valide le JWT RS256, vérifie la blacklist Redis,
    │                       vérifie la version du token
    ▼
Handler
```

L'endpoint `/metrics` est monté **après** le CORS pour ne pas être soumis aux restrictions d'origine. Les endpoints de health check (`/health`, `/health/live`, `/health/ready`) sont exclus du tracking Prometheus.

---

## 4. Injection de dépendances

### Le contrat : les traits

Le module `traits/` définit les contrats de chaque couche. Chaque trait est un fichier séparé par domaine :

```
traits/
├── mod.rs            ← ré-exporte tout
├── auth.rs           ← AuthService, RefreshTokenRepo, EmailService
├── items.rs          ← ItemRepo, ItemService, CircleItemRepo, CategoryRepo
├── circles.rs        ← CircleRepo, CircleMemberRepo, CircleInviteRepo, ...
├── community.rs      ← CommunityWishRepo, ModerationService, WishMessageService
├── notifications.rs  ← NotificationRepo, PushTokenRepo, NotificationService
├── social.rs         ← FriendRepo, FriendService
├── users.rs          ← UserRepo, UserService
└── infra.rs          ← ShareLinkRepo, UploadService, HealthCheck
```

Chaque trait impose `Send + Sync` pour la compatibilité avec le runtime async Tokio multi-thread :

```rust
#[async_trait]
pub trait ItemService: Send + Sync {
    async fn create_item(&self, user_id: Uuid, name: &str, ...) -> Result<ItemResponse, AppError>;
    async fn get_item(&self, id: Uuid, user_id: Uuid) -> Result<ItemResponse, AppError>;
    // ...
}
```

### Le câblage : `Arc<dyn Trait>`

Dans `main.rs`, chaque implémentation concrète est wrappée dans un `Arc<dyn Trait>` pour le polymorphisme à l'exécution :

```rust
let item_repo: Arc<dyn ItemRepo> = Arc::new(PgItemRepo::new(db.clone()));
let items: Arc<dyn ItemService> = Arc::new(PgItemService::new(
    db.clone(),
    item_repo.clone(),
    circle_item_repo.clone(),
    redis.clone(),
));
```

L'`AppState` ne contient que des `Arc<dyn Trait>` :

```rust
#[derive(Clone)]
pub struct AppState {
    pub auth: Arc<dyn AuthService>,
    pub items: Arc<dyn ItemService>,
    pub categories: Arc<dyn CategoryService>,
    pub users: Arc<dyn UserService>,
    pub uploads: Arc<dyn UploadService>,
    // ... chaque champ est un trait object
}
```

Avantage : n'importe quel handler reçoit un `State<AppState>` et travaille uniquement avec des abstractions. On peut substituer n'importe quelle implémentation sans toucher aux handlers.

### Implémentations Noop pour les services optionnels

Certains services externes ne sont pas toujours disponibles (dev local, CI). Le pattern Noop permet un dégradage propre :

```rust
// Modération : active si MODERATION_ENABLED=true et OPENAI_API_KEY présent
let moderation_svc: Arc<dyn ModerationService> = if config.moderation_enabled {
    Arc::new(OpenAIModerationService::new(api_key))
} else {
    Arc::new(NoopModerationService)  // ← accepte tout, ne fait rien
};

// Upload : R2 si les credentials sont configurées, sinon URLs de test
let upload_svc: Arc<dyn UploadService> = if r2_configured {
    Arc::new(R2UploadService::new(...).await)
} else {
    Arc::new(NoopUploadService)      // ← retourne des URLs fictives
};
```

Ce pattern évite les `Option<Arc<dyn Trait>>` et les vérifications `if let Some(...)` dans toute la codebase. Le service Noop implémente le même trait, les appelants n'ont pas besoin de savoir si le service est réel ou fictif.

---

## 5. Gestion d'erreurs

### L'enum `AppError`

Toutes les erreurs métier sont représentées par un enum unique dérivé de `thiserror` :

```rust
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("internal server error")]
    Internal(#[from] anyhow::Error),    // 500 — message générique (pas de fuite d'info)

    #[error("unauthorized: {0}")]
    Unauthorized(String),                // 401

    #[error("forbidden: {0}")]
    Forbidden(String),                   // 403

    #[error("not found: {0}")]
    NotFound(String),                    // 404

    #[error("conflict: {0}")]
    Conflict(String),                    // 409

    #[error("bad request: {0}")]
    BadRequest(String),                  // 400

    #[error("too many requests: {0}")]
    TooManyRequests(String),             // 429

    #[error("gone: {0}")]
    Gone(String),                        // 410

    #[error("service unavailable: {0}")]
    ServiceUnavailable(String),          // 503
}
```

### Mapping vers HTTP

L'implémentation de `IntoResponse` convertit chaque variante en un code HTTP + un body JSON normalisé :

```json
{
  "error": {
    "code": "NOT_FOUND",
    "message": "item not found"
  }
}
```

Point de sécurité : la variante `Internal` ne renvoie **jamais** le message d'erreur réel au client. Elle logge l'erreur complète côté serveur (`tracing::error!`) et retourne un message générique `"an internal error occurred"`. Cela évite toute fuite d'information sensible (noms de tables, requêtes SQL, etc.).

### Propagation avec `?`

Grâce à `#[from] anyhow::Error`, toute erreur non gérée (sqlx, redis, I/O) se convertit automatiquement en `AppError::Internal` via l'opérateur `?`. Les erreurs métier explicites utilisent les variantes nommées :

```rust
// Erreur métier explicite
if user.is_none() {
    return Err(AppError::NotFound("user not found".into()));
}

// Erreur infra propagée automatiquement → 500
let row = sqlx::query("SELECT ...").fetch_one(&self.db).await?;
```

---

## 6. Décisions techniques (ADR)

| Décision | Alternative écartée | Justification |
|---|---|---|
| **Rust** | Node.js, Go | Performances prévisibles sans GC. Type system strict qui élimine des catégories entières de bugs à la compilation (null safety, thread safety). Memory safety sans garbage collector. |
| **PostgreSQL** | MongoDB | Le modèle de données est hautement relationnel (users ↔ circles ↔ items ↔ share rules). Les règles de partage et les permissions exigent des transactions ACID. Les requêtes complexes avec jointures multiples sont naturelles en SQL. |
| **Redis** | Memcached, in-memory | Triple usage : cache de listes paginée, blacklist de tokens JWT révoqués, throttle `last_active_at` (SET NX EX). Support natif du TTL pour l'expiration automatique des clés. |
| **Axum** | Actix-web | Intégré nativement dans l'écosystème Tower (middlewares composables). Extracteurs type-safe. Async-native sans macros magiques. Communauté Tokio active. |
| **SQLx** | Diesel, SeaORM | Vérification des requêtes SQL **à la compilation** contre le schéma réel. Pas de DSL à apprendre : on écrit du SQL natif. Support async natif sans pool bloquant. |
| **JWT RS256** | HS256, sessions | Signature asymétrique : la clé privée signe (API), la clé publique vérifie (n'importe quel service). Permet une architecture future multi-service sans partage de secret. Blacklist via Redis pour la révocation. |
| **Caddy** | Nginx, Traefik | TLS automatique via ACME (zéro config certificat). Configuration déclarative. Rate limiting intégré. Reverse proxy avec health checks. |
| **Cloudflare R2** | AWS S3, stockage local | Compatible S3 (SDK standard). Pas de frais d'egress. CDN Cloudflare intégré. |
| **APNs (direct)** | Firebase FCM | Contrôle total du payload de notification. Pas de dépendance intermédiaire. Support natif du bundling et de la localisation iOS. |
| **Resend** | SendGrid, SES | API simple, bonne délivrabilité. SDK Rust via HTTP. Templates HTML gérés côté backend. |
| **OpenAI Moderation** | Modération manuelle | Filtrage automatique du contenu communautaire (wishes). Implémentation Noop pour désactiver en dev/CI. Pas de modération humaine nécessaire pour le MVP. |

---

## Glossaire

| Terme | Définition |
|---|---|
| **Handler** | Fonction Axum qui reçoit une requête HTTP et retourne une réponse |
| **Service** | Composant de logique métier, injecté via `Arc<dyn Trait>` |
| **Repository** | Composant d'accès aux données, une implémentation par source (Pg, Redis, ...) |
| **Extracteur** | Type implémentant `FromRequestParts` qui extrait des données de la requête (auth, pagination) |
| **Noop** | Implémentation vide d'un trait, utilisée quand le service réel n'est pas disponible |
| **AppState** | Structure partagée entre tous les handlers, contient les services injectés |
| **Trait object** | `dyn Trait` — polymorphisme dynamique en Rust, équivalent d'une interface en Java/Go |
