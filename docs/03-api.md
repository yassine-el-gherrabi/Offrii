# Reference API

## Base URL

```
https://api.offrii.com/v1/
```

Toutes les routes documentees ci-dessous sont prefixees par `/v1` sauf indication contraire (endpoints publics hors-versioning).

---

## Authentification

L'API utilise des **JSON Web Tokens (JWT)** signes avec l'algorithme asymetrique **RS256** (cle privee sur le serveur, cle publique pour verification).

| Parametre | Valeur |
|---|---|
| Algorithme | RS256 (asymetrique) |
| Duree access token | 15 minutes |
| Duree refresh token | 7 jours |
| Rotation | Atomique -- chaque appel a `/auth/refresh` invalide l'ancien refresh token et en emet un nouveau |
| Revocation | Blacklist via Redis (JTI) au logout |

### Format de l'en-tete

```http
Authorization: Bearer <access_token>
```

Les endpoints marques **Auth = Oui** dans les tables ci-dessous exigent cet en-tete. Sans token valide, l'API repond `401 Unauthorized`.

---

## Format de reponse

Toutes les reponses sont en **JSON** (`Content-Type: application/json`).

### Reponse paginee

Les endpoints de liste retournent une enveloppe paginee :

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

| Parametre query | Defaut | Min | Max | Description |
|---|---|---|---|---|
| `page` | 1 | 1 | -- | Numero de page |
| `limit` | 20 | 1 | 100 | Nombre d'elements par page |

### Reponse d'erreur

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
| `BadRequest` | 400 | `BAD_REQUEST` | Validation echouee, champs manquants |
| `Unauthorized` | 401 | `UNAUTHORIZED` | Token manquant, expire ou invalide |
| `Forbidden` | 403 | `FORBIDDEN` | Droits insuffisants (ex: pas proprietaire) |
| `NotFound` | 404 | `NOT_FOUND` | Ressource inexistante |
| `Conflict` | 409 | `CONFLICT` | Doublon (email deja pris, invitation existante) |
| `Gone` | 410 | `GONE` | Ressource supprimee definitivement |
| `TooManyRequests` | 429 | `TOO_MANY_REQUESTS` | Rate limiting depasse |
| `Internal` | 500 | `INTERNAL_ERROR` | Erreur serveur (message generique, pas de fuite) |
| `ServiceUnavailable` | 503 | `SERVICE_UNAVAILABLE` | Dependance indisponible (DB, Redis) |

> **Securite** : les erreurs `500` ne renvoient jamais le message interne -- toujours `"an internal error occurred"`.

---

## Endpoints par domaine

### Auth

Prefixe : `/v1/auth`

| Methode | Chemin | Auth | Description |
|---|---|---|---|
| POST | `/register` | Non | Inscription (email + mot de passe) |
| POST | `/login` | Non | Connexion (email ou username + mot de passe) |
| POST | `/refresh` | Non | Renouvellement du couple access/refresh token |
| POST | `/logout` | Oui | Deconnexion (blacklist du JTI) |
| POST | `/change-password` | Oui | Changement de mot de passe |
| POST | `/forgot-password` | Non | Envoi d'un code de reinitialisation par email |
| POST | `/verify-reset-code` | Non | Verification du code de reinitialisation |
| POST | `/reset-password` | Non | Reinitialisation du mot de passe avec le code |
| POST | `/verify-email` | Non | Verification de l'adresse email (via token) |
| POST | `/resend-verification` | Oui | Renvoi de l'email de verification |
| POST | `/google` | Non | Authentification OAuth Google (id_token) |
| POST | `/apple` | Non | Authentification OAuth Apple (id_token) |

### Users

Prefixe : `/v1/me`

| Methode | Chemin | Auth | Description |
|---|---|---|---|
| GET | `/profile` | Oui | Recuperer le profil de l'utilisateur connecte |
| PATCH | `/profile` | Oui | Modifier le profil (display_name, username, avatar) |
| DELETE | `/profile` | Oui | Supprimer le compte (RGPD) |
| GET | `/export` | Oui | Export complet des donnees (RGPD) |
| POST | `/email` | Oui | Demande de changement d'adresse email |

### Items (Envies)

Prefixe : `/v1/items`

| Methode | Chemin | Auth | Description |
|---|---|---|---|
| GET | `/` | Oui | Lister mes envies (pagine, filtres par statut) |
| POST | `/` | Oui | Creer une envie |
| GET | `/{id}` | Oui | Detail d'une envie |
| PATCH | `/{id}` | Oui | Modifier une envie |
| DELETE | `/{id}` | Oui | Supprimer une envie |
| POST | `/{id}/claim` | Oui | Reserver une envie (en tant que proche) |
| DELETE | `/{id}/claim` | Oui | Annuler sa reservation |
| DELETE | `/{id}/web-claim` | Oui | Annuler une reservation (par le proprietaire, web) |
| POST | `/batch-delete` | Oui | Suppression groupee d'envies |

