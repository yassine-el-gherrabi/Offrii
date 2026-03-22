# Référence API

## Base URL

```
https://api.offrii.com/v1/
```

Toutes les routes documentées ci-dessous sont préfixées par `/v1` sauf indication contraire (endpoints publics hors-versioning).

---

## Authentification

L'API utilise des **JSON Web Tokens (JWT)** signés avec l'algorithme asymétrique **RS256** (clé privée sur le serveur, clé publique pour vérification).

| Paramètre | Valeur |
|---|---|
| Algorithme | RS256 (asymétrique) |
| Durée access token | 15 minutes |
| Durée refresh token | 7 jours |
| Rotation | Atomique -- chaque appel à `/auth/refresh` invalide l'ancien refresh token et en émet un nouveau |
| Révocation | Blacklist via Redis (JTI) au logout |

### Format de l'en-tête

```http
Authorization: Bearer <access_token>
```

Les endpoints marqués **Auth = Oui** dans les tables ci-dessous exigent cet en-tête. Sans token valide, l'API répond `401 Unauthorized`.

---

## Format de réponse

Toutes les réponses sont en **JSON** (`Content-Type: application/json`).

### Réponse paginée

Les endpoints de liste retournent une enveloppe paginée :

```json
{
  "data": [ ... ],
  "pagination": {
    "total": 42,
    "page": 1,
    "limit": 20,
    "total_pages": 3,
    "has_more": true
  }
}
```

| Paramètre query | Défaut | Min | Max | Description |
|---|---|---|---|---|
| `page` | 1 | 1 | -- | Numéro de page |
| `limit` | 20 | 1 | 100 | Nombre d'éléments par page |

### Réponse d'erreur

```json
{
  "error": {
    "code": "NOT_FOUND",
    "message": "item not found"
  }
}
```

---

## Codes d'erreur

| Variante AppError | Code HTTP | `error.code` | Cas d'usage typique |
|---|---|---|---|
| `BadRequest` | 400 | `BAD_REQUEST` | Validation échouée, champs manquants |
| `Unauthorized` | 401 | `UNAUTHORIZED` | Token manquant, expiré ou invalide |
| `Forbidden` | 403 | `FORBIDDEN` | Droits insuffisants (ex: pas propriétaire) |
| `NotFound` | 404 | `NOT_FOUND` | Ressource inexistante |
| `Conflict` | 409 | `CONFLICT` | Doublon (email déjà pris, invitation existante) |
| `Gone` | 410 | `GONE` | Ressource supprimée définitivement |
| `TooManyRequests` | 429 | `TOO_MANY_REQUESTS` | Rate limiting dépassé |
| `Internal` | 500 | `INTERNAL_ERROR` | Erreur serveur (message générique, pas de fuite) |
| `ServiceUnavailable` | 503 | `SERVICE_UNAVAILABLE` | Dépendance indisponible (DB, Redis) |

> **Sécurité** : les erreurs `500` ne renvoient jamais le message interne -- toujours `"an internal error occurred"`.

---

## Endpoints par domaine

### Auth

Préfixe : `/v1/auth`

| Méthode | Chemin | Auth | Description |
|---|---|---|---|
| POST | `/register` | Non | Inscription (email + mot de passe) |
| POST | `/login` | Non | Connexion (email ou username + mot de passe) |
| POST | `/refresh` | Non | Renouvellement du couple access/refresh token |
| POST | `/logout` | Oui | Déconnexion (blacklist du JTI) |
| POST | `/change-password` | Oui | Changement de mot de passe |
| POST | `/forgot-password` | Non | Envoi d'un code de réinitialisation par email |
| POST | `/verify-reset-code` | Non | Vérification du code de réinitialisation |
| POST | `/reset-password` | Non | Réinitialisation du mot de passe avec le code |
| POST | `/verify-email` | Non | Vérification de l'adresse email (via token) |
| POST | `/resend-verification` | Oui | Renvoi de l'email de vérification |
| POST | `/google` | Non | Authentification OAuth Google (id_token) |
| POST | `/apple` | Non | Authentification OAuth Apple (id_token) |

### Users

Préfixe : `/v1/me`

| Méthode | Chemin | Auth | Description |
|---|---|---|---|
| GET | `/profile` | Oui | Récupérer le profil de l'utilisateur connecté |
| PATCH | `/profile` | Oui | Modifier le profil (display_name, username, avatar) |
| DELETE | `/profile` | Oui | Supprimer le compte (RGPD) |
| GET | `/export` | Oui | Export complet des données (RGPD) |
| POST | `/email` | Oui | Demande de changement d'adresse email |

### Items (Envies)

Préfixe : `/v1/items`

