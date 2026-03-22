# Dev Fixtures

Données de démonstration pour les environnements dev et staging.

**Mot de passe universel** : `DemoPass123x`

## Activation

Le seed tourne automatiquement quand `SEED_DEV_DATA=true` dans le `.env`.
Il est idempotent — relancer ne duplique rien.

```bash
# Clean restart avec seed
docker compose down
docker volume rm offrii_postgres_data
docker compose up -d --build
```

---

## Comptes

| Email | Type | Vérifié | Admin | Avatar | Display Name |
|-------|------|---------|-------|--------|-------------|
| `yassine@demo.com` | Email + password | Oui | **Oui** | Oui | Yassine |
| `marie@demo.com` | Email + password | Oui | Non | Oui | Marie Dupont |
| `lucas@demo.com` | Email + password | **Non** | Non | Non | *(aucun)* |
| `sophie@gmail.com` | Google OAuth only | Oui | Non | Oui | Sophie Martin |
| `thomas@icloud.com` | Apple OAuth only | Oui | Non | Non | *(aucun)* |
| `camille@demo.com` | Email + Google lié | Oui | Non | Non | Camille R. |
| `newuser@demo.com` | Email + password | Oui | Non | Non | Nouveau |
| `reporter@demo.com` | Email + password | Oui | Non | Non | Reporter |

### Cas couverts

