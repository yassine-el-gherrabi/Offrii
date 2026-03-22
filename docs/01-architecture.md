# Architecture Technique - Offrii Backend

> Document technique pour le dossier CDA.
> Derniere mise a jour : mars 2026.

---

## 1. Vue d'ensemble

L'infrastructure Offrii suit un modele client-serveur classique avec un backend API REST monolithique modulaire.

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
   │ (donnees)  │  │ (cache)  │  │ (media) │  │ (push)  │
   └────────────┘  └─────────┘  └─────────┘  └─────────┘
                                      │
                               ┌──────┴──────┐
                               │   Resend    │  ← Emails transactionnels
                               └─────────────┘
                               ┌─────────────┐
                               │   OpenAI    │  ← Moderation de contenu
                               └─────────────┘
```

**Flux d'une requete typique** : le client iOS envoie une requete HTTPS. Caddy termine le TLS, applique le rate limiting, puis proxifie vers l'API Axum en HTTP interne. L'API traverse la pile de middlewares, authentifie le JWT, puis route vers le handler concerne. Le handler delegue au service, qui orchestre la logique metier et appelle un ou plusieurs repositories pour l'acces aux donnees.

---

## 2. Architecture backend : Handler → Service → Repository

Le backend est structure en trois couches avec une separation stricte des responsabilites.

### Responsabilites par couche

| Couche | Role | Depend de |
|---|---|---|
| **Handler** | Deserialisation HTTP, validation des entrees, appel au service, serialisation de la reponse | Service (via trait) |
| **Service** | Logique metier, orchestration multi-repo, regles de gestion, cache | Repository (via trait) |
| **Repository** | Acces aux donnees (SQL), mapping modele, aucune logique metier | Base de donnees (PgPool) |

### Pourquoi ce pattern

**Testabilite** : chaque couche peut etre testee isolement. Un service peut etre teste avec un mock de repository sans base de donnees. Un handler peut etre teste avec un mock de service.

**Principe SOLID respecte** :
- **S** (Single Responsibility) : le handler ne fait que du HTTP, le service ne fait que de la logique, le repo ne fait que du SQL.
- **D** (Dependency Inversion) : les couches hautes dependent d'abstractions (traits), pas d'implementations concretes.
- **O** (Open/Closed) : on peut ajouter une implementation `RedisItemRepo` sans modifier `PgItemService`.

### Exemple concret : le flux d'un item

```
POST /v1/items
    │
    ▼
items::handler::create_item(AuthUser, State<AppState>, Json<CreateItemRequest>)
    │  ← extrait le user_id du JWT, valide le body
    ▼
state.items.create_item(user_id, name, ...)        // appel via Arc<dyn ItemService>
    │  ← verifie les regles metier (limites, permissions)
    ▼
item_repo.create(user_id, name, ...)               // appel via Arc<dyn ItemRepo>
    │  ← INSERT INTO items ... RETURNING *
    ▼
ItemResponse (DTO) ← sérialise en JSON → 201 Created
```

---

## 3. Pile de middlewares

Les middlewares Axum sont appliques en ordre LIFO (le dernier `.layer()` s'execute en premier). Voici l'ordre d'execution reel sur chaque requete entrante :

```
Requete entrante
    │
    ▼
1. Prometheus Metrics     ← compteurs request_count, request_duration par methode/path/status
    │
    ▼
2. Security Headers       ← X-Content-Type-Options: nosniff
    │                       X-Frame-Options: DENY
    │                       Strict-Transport-Security: max-age=31536000
    ▼
3. CORS                   ← origines autorisees : offrii.com, api.offrii.com, cdn.offrii.com
    │                       methodes : GET, POST, PUT, PATCH, DELETE, OPTIONS
    │                       headers : Authorization, Content-Type, Accept
    ▼
4. Trace                  ← logs structures (JSON) via tracing, correlation par requete
    │
    ▼
5. Timeout                ← 30 secondes max par requete, retourne 408 Request Timeout
    │
    ▼