| Méthode | Chemin | Auth | Description |
|---|---|---|---|
| GET | `/` | Oui | Lister mes envies (paginé, filtrés par statut) |
| POST | `/` | Oui | Créer une envie |
| GET | `/{id}` | Oui | Détail d'une envie |
| PATCH | `/{id}` | Oui | Modifier une envie |
| DELETE | `/{id}` | Oui | Supprimer une envie |
| POST | `/{id}/claim` | Oui | Réserver une envie (en tant que proche) |
| DELETE | `/{id}/claim` | Oui | Annuler sa réservation |
| DELETE | `/{id}/web-claim` | Oui | Annuler une réservation (par le propriétaire, web) |
| POST | `/batch-delete` | Oui | Suppression groupée d'envies |

### Catégories

Préfixe : `/v1/categories`

| Méthode | Chemin | Auth | Description |
|---|---|---|---|
| GET | `/` | Oui | Lister les catégories d'envies disponibles |

### Circles (Proches)

Préfixe : `/v1/circles`

| Méthode | Chemin | Auth | Description |
|---|---|---|---|
| GET | `/` | Oui | Lister mes cercles |
| POST | `/` | Oui | Créer un cercle |
| GET | `/{id}` | Oui | Détail d'un cercle |
| PATCH | `/{id}` | Oui | Modifier un cercle (nom, emoji) |
| DELETE | `/{id}` | Oui | Supprimer un cercle |
| POST | `/direct/{user_id}` | Oui | Créer un cercle direct (1-à-1 avec un ami) |
| POST | `/{id}/invite` | Oui | Générer un lien d'invitation |
| POST | `/join/{token}` | Oui | Rejoindre un cercle via invitation |
| POST | `/{id}/members` | Oui | Ajouter un membre (ami) |
| DELETE | `/{id}/members/{uid}` | Oui | Retirer un membre |
| GET | `/{id}/invites` | Oui | Lister les invitations actives |
| DELETE | `/{id}/invites/{iid}` | Oui | Révoquer une invitation |
| POST | `/{id}/items` | Oui | Partager une envie dans un cercle |
| GET | `/{id}/items` | Oui | Lister les envies partagées dans un cercle |
| POST | `/{id}/items/batch` | Oui | Partager plusieurs envies d'un coup |
| GET | `/{id}/items/{item_id}` | Oui | Détail d'une envie partagée |
| DELETE | `/{id}/items/{item_id}` | Oui | Retirer une envie d'un cercle |
| GET | `/{id}/share-rule` | Oui | Voir la règle de partage automatique |
| PUT | `/{id}/share-rule` | Oui | Définir la règle de partage automatique |
| GET | `/{id}/feed` | Oui | Fil d'actualité du cercle |
| POST | `/{id}/transfer` | Oui | Transférer la propriété du cercle |
| GET | `/my-reservations` | Oui | Lister mes réservations dans tous les cercles |
| GET | `/my-share-rules` | Oui | Lister mes règles de partage |

### Friends (Amis)

Préfixe : `/v1/me` (demandes) et `/v1/users` (recherche)

| Méthode | Chemin | Auth | Description |
|---|---|---|---|
| GET | `/users/search` | Oui | Rechercher un utilisateur (par nom ou username) |
| POST | `/me/friend-requests` | Oui | Envoyer une demande d'ami |
| GET | `/me/friend-requests` | Oui | Lister les demandes reçues en attente |
| GET | `/me/friend-requests/sent` | Oui | Lister les demandes envoyées |
| POST | `/me/friend-requests/{id}/accept` | Oui | Accepter une demande |
| DELETE | `/me/friend-requests/{id}/cancel` | Oui | Annuler une demande envoyée |
| DELETE | `/me/friend-requests/{id}` | Oui | Refuser une demande reçue |
| GET | `/me/friends` | Oui | Lister mes amis |
| DELETE | `/me/friends/{user_id}` | Oui | Supprimer un ami |

### Community Wishes -- Entraide (Besoins)

Préfixe : `/v1/community/wishes`

| Méthode | Chemin | Auth | Description |
|---|---|---|---|
| GET | `/` | Oui | Lister les besoins communautaires (paginé) |
| POST | `/` | Oui | Créer un besoin |
| GET | `/mine` | Oui | Lister mes besoins |
| GET | `/my-offers` | Oui | Lister les besoins où j'ai proposé mon aide |
| GET | `/recent-fulfilled` | Oui | Besoins récemment comblés |
| GET | `/{id}` | Oui | Détail d'un besoin |
| PATCH | `/{id}` | Oui | Modifier un besoin |
| DELETE | `/{id}` | Oui | Supprimer un besoin |
| POST | `/{id}/close` | Oui | Clôturer un besoin |
| POST | `/{id}/reopen` | Oui | Rouvrir un besoin clôturé |
| POST | `/{id}/offer` | Oui | Proposer son aide |
| DELETE | `/{id}/offer` | Oui | Retirer son offre d'aide |
| POST | `/{id}/reject` | Oui | Rejeter une offre d'aide |
| POST | `/{id}/confirm` | Oui | Confirmer la réalisation du besoin |
| POST | `/{id}/report` | Oui | Signaler un besoin (modération) |
| POST | `/{id}/block` | Oui | Bloquer un besoin (masquer) |
| DELETE | `/{id}/block` | Oui | Débloquer un besoin |