- **yassine** : admin, profil complet, compte ancien (30j), username customisé
- **marie** : user régulier complet, compte 14j
- **lucas** : email non vérifié, pas de display_name, username auto-généré
- **sophie** : OAuth Google only (pas de mot de passe), avatar Google
- **thomas** : OAuth Apple only (pas de mot de passe, pas d'avatar)
- **camille** : dual auth (email + Google lié), les deux fonctionnent
- **newuser** : compte créé il y a 1h → bloqué par `MIN_ACCOUNT_AGE_HOURS` pour l'entraide
- **reporter** : utilisé pour les scénarios de signalement et blocage

---

## Items (Envies)

20 items répartis principalement entre Yassine et Marie.

| Scénario | User | Description |
|----------|------|-------------|
| Basique | Yassine | Nom uniquement, priorité 2, pas de catégorie |
| Complet | Yassine | Tous les champs remplis (description, prix, liens, image, OG metadata) |
| Acheté | Yassine | `status=purchased`, `purchased_at` auto-set |
| Supprimé | Yassine | `status=deleted` (soft delete) |
| Priorité 1 (haute) | Yassine | Flamme haute |
| Priorité 3 (basse) | Yassine | Flamme basse |
| Privé | Yassine | `is_private=true`, invisible dans les partages par règle |
| Claimed via app | Yassine | Réservé par Marie via l'app |
| Claimed via web | Yassine | Réservé par un visiteur web ("Maman"), avec token |
| Avec liens multiples | Yassine | 2 URLs (Nike + Zalando) |
| Avec OG metadata | Yassine | `og_title`, `og_image_url`, `og_site_name` remplis |
| Sans catégorie | Yassine | `category_id=NULL` |
| Par catégorie | Yassine/Marie | Tech, Mode, Maison, Loisirs, Santé, Autre |
| Partagé en cercle | Yassine | Partagé explicitement via `circle_items` |

---

## Cercles (Proches)

| Cercle | Type | Owner | Membres | Image |
|--------|------|-------|---------|-------|
| Direct Yassine ↔ Marie | Direct (1-to-1) | Yassine | Yassine, Marie | Non |
| Direct Yassine ↔ Lucas | Direct (1-to-1) | Yassine | Yassine, Lucas | Non |
| Famille | Groupe | Marie | Marie, Yassine, Lucas, Camille | Oui |
| Amis proches | Groupe | Yassine | Yassine, Marie, Sophie | Non |

### Règles de partage

| Cercle | User | Mode | Détail |
|--------|------|------|--------|
| Direct Yassine ↔ Marie | Yassine | `all` | Partage tous les items actifs non-privés |
| Famille | Marie | `categories` | Partage uniquement Tech + Mode |
| Famille | Yassine | `selection` | Partage item par item (via circle_items) |
| Amis proches | Yassine | `none` | Pas de partage automatique |

---

## Amitiés

| Relation | Statut |
|----------|--------|
| Yassine ↔ Marie | Acceptée (+ cercle direct) |
| Yassine ↔ Lucas | Acceptée (+ cercle direct) |
| Marie ↔ Camille | Acceptée |
| Sophie → Yassine | Demande en attente |
| Lucas → Marie | Demande refusée |

---

## Entraide (Community Wishes)

| Besoin | Owner | Statut | Catégorie | Particularité |
|--------|-------|--------|-----------|---------------|
| Aide pour cours de maths | Marie | `open` | education | Disponible, avec description |
| Vêtements enfants taille 6 | Yassine | `matched` | clothing | Matché avec Marie, messages échangés |
| Consultation médicale | Marie | `fulfilled` | health | Comblé, avec `fulfilled_at` |
| Aide déménagement | Yassine | `closed` | home | Fermé par le propriétaire |
| Garde d'enfants | Reporter | `rejected` | children | Rejeté par modération |
| Besoin flaggé | Lucas | `flagged` | other | Signalé, `moderation_note` défini |
| Besoin en attente | Yassine | `pending` | education | En attente de modération |
| Besoin réouvert | Marie | `open` | home | `reopen_count=1`, a été réouvert |

### Messages (sur les wishes matchés/fulfilled)

- Vêtements enfants : 2 messages (Marie → Yassine, Yassine → Marie)
- Consultation médicale : 2 messages (donor → Marie, Marie → donor)

### Signalements

| Wish | Reporter | Raison | Détails |
|------|----------|--------|---------|
| Besoin flaggé | Yassine | `inappropriate` | *(aucun)* |
| Besoin flaggé | Marie | `spam` | "Ceci est du spam" |
| Besoin rejeté | Reporter | `other` | "Ne semble pas légitime" |

### Blocages

- Yassine bloque le besoin flaggé
- Marie bloque le besoin rejeté

---

## Share Links (Liens de partage)

| Label | Owner | Scope | Permissions | Statut |
|-------|-------|-------|-------------|--------|
| *(aucun)* | Yassine | `all` | `view_and_claim` | Actif, jamais expire |
| Pour la famille | Marie | `category` (Tech) | `view_and_claim` | Actif, expire dans 7j |
| Sélection | Yassine | `selection` | `view_and_claim` | Actif |
| Ancien lien | Yassine | `all` | `view_only` | **Désactivé** |

---

## Notifications

11 notifications couvrant tous les types :

| Type | Pour | Lu |
|------|------|----|
| `friend_request` | Yassine | Non |
| `friend_accepted` | Marie | Oui |
| `item_claimed` | Yassine | Non |
| `item_shared` | Marie | Non |
| `circle_member_joined` | Marie | Non |
| `wish_message` | Yassine | Non |
| `wish_offer` | Marie | Non |
| `wish_confirmed` | Marie | Oui |
| `wish_moderation_flagged` | Lucas | Non |
| `wish_rejected` | Reporter | Non |
| Web claim notification | Yassine | Non |

---

## Autres données

| Entité | Contenu |
|--------|---------|
| **Push tokens** | Yassine (2 devices iOS), Marie (1 iOS), Lucas (1 Android) |
| **Refresh tokens** | 2 actifs, 1 révoqué, 1 expiré |
| **Email verification** | 1 actif (Lucas, non vérifié), 1 expiré |
| **Circle invites** | 1 actif (Famille, 3 uses max, 1 utilisé), 1 expiré |
| **Circle events** | member_joined, item_shared, item_claimed, item_received |