### Categories

Prefixe : `/v1/categories`

| Methode | Chemin | Auth | Description |
|---|---|---|---|
| GET | `/` | Oui | Lister les categories d'envies disponibles |

### Circles (Proches)

Prefixe : `/v1/circles`

| Methode | Chemin | Auth | Description |
|---|---|---|---|
| GET | `/` | Oui | Lister mes cercles |
| POST | `/` | Oui | Creer un cercle |
| GET | `/{id}` | Oui | Detail d'un cercle |
| PATCH | `/{id}` | Oui | Modifier un cercle (nom, emoji) |
| DELETE | `/{id}` | Oui | Supprimer un cercle |
| POST | `/direct/{user_id}` | Oui | Creer un cercle direct (1-a-1 avec un ami) |
| POST | `/{id}/invite` | Oui | Generer un lien d'invitation |
| POST | `/join/{token}` | Oui | Rejoindre un cercle via invitation |
| POST | `/{id}/members` | Oui | Ajouter un membre (ami) |
| DELETE | `/{id}/members/{uid}` | Oui | Retirer un membre |
| GET | `/{id}/invites` | Oui | Lister les invitations actives |
| DELETE | `/{id}/invites/{iid}` | Oui | Revoquer une invitation |
| POST | `/{id}/items` | Oui | Partager une envie dans un cercle |
| GET | `/{id}/items` | Oui | Lister les envies partagees dans un cercle |
| POST | `/{id}/items/batch` | Oui | Partager plusieurs envies d'un coup |
| GET | `/{id}/items/{item_id}` | Oui | Detail d'une envie partagee |
| DELETE | `/{id}/items/{item_id}` | Oui | Retirer une envie d'un cercle |
| GET | `/{id}/share-rule` | Oui | Voir la regle de partage automatique |
| PUT | `/{id}/share-rule` | Oui | Definir la regle de partage automatique |
| GET | `/{id}/feed` | Oui | Fil d'actualite du cercle |
| POST | `/{id}/transfer` | Oui | Transferer la propriete du cercle |
| GET | `/my-reservations` | Oui | Lister mes reservations dans tous les cercles |
| GET | `/my-share-rules` | Oui | Lister mes regles de partage |

### Friends (Amis)

Prefixe : `/v1/me` (demandes) et `/v1/users` (recherche)

| Methode | Chemin | Auth | Description |
|---|---|---|---|
| GET | `/users/search` | Oui | Rechercher un utilisateur (par nom ou username) |
| POST | `/me/friend-requests` | Oui | Envoyer une demande d'ami |
| GET | `/me/friend-requests` | Oui | Lister les demandes recues en attente |
| GET | `/me/friend-requests/sent` | Oui | Lister les demandes envoyees |
| POST | `/me/friend-requests/{id}/accept` | Oui | Accepter une demande |
| DELETE | `/me/friend-requests/{id}/cancel` | Oui | Annuler une demande envoyee |
| DELETE | `/me/friend-requests/{id}` | Oui | Refuser une demande recue |
| GET | `/me/friends` | Oui | Lister mes amis |
| DELETE | `/me/friends/{user_id}` | Oui | Supprimer un ami |

### Community Wishes -- Entraide (Besoins)

Prefixe : `/v1/community/wishes`

| Methode | Chemin | Auth | Description |
|---|---|---|---|
| GET | `/` | Oui | Lister les besoins communautaires (pagine) |
| POST | `/` | Oui | Creer un besoin |
| GET | `/mine` | Oui | Lister mes besoins |
| GET | `/my-offers` | Oui | Lister les besoins ou j'ai propose mon aide |
| GET | `/recent-fulfilled` | Oui | Besoins recemment combles |
| GET | `/{id}` | Oui | Detail d'un besoin |
| PATCH | `/{id}` | Oui | Modifier un besoin |
| DELETE | `/{id}` | Oui | Supprimer un besoin |
| POST | `/{id}/close` | Oui | Cloturer un besoin |
| POST | `/{id}/reopen` | Oui | Rouvrir un besoin cloture |
| POST | `/{id}/offer` | Oui | Proposer son aide |
| DELETE | `/{id}/offer` | Oui | Retirer son offre d'aide |
| POST | `/{id}/reject` | Oui | Rejeter une offre d'aide |
| POST | `/{id}/confirm` | Oui | Confirmer la realisation du besoin |
| POST | `/{id}/report` | Oui | Signaler un besoin (moderation) |
| POST | `/{id}/block` | Oui | Bloquer un besoin (masquer) |
| DELETE | `/{id}/block` | Oui | Debloquer un besoin |

