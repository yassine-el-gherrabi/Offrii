# Règles Métier

> **Objectif de cette page** : documenter les invariants métier d'Offrii, les machines à états, les constantes
> configurables et les garde-fous anti-abus. Chaque règle est justifiée par un *pourquoi* métier, pas
> seulement un *quoi* technique.

---

## Table des matières

1. [Machine à états -- Envies (Items)](#1-machine-a-etats--envies-items)
2. [Machine à états -- Entraide (Community Wishes)](#2-machine-a-etats--entraide-community-wishes)
3. [Constantes métier](#3-constantes-metier)
4. [Cercles et partage](#4-cercles-et-partage)
5. [Modération](#5-moderation)
6. [Anti-spam et rate limiting](#6-anti-spam-et-rate-limiting)
7. [Notifications](#7-notifications)
8. [Amis](#8-amis)

---

## 1. Machine à états -- Envies (Items)

Les **envies** (items) représentent les souhaits personnels d'un utilisateur : objets désirés, idées cadeaux, etc.
Elles sont visibles uniquement par l'utilisateur et les membres des cercles avec lesquels elles sont partagées.

### 1.1 États

| État | Description | Visible dans les listes ? |
|------|-------------|---------------------------|
| `active` | Envie active, visible et réservable par les proches | Oui |
| `purchased` | Marquée comme achetée/offerte | Oui (section "achetés") |
| `deleted` | Suppression logique (soft delete) | Non |

### 1.2 Transitions

```
                 +-----------+
                 |  active   |
                 +-----+-----+
                       |
          +------------+------------+
          |                         |
    claim (ami)              update status
          |                    "purchased"
          v                         |
   active + claimed_by        +-----v-----+
          |                   | purchased  |
     unclaim                  +-----------+
          |
          v
     active (libre)

     Depuis tout état non-terminal :
          |
      soft delete
          |
          v
     +---------+
     | deleted |
     +---------+
```

### 1.3 Règles de transition

| Transition | Qui peut la déclencher | Conditions | Pourquoi |
|-----------|----------------------|------------|----------|
| Créer une envie | Propriétaire | Priorité entre 1 et 3, prix >= 0, catégorie valide si fournie | Borner la priorité évite les valeurs absurdes ; le prix négatif n'a pas de sens métier |
| Passer à `purchased` | Propriétaire | Envie en statut `active` | Seul le propriétaire sait si le cadeau a été reçu |
| Soft delete | Propriétaire | Envie non supprimée | Suppression logique pour préserver l'intégrité référentielle (historique de cercle, événements) |
| Batch delete | Propriétaire | Maximum 100 envies par appel | Limiter l'impact d'un appel unique sur la base de données |
| Claim (réserver) | Ami (via cercle) | Envie `active`, non déjà réservée, pas sa propre envie | Un ami réserve un cadeau pour l'offrir -- il ne peut pas réserver le sien |
| Unclaim | Le réservataire | Envie claimée par lui | Seul celui qui a réservé peut annuler pour éviter les conflits |
| Owner unclaim (web) | Propriétaire | Uniquement si `claimed_via = "web"` | Le propriétaire peut retirer un claim web (anonyme) mais pas un claim app (l'ami a explicitement réservé) |

### 1.4 Flag `is_private`

Quand une envie est marquée `is_private = true` :

- Toutes les entrées `circle_items` associées sont **immédiatement supprimées** ;
- Les règles de partage automatique (`all`, `categories`) l'**excluent** systématiquement ;
- L'envie reste visible uniquement par son propriétaire.

**Pourquoi** : un utilisateur doit pouvoir garder certains souhaits strictement privés (cadeaux surprises
pour soi-même, envies sensibles) sans risquer une fuite via le partage automatique.

### 1.5 Enrichissement OG (Open Graph)

Lors de la création ou de la mise à jour d'une envie avec des liens, le système récupère les métadonnées
Open Graph (image, titre, nom du site) de manière **asynchrone** (fire-and-forget via `tokio::spawn`).

**Pourquoi** : améliorer l'expérience utilisateur en affichant un aperçu riche du lien sans bloquer la
réponse API. Si la récupération échoue, l'envie reste valide sans aperçu.

### 1.6 Cache Redis

Les listes d'envies sont cachées dans Redis avec un système de **versioning** :

- Clé : `items:{user_id}:v{version}:{params}`
- TTL : **300 secondes** (5 minutes)
- Toute mutation (création, édition, suppression, claim) **incrémente la version**, ce qui invalide
  automatiquement toutes les entrées de cache précédentes.

**Pourquoi** : éviter les requêtes SQL coûteuses sur les listings paginés tout en garantissant la cohérence
après chaque mutation.

---

## 2. Machine à états -- Entraide (Community Wishes)

L'**entraide** est le cœur social d'Offrii : un mur communautaire où les utilisateurs publient des besoins
et où d'autres proposent leur aide. Ce système est beaucoup plus complexe que les envies personnelles car il
implique la modération, les signalements et l'interaction entre inconnus.

### 2.1 États

| État | Description | Visible publiquement ? |
|------|-------------|----------------------|
| `pending` | En attente de modération IA | Non |
| `open` | Publié et visible sur le mur | Oui |
| `flagged` | Rejeté par l'IA ou service indisponible, en attente de review admin | Non |
| `review` | Seuil de signalements communautaires atteint | Non |
| `matched` | Un donneur s'est proposé | Oui (avec mention "en cours") |
| `fulfilled` | Le propriétaire a confirmé le don | Non (archive) |
| `closed` | Fermé par le propriétaire | Non (archive) |
| `rejected` | Rejeté par un administrateur | Non (archive) |

### 2.2 Diagramme de transitions

```
                      création
                         |
                         v
                    +---------+
                    | pending |  (modération IA asynchrone)
                    +----+----+
                         |
              +----------+----------+
              |                     |
         IA approuve           IA flagge ou indisponible
              |                     |
              v                     v
         +--------+           +---------+
         |  open  |           | flagged |
         +---+----+           +----+----+
             |                     |
    +--------+--------+     admin approve / admin reject
    |        |        |            |          |
  offer   report   close     +----+----+     v
    |        |        |      |  open   |  +----------+
    v        v        v      +---------+  | rejected |
+--------+ seuil  +--------+             +----------+
| matched| atteint| closed |
+---+----+   |    +---+----+
    |        v        ^
    |   +--------+    |
    |   | review |----+  (reopen par le propriétaire)
    |   +--------+
    |
    +--------+--------+
    |        |        |
 confirm  reject   withdraw
  (owner)  (owner)  (donor)
    |        |        |
    v        v        v
+-----------+ +------+ +------+
| fulfilled | | open | | open |
+-----------+ +------+ +------+
```

### 2.3 Règles de transition détaillées

| Transition | Déclencheur | Pré-conditions | Effet | Pourquoi |
|-----------|------------|----------------|-------|----------|
| `creation` -> `pending` | Utilisateur | Compte âgé de 24h+, email vérifié, < 3 besoins actifs, display_name requis si non-anonyme | Le besoin est créé en `pending`, modération IA lancée en arrière-plan | Empêcher le spam de nouveaux comptes ; limiter l'engorgement du mur |
| `pending` -> `open` | Système (IA) | Modération IA approuve le contenu | Le besoin devient visible sur le mur | La modération IA filtre les contenus inappropriés avant publication |
| `pending` -> `flagged` | Système (IA) | L'IA détecte un contenu problématique OU le service est indisponible après 3 tentatives | Le besoin est invisible, en attente d'un admin | En cas de doute (y compris panne), on préfère la prudence : un humain tranchera |
| `open` -> `matched` | Donneur | Compte âgé de 24h+, email vérifié, pas le propriétaire, pas de signalement en cours | `matched_with` = donneur, `matched_at` = maintenant | Le donneur exprime sa volonté d'aider ; le besoin reste visible mais réservé |
| `matched` -> `open` | Donneur (`withdraw`) | Statut `matched`, c'est bien le donneur actuel | Match effacé, messages supprimés | Le donneur peut se rétracter ; les messages sont purgés pour la confidentialité du prochain donneur |
| `matched` -> `open` | Propriétaire (`reject`) | Statut `matched`, c'est bien le propriétaire | Match effacé, messages supprimés | Le propriétaire peut décliner une offre inadaptée |
| `matched` -> `fulfilled` | Propriétaire (`confirm`) | Statut `matched` | `fulfilled_at` = maintenant | Le propriétaire confirme avoir reçu l'aide promise |
| `open` -> `review` | Système | `report_count >= 5` (seuil) | Besoin retiré du mur en attendant un admin | La communauté s'automodère : si 5 personnes signalent, il y a probablement un problème |
| `review`/`closed` -> `pending` | Propriétaire (`reopen`) | `reopen_count < 2`, cooldown de 24h respecté | Compteur de reopens incrémenté, signalements remis à zéro, re-modération IA | Permettre à l'auteur de corriger son besoin, mais limiter les abus de réouverture |
| `open`/`matched`/... -> `closed` | Propriétaire | Pas en état terminal | `closed_at` = maintenant, notifié le donneur si matched | L'auteur peut toujours fermer son besoin de lui-même |
| `flagged`/`review` -> `open` | Admin | Statut `flagged` ou `review` | Approuvé manuellement | Un humain valide que le contenu est conforme |
| `flagged`/`review` -> `rejected` | Admin | Statut `flagged` ou `review` | Rejeté définitivement | Le contenu enfreint les règles communautaires |
| Édition du contenu | Propriétaire | Statut `open`, `review` ou `closed` | Retour en `pending`, signalements remis à zéro, re-modération | Le contenu a changé, les anciens signalements ne sont plus pertinents |
| Suppression | Propriétaire | Pas en `matched` ni `fulfilled` | Suppression physique | On ne peut pas supprimer un besoin en cours d'aide (protection du donneur) ; les besoins fulfills sont conservés pour l'historique |

### 2.4 Blocage

Un utilisateur peut **bloquer** un besoin (`wish_blocks`). Les besoins bloqués sont masqués de son fil
sans impact sur les autres utilisateurs. C'est une opération locale et silencieuse.

**Pourquoi** : permettre à chaque utilisateur de personnaliser son expérience sans impacter la visibilité
globale.

---

## 3. Constantes métier

Toutes les constantes proviennent du code source (`community_wish_service.rs`, `item_service.rs`,
`auth_service.rs`, `upload_service.rs`).

### 3.1 Entraide

| Constante | Valeur | Justification |
|-----------|--------|---------------|
| `MAX_ACTIVE_WISHES_PER_USER` | **3** | Éviter l'engorgement du mur d'entraide. Si un utilisateur pouvait publier 50 besoins, le mur serait dominé par quelques profils. 3 est un compromis entre expression et équité. |
| `MIN_ACCOUNT_AGE_HOURS` | **24h** | Barrière anti-spam : un compte créé il y a 5 minutes est probablement un bot ou un troll. 24h force une intention réelle. |
| `WISH_REPORT_THRESHOLD` | **5** | Seuil d'auto-modération communautaire. Trop bas (2) provoquerait des faux positifs par grief ; trop haut (20) laisserait du contenu problématique visible trop longtemps. |
| `MAX_REPORTS_PER_USER_PER_DAY` | **10** | Empêcher un utilisateur malveillant de signaler massivement pour provoquer la modération de besoins légitimes. |
| `MAX_REOPEN_COUNT` | **2** | Limiter les boucles de réouverture abusives. Après 2 tentatives, seul un admin peut intervenir. |
| `REOPEN_COOLDOWN_HOURS` | **24h** | Forcer la réflexion avant de rouvrir. Éviter les reopens impulsifs en boucle. |
| `CACHE_TTL_SECS` (wishes) | **60s** | Le mur est plus dynamique que les envies personnelles. Un TTL court (1 min) garantit la fraîcheur sans surcharger la DB. |

### 3.2 Envies (Items)

| Constante | Valeur | Justification |
|-----------|--------|---------------|
| `ALLOWED_SORTS` | `created_at`, `priority`, `name` | Whitelist stricte pour empêcher l'injection SQL via le paramètre de tri. |
| `MAX_PAGE` | **1000** | Éviter les OFFSET énormes qui dégradent les performances PostgreSQL. |
| `LIST_CACHE_TTL_SECS` | **300s** (5 min) | Les listes d'envies changent moins souvent que le mur communautaire. Un TTL plus long réduit la charge DB. |
| Priorité | **1 à 3** | 1 = faible, 2 = normal, 3 = haute. Défaut à 2. Trois niveaux suffisent pour exprimer l'urgence sans complexité inutile. |
| Batch delete max | **100** | Protéger la base de données contre les suppressions massives en un seul appel. |

### 3.3 Authentification

| Constante | Valeur | Justification |
|-----------|--------|---------------|
| `MAX_REFRESH_TOKENS_PER_USER` | **5** | Un utilisateur a typiquement 1-3 appareils (iPhone, iPad, Mac). 5 laisse une marge pour les navigateurs web tout en limitant la surface d'attaque. Les tokens les plus anciens sont révoqués automatiquement. |
| Login rate limit | **10 tentatives / 5 min** | Standard OWASP pour le brute-force. Après 10 échecs, le compte est temporairement bloqué. |
| Password reset rate | **1/min**, **3/5min**, **10/jour** | Triple layer de protection : empêcher le flood de mails (coût), le spam (UX), et le harcèlement par email. |
| Reset code TTL | **30 min** | Assez long pour consulter sa boîte mail, assez court pour qu'un code intercepté ne soit pas exploitable longtemps. |
| Reset code brute-force | **5 tentatives** | Un code à 6 chiffres (1M combinaisons) est cassé en 200 000 tentatives en moyenne. Limiter à 5 rend le brute-force impossible. |
| Email verification cooldown | **5 min** | Éviter de surcharger le service d'envoi d'emails. |
| Dummy hash (timing attack) | `DUMMY_HASH` | Quand un email n'existe pas en base, on vérifie quand même un hash bidon pour que le temps de réponse soit identique à celui d'un "mauvais mot de passe". Empêche l'énumération d'emails par timing. |
| Token versioning | `token_version` | Chaque changement de mot de passe ou opération sensible incrémente la version. Les tokens émis avec une version antérieure sont immédiatement refusés. Permet la révocation de masse sans blacklister individuellement chaque token. |

### 3.4 Upload

| Constante | Valeur | Justification |
|-----------|--------|---------------|
| `MAX_UPLOAD_BYTES` | **5 Mo** | Équilibre entre qualité d'image et bande passante mobile. Au-delà, le temps d'upload dégrade l'UX sur 4G. |
| `WEBP_QUALITY` | **75** | Qualité lossy WebP. 75 offre un bon ratio qualité/taille (réduction ~70% vs JPEG à qualité équivalente). |
| `ALLOWED_TYPES` | `jpeg`, `png`, `webp`, `heic`, `heif` | Supporter les formats courants iOS (HEIC) et Android (JPEG/PNG) + WebP natif. |
| Avatar/circle resize | **400x400** | Crop carré puis réduction. 400px est suffisant pour les avatars Retina (200pt x @2x) sans gaspiller du stockage. |
| Item resize | **max 800px largeur** | Assez large pour un affichage plein écran mobile, assez petit pour un chargement rapide. |
| EXIF orientation | Appliquée avant crop | Les photos iPhone ont souvent l'orientation en métadonnées EXIF plutôt qu'en pixels. Sans correction, les images apparaissent pivotées. |

---

## 4. Cercles et partage

Les **cercles** sont le mécanisme de partage d'Offrii. Un cercle est un groupe d'utilisateurs qui voient les envies des autres membres.

### 4.1 Types de cercles

| Type | `is_direct` | Création | Suppression | Renommage |
|------|-------------|----------|-------------|-----------|
| **Cercle direct** | `true` | Automatique à l'acceptation d'une amitié | Automatique à la suppression de l'amitié | Interdit (affiche le nom de l'ami) |
| **Cercle de groupe** | `false` | Manuelle par un utilisateur | Par le propriétaire uniquement | Par tout membre |

### 4.2 Cercles directs (max 2 participants)

Quand une amitié est acceptée, le système crée automatiquement un cercle direct entre les deux amis. Ce cercle :

- Ne contient que 2 membres (les deux amis) ;
- N'a pas de nom (l'interface affiche le nom de l'autre membre) ;
- Ne peut pas être supprimé manuellement (il suit le cycle de vie de l'amitié) ;
- Ne peut pas avoir de membres supplémentaires.

**Pourquoi** : simplifier le partage un-à-un. L'utilisateur ajoute un ami et peut immédiatement partager
ses envies avec lui, sans créer manuellement un cercle. Le cercle direct matérialise la relation d'amitié
dans le système de partage.

### 4.3 Modes de partage (`circle_share_rules`)

Chaque membre d'un cercle définit ses propres règles de partage pour ce cercle :

| Mode | Comportement | Cas d'usage |
|------|-------------|-------------|
| `none` | Aucune envie n'est partagée avec ce cercle | "Je veux voir les envies des autres mais pas montrer les miennes" |
| `all` | Toutes les envies actives et non-privées sont automatiquement partagées | "Ma famille voit tout" |
| `categories` | Seules les envies des catégories sélectionnées (`category_ids`) sont partagées | "Mes collègues ne voient que ma catégorie Tech" |
| `selection` | Partage explicite envie par envie via `circle_items` | "Je choisis manuellement ce que je partage" (ancien mode, rétrocompatibilité) |

**Rétrocompatibilité** : si un membre n'a aucune règle (`NULL`), le système retombe en mode `selection`
implicite basée sur les entrées `circle_items` existantes.

**Invariant critique** : les envies marquées `is_private = true` ne sont **jamais** partagées via les
règles automatiques (`all`, `categories`), même si la règle correspond.

### 4.4 Invitations (token-based)

L'accès aux cercles de groupe se fait par invitation via un token :

| Propriété | Détail |
|-----------|--------|
| Format du token | 32 caractères alphanumériques aléatoires |
| Durée de validité | Configurable, défaut 24h, min 1h, max 30 jours (720h) |
| Nombre d'utilisations | Configurable, défaut 1, minimum 1 |
| Révocation | Par le créateur de l'invite OU par le propriétaire du cercle |

Le join est **transactionnel** : l'incrément du compteur d'utilisation, l'ajout du membre et la création
de l'événement se font dans une seule transaction. Si l'ajout échoue, l'invite n'est pas consommée.

**Pourquoi** : le système de tokens évite de devoir être amis pour rejoindre un cercle de groupe
(cas d'usage : famille élargie, collègues). Le contrôle par utilisations et expiration limite
les risques de diffusion non souhaitée.

### 4.5 Événements de cercle (audit log)

Chaque action significative dans un cercle génère un événement dans `circle_events` :

| Type d'événement | Déclencheur | Données |
|-----------------|------------|---------|
| `item_shared` | Partage d'une envie | `actor_id`, `item_id` |
| `item_unshared` | Retrait d'une envie | `actor_id`, `item_id` |
| `item_claimed` | Réservation d'une envie | `actor_id`, `item_id` |
| `item_unclaimed` | Annulation de réservation | `actor_id`, `item_id` |
| `member_joined` | Nouveau membre | `actor_id` |
| `member_left` | Départ d'un membre | `actor_id` |

**Anti-spoiler** : les événements de type `item_claimed` et `item_unclaimed` sont masqués au propriétaire
de l'envie. Cela préserve la surprise du cadeau.

**Pourquoi** : le log d'événements sert à la fois à l'affichage de la dernière activité dans la liste
des cercles et à la traçabilité pour le debugging.

### 4.6 Réservation dans un cercle

Quand un membre réserve une envie dans un cercle :

1. L'envie est marquée `claimed_by` + `claimed_via` ("app" ou "web") ;
2. Un événement `item_claimed` est enregistré ;
3. Tous les membres du cercle (sauf le propriétaire de l'envie) sont notifiés ;
4. Le cache du propriétaire est invalidé.

La distinction `claimed_via` est importante :
- **"app"** : réservé explicitement par un utilisateur identifié. Seul le réservataire peut annuler.
- **"web"** : réservé via le lien de partage web (anonyme). Le propriétaire peut aussi annuler.

---

## 5. Modération

### 5.1 Modération IA (OpenAI)

Chaque besoin communautaire passe par une vérification IA **avant** publication :

1. Le besoin est créé en statut `pending` ;
2. Un `tokio::spawn` lance la vérification via `ModerationService.check_wish()` ;
3. Le service envoie le titre, la description, la catégorie, l'image et les liens à l'API OpenAI ;
4. **3 tentatives** avec backoff exponentiel (2s, 4s) en cas d'échec ;
5. Résultat :
   - `Approved` -> statut `open`, notification "Besoin publié !"
   - `Flagged { reason }` -> statut `flagged`, notification "Besoin en révision"
   - Échec après 3 tentatives -> statut `flagged` (failsafe)

**Pourquoi le failsafe flagge plutôt qu'approuve** : en cas de panne du service de modération, il vaut
mieux retarder la publication (un admin validera manuellement) que de laisser passer du contenu
potentiellement dangereux.

### 5.2 Signalements communautaires

| Règle | Détail | Pourquoi |
|-------|--------|----------|
| Qui peut signaler | Tout utilisateur avec email vérifié et compte âgé de 24h+ | Mêmes barrières que pour la création, pour éviter les signalements de spam-bots |
| Pas son propre besoin | Le propriétaire ne peut pas signaler son besoin | Incohérent et potentiellement abusif |
| Seuls les besoins `open` | On ne peut pas signaler un besoin déjà en review ou fermé | Inutile et pourrait corrompre les compteurs |
| Limite journalière | 10 signalements par utilisateur par jour | Empêcher le "report bombing" (spam de signalements contre des besoins légitimes) |
| Un seul signalement par besoin | Un utilisateur ne peut signaler le même besoin qu'une fois | Éviter le gonflement artificiel du compteur |
| Seuil automatique | 5 signalements -> passage en `review` | La communauté retire le contenu du mur ; un admin tranchera |

### 5.3 File d'attente admin

Les administrateurs disposent d'un endpoint dédié (`admin_list_flagged`) qui liste les besoins en statut
`flagged` et `review`, avec :

- Le contenu complet (titre, description, catégorie, image, liens) ;
- La note de modération IA (`moderation_note`) ;
- Le nombre de signalements ;
- L'identifiant du propriétaire.

Actions possibles :
- **Approuver** : passage à `open` + notification au propriétaire ;
- **Rejeter** : passage à `rejected` + notification au propriétaire.

### 5.4 Re-modération à l'édition

Quand un propriétaire modifie le contenu d'un besoin :

1. Les signalements sont **remis à zéro** (le contenu a changé, les anciens signalements ne sont plus pertinents) ;
2. La `moderation_note` est effacée ;
3. Le statut passe à `pending` ;
4. Une nouvelle vérification IA est lancée.

**Pourquoi** : un besoin initialement approuvé pourrait devenir problématique après édition. Il faut
re-vérifier systématiquement.

---

## 6. Anti-spam et rate limiting

### 6.1 Rate limits par service

| Endpoint / Action | Limite | Fenêtre | Stockage | Pourquoi |
|------------------|--------|---------|----------|----------|
| Login | 10 tentatives | 5 min | Redis INCR + EXPIRE | Protection brute-force standard OWASP |
| Password reset request | 1 / email | 60s | Redis SET NX | Empêcher le flood de mails |
| Password reset resend | 3 / email | 5 min | Redis INCR | Limite supplémentaire anti-abus |
| Password reset resend (jour) | 10 / email | 24h | Redis INCR | Plafond journalier pour empêcher le harcèlement par email |
| Reset code verification | 5 tentatives / email | 30 min | Redis INCR | Rendre le brute-force du code 6 chiffres impossible |
| Email verification resend | 1 | 5 min | DB (token `created_at`) | Éviter la surcharge du service d'envoi |
| Email change request | 1 | 5 min | DB (token `created_at`) | Idem |
| Signalements communautaires | 10 / utilisateur | 24h | DB (requête COUNT) | Empêcher le report bombing |

### 6.2 Barrières d'entrée pour les actions communautaires

Certaines actions sur le mur d'entraide exigent des pré-requis :

| Action | Email vérifié | Compte âgé de 24h+ |
|--------|:------------:|:------------------:|
| Créer un besoin | Oui | Oui |
| Proposer son aide (`offer`) | Oui | Oui |
| Signaler un besoin | Oui | Oui |
| Consulter le mur | Non | Non |

**Pourquoi la double barrière** : l'email vérifié prouve que l'adresse est réelle (pas de spam) ; l'âge
du compte empêche les bots de créer et agir dans la même minute.

### 6.3 Recherche utilisateurs

La recherche d'amis par username est bornée :

- Requête vide ou > 50 caractères -> résultat vide (pas d'erreur) ;
- Matching par préfixe (`ILIKE 'query%'`) ;
- Limite à 10 résultats ;
- Exclut l'appelant des résultats.

**Pourquoi** : éviter les requêtes LIKE coûteuses sur des patterns longs et limiter l'énumération d'utilisateurs.

---

## 7. Notifications

### 7.1 Architecture

Les notifications suivent un pattern **fire-and-forget** : l'action métier (création d'ami, réservation
d'envie, etc.) s'exécute de manière synchrone, puis la notification est envoyée via `tokio::spawn`
sans bloquer la réponse.

Deux canaux de livraison :

| Canal | Méthode | Persistance |
|-------|---------|-------------|
| **In-app** | Insert dans la table `notifications` | Oui (consultable dans le centre de notifications) |
| **Push (APNs)** | Envoi via le service de notification, batch par appareil | Non (éphémère) |

### 7.2 Types de notifications

| Type | Déclencheur | Destinataire |
|------|------------|-------------|
| `friend_request` | Envoi d'une demande d'ami | Destinataire de la demande |
| `friend_accepted` | Acceptation d'une demande | Émetteur de la demande |
| `item_shared` | Partage d'une envie dans un cercle | Tous les membres du cercle (sauf l'auteur) |
| `item_claimed` | Réservation d'une envie | Membres du cercle (sauf le propriétaire de l'envie -- anti-spoiler) |
| `item_unclaimed` | Annulation de réservation | Membres du cercle (sauf le propriétaire de l'envie) |
| `circle_member_joined` | Arrivée dans un cercle via invite | Tous les membres (sauf le nouveau) |
| `wish_moderation_approved` | Besoin approuvé par l'IA | Propriétaire du besoin |
| `wish_moderation_flagged` | Besoin flaggé par l'IA | Propriétaire du besoin |
| `wish_offer` | Un donneur propose son aide | Propriétaire du besoin |
| `wish_offer_withdrawn` | Le donneur retire son offre | Propriétaire du besoin |
| `wish_offer_rejected` | Le propriétaire décline l'offre | Donneur |
| `wish_confirmed` | Le propriétaire confirme le don | Donneur |
| `wish_closed` | Le propriétaire ferme un besoin matched | Donneur |
| `wish_reported` | Seuil de signalements atteint | Propriétaire du besoin |
| `wish_approved` | Admin approuve un besoin flaggé/review | Propriétaire du besoin |
| `wish_rejected` | Admin rejette un besoin | Propriétaire du besoin |

### 7.3 Localisation des push

Les notifications push utilisent le mécanisme de **localisation côté client** d'APNs :

- `loc_key` : clé de traduction (ex: `push.friend_request.body`) ;
- `title_loc_key` : clé de traduction pour le titre ;
- `loc_args` : arguments dynamiques (typiquement le nom de l'acteur).

**Pourquoi** : l'application iOS affiche la notification dans la langue du système de l'utilisateur,
sans que le backend ait besoin de connaître cette langue.

### 7.4 Nettoyage des notifications

- Quand une demande d'ami est **annulée**, la notification `friend_request` correspondante est supprimée ;
- Quand une amitié est **supprimée**, toutes les notifications liées (`friend_request`, `friend_accepted`,
  `friend_activity`) entre les deux utilisateurs sont purgées.

**Pourquoi** : éviter d'afficher des notifications obsolètes qui pourraient confondre l'utilisateur.

---

## 8. Amis

### 8.1 Flux de demande d'ami

```
    [Alice]                                    [Bob]
       |                                         |
       |--- send_request (by username) --------->|
       |                                         |
       |                        friend_request notif
       |                                         |
       |                          accept / decline
       |                                         |
       |<--- accept_request ---------------------|
       |                                         |
  friend_accepted notif                          |
       |                                         |
  [amitié créée + cercle direct créé]            |
```

### 8.2 Règles du flux

| Action | Qui | Pré-conditions | Effet |
|--------|-----|----------------|-------|
| **Envoyer une demande** | Émetteur | Pas à soi-même, pas déjà amis, pas de demande pending dans les deux sens | Crée un `friend_request` avec `status = pending`, notifie le destinataire |
| **Accepter** | Destinataire uniquement | Demande `pending`, demande dirigée vers lui | Dans une transaction : statut -> `accepted`, création friendship, création cercle direct |
| **Décliner** | Destinataire uniquement | Demande `pending` | Statut -> `declined`. **Pas de notification** à l'émetteur (pattern LinkedIn : silence poli) |
| **Annuler** | Émetteur uniquement | Demande `pending` | Statut -> `cancelled`, suppression de la notification envoyée au destinataire |
| **Re-demander** | Émetteur | Après un decline/cancel, les anciennes demandes non-pending sont nettoyées | Possibilité de renvoyer une demande après un refus |

### 8.3 Stockage canonique des amitiés

Les amitiés sont stockées dans la table `friendships` avec un invariant critique :

```
user_a_id < user_b_id  (toujours)
```

**Pourquoi** : cet ordre canonique garantit l'**unicité sans index composé bidirectionnel**. Pour vérifier
si Alice et Bob sont amis, il suffit de chercher `(LEAST(alice, bob), GREATEST(alice, bob))`.
Sans cet invariant, il faudrait deux requêtes ou un index bidirectionnel coûteux.

Le verrouillage lors de l'envoi de demande utilise `SELECT ... FOR UPDATE` sur les `friend_requests`
existantes entre les deux utilisateurs pour éviter les race conditions (deux demandes simultanées
dans des sens opposés).

### 8.4 Suppression d'amitié

La suppression est une **opération atomique** en transaction unique qui :

1. Supprime la ligne `friendships` ;
2. Supprime les `friend_requests` entre les deux utilisateurs ;
3. Supprime le cercle direct (CASCADE supprime `circle_items`, `circle_events`, `circle_members`) ;
4. Purge les notifications liées.

**Pourquoi tout en transaction** : éviter un état incohérent où l'amitié est supprimée mais le cercle
direct subsiste (avec des envies potentiellement visibles par quelqu'un qui n'est plus ami).

---

## Glossaire

| Terme Offrii | Définition |
|-------------|-----------|
| **Envie** (Item) | Un souhait personnel ajouté par un utilisateur à sa liste |
| **Besoin** (Community Wish) | Une demande d'aide publiée sur le mur d'entraide communautaire |
| **Cercle** (Circle) | Un groupe d'utilisateurs qui partagent leurs envies entre eux |
| **Cercle direct** | Un cercle automatique à 2 personnes, créé lors d'une amitié |
| **Donneur** (Donor / matched_with) | L'utilisateur qui propose son aide sur un besoin communautaire |
| **Claim** | La réservation d'une envie par un ami (il compte l'offrir) |
| **Token version** | Compteur incrémental qui invalide tous les JWT émis avant l'incrément |
| **Fire-and-forget** | Pattern où une opération non-critique (notification, email) est lancée en arrière-plan sans attendre son résultat |
| **Soft delete** | Suppression logique (champ `status = 'deleted'`) plutôt que physique, préservant l'intégrité référentielle |