6. Body Limit             ← 10 Mo max (upload d'images)
    │
    ▼
7. Auth (extracteur)      ← pas un layer Tower, mais un extracteur Axum (FromRequestParts)
    │                       valide le JWT RS256, verifie la blacklist Redis,
    │                       verifie la version du token
    ▼
Handler
```

L'endpoint `/metrics` est monte **apres** le CORS pour ne pas etre soumis aux restrictions d'origine. Les endpoints de health check (`/health`, `/health/live`, `/health/ready`) sont exclus du tracking Prometheus.

---

## 4. Injection de dependances

### Le contrat : les traits

Le module `traits/` definit les contrats de chaque couche. Chaque trait est un fichier separe par domaine :

```
traits/
├── mod.rs            ← re-exporte tout
├── auth.rs           ← AuthService, RefreshTokenRepo, EmailService
├── items.rs          ← ItemRepo, ItemService, CircleItemRepo, CategoryRepo
├── circles.rs        ← CircleRepo, CircleMemberRepo, CircleInviteRepo, ...
├── community.rs      ← CommunityWishRepo, ModerationService, WishMessageService
├── notifications.rs  ← NotificationRepo, PushTokenRepo, NotificationService
├── social.rs         ← FriendRepo, FriendService
├── users.rs          ← UserRepo, UserService
└── infra.rs          ← ShareLinkRepo, UploadService, HealthCheck
```

Chaque trait impose `Send + Sync` pour la compatibilite avec le runtime async Tokio multi-thread :

```rust
#[async_trait]
pub trait ItemService: Send + Sync {
    async fn create_item(&self, user_id: Uuid, name: &str, ...) -> Result<ItemResponse, AppError>;
    async fn get_item(&self, id: Uuid, user_id: Uuid) -> Result<ItemResponse, AppError>;
    // ...
}
```

### Le cablage : `Arc<dyn Trait>`

Dans `main.rs`, chaque implementation concrete est wrappee dans un `Arc<dyn Trait>` pour le polymorphisme a l'execution :

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

Avantage : n'importe quel handler recoit un `State<AppState>` et travaille uniquement avec des abstractions. On peut substituer n'importe quelle implementation sans toucher aux handlers.

### Implementations Noop pour les services optionnels

Certains services externes ne sont pas toujours disponibles (dev local, CI). Le pattern Noop permet un degradage propre :

```rust
// Moderation : active si MODERATION_ENABLED=true et OPENAI_API_KEY present
let moderation_svc: Arc<dyn ModerationService> = if config.moderation_enabled {
    Arc::new(OpenAIModerationService::new(api_key))
} else {
    Arc::new(NoopModerationService)  // ← accepte tout, ne fait rien
};

// Upload : R2 si les credentials sont configurees, sinon URLs de test
let upload_svc: Arc<dyn UploadService> = if r2_configured {
    Arc::new(R2UploadService::new(...).await)
} else {
    Arc::new(NoopUploadService)      // ← retourne des URLs fictives
};
```

Ce pattern evite les `Option<Arc<dyn Trait>>` et les verifications `if let Some(...)` dans toute la codebase. Le service Noop implemente le meme trait, les appelants n'ont pas besoin de savoir si le service est reel ou fictif.

---

## 5. Gestion d'erreurs

### L'enum `AppError`

Toutes les erreurs metier sont representees par un enum unique derive de `thiserror` :

```rust
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("internal server error")]
    Internal(#[from] anyhow::Error),    // 500 — message generique (pas de fuite d'info)

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

L'implementation de `IntoResponse` convertit chaque variante en un code HTTP + un body JSON normalise :

```json
{
  "error": {
    "code": "NOT_FOUND",
    "message": "item not found"
  }
}
```

Point de securite : la variante `Internal` ne renvoie **jamais** le message d'erreur reel au client. Elle logge l'erreur complete cote serveur (`tracing::error!`) et retourne un message generique `"an internal error occurred"`. Cela evite toute fuite d'information sensible (noms de tables, requetes SQL, etc.).

### Propagation avec `?`

Grace a `#[from] anyhow::Error`, toute erreur non geree (sqlx, redis, I/O) se convertit automatiquement en `AppError::Internal` via l'operateur `?`. Les erreurs metier explicites utilisent les variantes nommees :

```rust
// Erreur metier explicite
if user.is_none() {
    return Err(AppError::NotFound("user not found".into()));
}

// Erreur infra propagee automatiquement → 500
let row = sqlx::query("SELECT ...").fetch_one(&self.db).await?;
```

---

## 6. Decisions techniques (ADR)

| Decision | Alternative ecartee | Justification |
|---|---|---|
| **Rust** | Node.js, Go | Performances previsibles sans GC. Type system strict qui elimine des categories entieres de bugs a la compilation (null safety, thread safety). Memory safety sans garbage collector. |
| **PostgreSQL** | MongoDB | Le modele de donnees est hautement relationnel (users ↔ circles ↔ items ↔ share rules). Les regles de partage et les permissions exigent des transactions ACID. Les requetes complexes avec jointures multiples sont naturelles en SQL. |
| **Redis** | Memcached, in-memory | Triple usage : cache de listes paginee, blacklist de tokens JWT revoques, throttle `last_active_at` (SET NX EX). Support natif du TTL pour l'expiration automatique des cles. |
| **Axum** | Actix-web | Integre nativement dans l'ecosysteme Tower (middlewares composables). Extracteurs type-safe. Async-native sans macros magiques. Communaute Tokio active. |
| **SQLx** | Diesel, SeaORM | Verification des requetes SQL **a la compilation** contre le schema reel. Pas de DSL a apprendre : on ecrit du SQL natif. Support async natif sans pool bloquant. |
| **JWT RS256** | HS256, sessions | Signature asymetrique : la cle privee signe (API), la cle publique verifie (n'importe quel service). Permet une architecture future multi-service sans partage de secret. Blacklist via Redis pour la revocation. |
| **Caddy** | Nginx, Traefik | TLS automatique via ACME (zero config certificat). Configuration declarative. Rate limiting integre. Reverse proxy avec health checks. |
| **Cloudflare R2** | AWS S3, stockage local | Compatible S3 (SDK standard). Pas de frais d'egress. CDN Cloudflare integre. |
| **APNs (direct)** | Firebase FCM | Controle total du payload de notification. Pas de dependance intermediaire. Support natif du bundling et de la localisation iOS. |
| **Resend** | SendGrid, SES | API simple, bonne delivrabilite. SDK Rust via HTTP. Templates HTML geres cote backend. |
| **OpenAI Moderation** | Moderation manuelle | Filtrage automatique du contenu communautaire (wishes). Implementation Noop pour desactiver en dev/CI. Pas de moderation humaine necessaire pour le MVP. |

---

## Glossaire

| Terme | Definition |
|---|---|
| **Handler** | Fonction Axum qui recoit une requete HTTP et retourne une reponse |
| **Service** | Composant de logique metier, injecte via `Arc<dyn Trait>` |
| **Repository** | Composant d'acces aux donnees, une implementation par source (Pg, Redis, ...) |
| **Extracteur** | Type implementant `FromRequestParts` qui extrait des donnees de la requete (auth, pagination) |
| **Noop** | Implementation vide d'un trait, utilisee quand le service reel n'est pas disponible |
| **AppState** | Structure partagee entre tous les handlers, contient les services injectes |
| **Trait object** | `dyn Trait` — polymorphisme dynamique en Rust, equivalent d'une interface en Java/Go |