#### Messages (Entraide)

Préfixe : `/v1/community/wishes/{wish_id}/messages`

| Méthode | Chemin | Auth | Description |
|---|---|---|---|
| GET | `/` | Oui | Lister les messages d'un besoin (paginé) |
| POST | `/` | Oui | Envoyer un message sur un besoin |

### Notifications

Préfixe : `/v1/me/notifications`

| Méthode | Chemin | Auth | Description |
|---|---|---|---|
| GET | `/` | Oui | Lister les notifications (paginée) |
| POST | `/read` | Oui | Marquer toutes les notifications comme lues |
| POST | `/{id}/read` | Oui | Marquer une notification comme lue |
| DELETE | `/{id}` | Oui | Supprimer une notification |
| GET | `/unread-count` | Oui | Nombre de notifications non lues |

### Push Tokens (APNs)

Préfixe : `/v1/push-tokens`

| Méthode | Chemin | Auth | Description |
|---|---|---|---|
| POST | `/` | Oui | Enregistrer un token push APNs |
| DELETE | `/{token}` | Oui | Désenregistrer un token push |

### Share Links (Liens de partage)

Préfixe : `/v1/share-links`

| Méthode | Chemin | Auth | Description |
|---|---|---|---|
| GET | `/` | Oui | Lister mes liens de partage (paginé) |
| POST | `/` | Oui | Créer un lien de partage |
| PATCH | `/{id}` | Oui | Modifier un lien de partage |
| DELETE | `/{id}` | Oui | Supprimer un lien de partage |

### Upload

Préfixe : `/v1/upload`

| Méthode | Chemin | Auth | Description |
|---|---|---|---|
| POST | `/image` | Oui | Uploader une image (max 10 Mo, stockage R2) |

### Admin (Modération)

Préfixe : `/v1/admin`

| Méthode | Chemin | Auth | Description |
|---|---|---|---|
| GET | `/wishes/pending` | Oui | Lister les besoins en attente de modération |
| POST | `/wishes/{id}/approve` | Oui | Approuver un besoin |
| POST | `/wishes/{id}/reject` | Oui | Rejeter un besoin |

> Les endpoints admin sont réservés aux utilisateurs avec le rôle `admin`.

---

## Endpoints publics (hors `/v1`)

Ces endpoints sont accessibles sans authentification et sans préfixe de version :

| Méthode | Chemin | Description |
|---|---|---|
| GET | `/health` | Health check complet (DB + Redis) |
| GET | `/health/live` | Liveness probe (toujours `200`) |
| GET | `/health/ready` | Readiness probe (= health check) |
| GET | `/shared/{token}` | Vue publique d'une wishlist partagée (HTML ou JSON) |
| POST | `/shared/{token}/items/{item_id}/claim` | Réserver une envie via lien partagé |
| DELETE | `/shared/{token}/items/{item_id}/claim` | Annuler une réservation via lien partagé |
| POST | `/shared/{token}/items/{item_id}/web-claim` | Réservation web (sans compte) |
| DELETE | `/shared/{token}/items/{item_id}/web-claim` | Annulation réservation web |
| GET | `/join/{token}` | Page d'invitation à un cercle (HTML) |
| GET | `/legal/privacy` | Politique de confidentialité |
| GET | `/legal/terms` | Conditions générales d'utilisation |
| GET | `/legal/mentions` | Mentions légales |
| GET | `/favicon.png` | Favicon PNG 32x32 |
| GET | `/favicon.ico` | Favicon ICO |
| GET | `/metrics` | Métriques Prometheus (hors CORS) |

---

## Swagger UI

La documentation interactive OpenAPI est disponible à :

```
GET /docs/
```

| Environnement | Accès |
|---|---|
| Développement | Libre (compilé uniquement en `debug_assertions`) |
| Production | Non exposé (le build release n'inclut pas Swagger UI) |

Le schéma OpenAPI brut est servi à `/api-doc/openapi.json`.

---

## Limites et headers

| Paramètre | Valeur |
|---|---|
| Body max | 10 Mo |
| Timeout requête | 30 secondes |
| CORS origins | `offrii.com`, `api.offrii.com`, `cdn.offrii.com`, `staging.offrii.com` |
| CORS méthodes | GET, POST, PUT, PATCH, DELETE, OPTIONS |
| HSTS | `max-age=31536000; includeSubDomains` |
| X-Content-Type-Options | `nosniff` |
| X-Frame-Options | `DENY` |