#### Messages (Entraide)

Prefixe : `/v1/community/wishes/{wish_id}/messages`

| Methode | Chemin | Auth | Description |
|---|---|---|---|
| GET | `/` | Oui | Lister les messages d'un besoin (pagine) |
| POST | `/` | Oui | Envoyer un message sur un besoin |

### Notifications

Prefixe : `/v1/me/notifications`

| Methode | Chemin | Auth | Description |
|---|---|---|---|
| GET | `/` | Oui | Lister les notifications (paginee) |
| POST | `/read` | Oui | Marquer toutes les notifications comme lues |
| POST | `/{id}/read` | Oui | Marquer une notification comme lue |
| DELETE | `/{id}` | Oui | Supprimer une notification |
| GET | `/unread-count` | Oui | Nombre de notifications non lues |

### Push Tokens (APNs)

Prefixe : `/v1/push-tokens`

| Methode | Chemin | Auth | Description |
|---|---|---|---|
| POST | `/` | Oui | Enregistrer un token push APNs |
| DELETE | `/{token}` | Oui | Desenregistrer un token push |

### Share Links (Liens de partage)

Prefixe : `/v1/share-links`

| Methode | Chemin | Auth | Description |
|---|---|---|---|
| GET | `/` | Oui | Lister mes liens de partage (pagine) |
| POST | `/` | Oui | Creer un lien de partage |
| PATCH | `/{id}` | Oui | Modifier un lien de partage |
| DELETE | `/{id}` | Oui | Supprimer un lien de partage |

### Upload

Prefixe : `/v1/upload`

| Methode | Chemin | Auth | Description |
|---|---|---|---|
| POST | `/image` | Oui | Uploader une image (max 10 Mo, stockage R2) |

### Admin (Moderation)

Prefixe : `/v1/admin`

| Methode | Chemin | Auth | Description |
|---|---|---|---|
| GET | `/wishes/pending` | Oui | Lister les besoins en attente de moderation |
| POST | `/wishes/{id}/approve` | Oui | Approuver un besoin |
| POST | `/wishes/{id}/reject` | Oui | Rejeter un besoin |

> Les endpoints admin sont reserves aux utilisateurs avec le role `admin`.

---

## Endpoints publics (hors `/v1`)

Ces endpoints sont accessibles sans authentification et sans prefixe de version :

| Methode | Chemin | Description |
|---|---|---|
| GET | `/health` | Health check complet (DB + Redis) |
| GET | `/health/live` | Liveness probe (toujours `200`) |
| GET | `/health/ready` | Readiness probe (= health check) |
| GET | `/shared/{token}` | Vue publique d'une wishlist partagee (HTML ou JSON) |
| POST | `/shared/{token}/items/{item_id}/claim` | Reserver une envie via lien partage |
| DELETE | `/shared/{token}/items/{item_id}/claim` | Annuler une reservation via lien partage |
| POST | `/shared/{token}/items/{item_id}/web-claim` | Reservation web (sans compte) |
| DELETE | `/shared/{token}/items/{item_id}/web-claim` | Annulation reservation web |
| GET | `/join/{token}` | Page d'invitation a un cercle (HTML) |
| GET | `/legal/privacy` | Politique de confidentialite |
| GET | `/legal/terms` | Conditions generales d'utilisation |
| GET | `/legal/mentions` | Mentions legales |
| GET | `/favicon.png` | Favicon PNG 32x32 |
| GET | `/favicon.ico` | Favicon ICO |
| GET | `/metrics` | Metriques Prometheus (hors CORS) |

---

## Swagger UI

La documentation interactive OpenAPI est disponible a :

```
GET /docs/
```

| Environnement | Acces |
|---|---|
| Developpement | Libre (compile uniquement en `debug_assertions`) |
| Production | Non expose (le build release n'inclut pas Swagger UI) |

Le schema OpenAPI brut est servi a `/api-doc/openapi.json`.

---

## Limites et headers

| Parametre | Valeur |
|---|---|
| Body max | 10 Mo |
| Timeout requete | 30 secondes |
| CORS origins | `offrii.com`, `api.offrii.com`, `cdn.offrii.com`, `staging.offrii.com` |
| CORS methodes | GET, POST, PUT, PATCH, DELETE, OPTIONS |
| HSTS | `max-age=31536000; includeSubDomains` |
| X-Content-Type-Options | `nosniff` |
| X-Frame-Options | `DENY` |
