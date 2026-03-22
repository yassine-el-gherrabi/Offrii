# Regles Metier

> **Objectif de cette page** : documenter les invariants metier d'Offrii, les machines a etats, les constantes
> configurables et les garde-fous anti-abus. Chaque regle est justifiee par un *pourquoi* metier, pas
> seulement un *quoi* technique.

---

## Table des matieres

1. [Machine a etats -- Envies (Items)](#1-machine-a-etats--envies-items)
2. [Machine a etats -- Entraide (Community Wishes)](#2-machine-a-etats--entraide-community-wishes)
3. [Constantes metier](#3-constantes-metier)
4. [Cercles et partage](#4-cercles-et-partage)
5. [Moderation](#5-moderation)
6. [Anti-spam et rate limiting](#6-anti-spam-et-rate-limiting)
7. [Notifications](#7-notifications)
8. [Amis](#8-amis)

---

## 1. Machine a etats -- Envies (Items)

Les **envies** (items) representent les souhaits personnels d'un utilisateur : objets desires, idees cadeaux, etc.
Elles sont visibles uniquement par l'utilisateur et les membres des cercles avec lesquels elles sont partagees.

### 1.1 Etats

| Etat | Description | Visible dans les listes ? |
|------|-------------|---------------------------|
| `active` | Envie active, visible et reservable par les proches | Oui |
| `purchased` | Marquee comme achetee/offerte | Oui (section "achetes") |
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

     Depuis tout etat non-terminal :
          |
      soft delete
          |
          v
     +---------+
     | deleted |
     +---------+
```

### 1.3 Regles de transition

| Transition | Qui peut la declencher | Conditions | Pourquoi |
|-----------|----------------------|------------|----------|
| Creer une envie | Proprietaire | Priorite entre 1 et 3, prix >= 0, categorie valide si fournie | Borner la priorite evite les valeurs absurdes ; le prix negatif n'a pas de sens metier |
| Passer a `purchased` | Proprietaire | Envie en statut `active` | Seul le proprietaire sait si le cadeau a ete recu |
| Soft delete | Proprietaire | Envie non supprimee | Suppression logique pour preserver l'integrite referentielle (historique de cercle, evenements) |
| Batch delete | Proprietaire | Maximum 100 envies par appel | Limiter l'impact d'un appel unique sur la base de donnees |
| Claim (reserver) | Ami (via cercle) | Envie `active`, non deja reservee, pas sa propre envie | Un ami reserve un cadeau pour l'offrir -- il ne peut pas reserver le sien |
| Unclaim | Le reservataire | Envie claimee par lui | Seul celui qui a reserve peut annuler pour eviter les conflits |
| Owner unclaim (web) | Proprietaire | Uniquement si `claimed_via = "web"` | Le proprietaire peut retirer un claim web (anonyme) mais pas un claim app (l'ami a explicitement reserve) |

### 1.4 Flag `is_private`

Quand une envie est marquee `is_private = true` :

- Toutes les entrees `circle_items` associees sont **immediatement supprimees** ;
- Les regles de partage automatique (`all`, `categories`) l'**excluent** systematiquement ;
- L'envie reste visible uniquement par son proprietaire.

**Pourquoi** : un utilisateur doit pouvoir garder certains souhaits strictement prives (cadeaux surprises
pour soi-meme, envies sensibles) sans risquer une fuite via le partage automatique.

### 1.5 Enrichissement OG (Open Graph)

Lors de la creation ou de la mise a jour d'une envie avec des liens, le systeme recupere les metadonnees
Open Graph (image, titre, nom du site) de maniere **asynchrone** (fire-and-forget via `tokio::spawn`).

**Pourquoi** : ameliorer l'experience utilisateur en affichant un apercu riche du lien sans bloquer la
reponse API. Si la recuperation echoue, l'envie reste valide sans apercu.

### 1.6 Cache Redis

Les listes d'envies sont cachees dans Redis avec un systeme de **versioning** :

- Cle : `items:{user_id}:v{version}:{params}`
- TTL : **300 secondes** (5 minutes)
- Toute mutation (creation, edition, suppression, claim) **incremente la version**, ce qui invalide
  automatiquement toutes les entrees de cache precedentes.

**Pourquoi** : eviter les requetes SQL couteuses sur les listings pagines tout en garantissant la coherence
apres chaque mutation.

---

## 2. Machine a etats -- Entraide (Community Wishes)

L'**entraide** est le coeur social d'Offrii : un mur communautaire ou les utilisateurs publient des besoins
et ou d'autres proposent leur aide. Ce systeme est beaucoup plus complexe que les envies personnelles car il
implique la moderation, les signalements et l'interaction entre inconnus.

### 2.1 Etats

| Etat | Description | Visible publiquement ? |
|------|-------------|----------------------|
| `pending` | En attente de moderation IA | Non |
| `open` | Publie et visible sur le mur | Oui |
| `flagged` | Rejete par l'IA ou service indisponible, en attente de review admin | Non |
| `review` | Seuil de signalements communautaires atteint | Non |
| `matched` | Un donneur s'est propose | Oui (avec mention "en cours") |
| `fulfilled` | Le proprietaire a confirme le don | Non (archive) |
| `closed` | Ferme par le proprietaire | Non (archive) |
| `rejected` | Rejete par un administrateur | Non (archive) |

### 2.2 Diagramme de transitions

```
                      creation
                         |
                         v
                    +---------+
                    | pending |  (moderation IA asynchrone)
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
    |   | review |----+  (reopen par le proprietaire)
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

### 2.3 Regles de transition detaillees

| Transition | Declencheur | Pre-conditions | Effet | Pourquoi |
|-----------|------------|----------------|-------|----------|
| `creation` -> `pending` | Utilisateur | Compte age de 24h+, email verifie, < 3 besoins actifs, display_name requis si non-anonyme | Le besoin est cree en `pending`, moderation IA lancee en arriere-plan | Empecher le spam de nouveaux comptes ; limiter l'engorgement du mur |
| `pending` -> `open` | Systeme (IA) | Moderation IA approuve le contenu | Le besoin devient visible sur le mur | La moderation IA filtre les contenus inappropries avant publication |
| `pending` -> `flagged` | Systeme (IA) | L'IA detecte un contenu problematique OU le service est indisponible apres 3 tentatives | Le besoin est invisible, en attente d'un admin | En cas de doute (y compris panne), on prefere la prudence : un humain tranchera |
| `open` -> `matched` | Donneur | Compte age de 24h+, email verifie, pas le proprietaire, pas de signalement en cours | `matched_with` = donneur, `matched_at` = maintenant | Le donneur exprime sa volonte d'aider ; le besoin reste visible mais reserve |
| `matched` -> `open` | Donneur (`withdraw`) | Statut `matched`, c'est bien le donneur actuel | Match efface, messages supprimes | Le donneur peut se retracter ; les messages sont purges pour la confidentialite du prochain donneur |
| `matched` -> `open` | Proprietaire (`reject`) | Statut `matched`, c'est bien le proprietaire | Match efface, messages supprimes | Le proprietaire peut decliner une offre inadaptee |
| `matched` -> `fulfilled` | Proprietaire (`confirm`) | Statut `matched` | `fulfilled_at` = maintenant | Le proprietaire confirme avoir recu l'aide promise |
| `open` -> `review` | Systeme | `report_count >= 5` (seuil) | Besoin retire du mur en attendant un admin | La communaute s'automodere : si 5 personnes signalent, il y a probablement un probleme |
| `review`/`closed` -> `pending` | Proprietaire (`reopen`) | `reopen_count < 2`, cooldown de 24h respecte | Compteur de reopens incremente, signalements remis a zero, re-moderation IA | Permettre a l'auteur de corriger son besoin, mais limiter les abus de reouverture |
| `open`/`matched`/... -> `closed` | Proprietaire | Pas en etat terminal | `closed_at` = maintenant, notifie le donneur si matched | L'auteur peut toujours fermer son besoin de lui-meme |
| `flagged`/`review` -> `open` | Admin | Statut `flagged` ou `review` | Approuve manuellement | Un humain valide que le contenu est conforme |
| `flagged`/`review` -> `rejected` | Admin | Statut `flagged` ou `review` | Rejete definitivement | Le contenu enfreint les regles communautaires |
| Edition du contenu | Proprietaire | Statut `open`, `review` ou `closed` | Retour en `pending`, signalements remis a zero, re-moderation | Le contenu a change, les anciens signalements ne sont plus pertinents |
| Suppression | Proprietaire | Pas en `matched` ni `fulfilled` | Suppression physique | On ne peut pas supprimer un besoin en cours d'aide (protection du donneur) ; les besoins fulfills sont conserves pour l'historique |

### 2.4 Blocage

Un utilisateur peut **bloquer** un besoin (`wish_blocks`). Les besoins bloques sont masques de son fil
sans impact sur les autres utilisateurs. C'est une operation locale et silencieuse.

**Pourquoi** : permettre a chaque utilisateur de personnaliser son experience sans impacter la visibilite
globale.

---

## 3. Constantes metier

Toutes les constantes proviennent du code source (`community_wish_service.rs`, `item_service.rs`,
`auth_service.rs`, `upload_service.rs`).

### 3.1 Entraide

| Constante | Valeur | Justification |
|-----------|--------|---------------|
| `MAX_ACTIVE_WISHES_PER_USER` | **3** | Eviter l'engorgement du mur d'entraide. Si un utilisateur pouvait publier 50 besoins, le mur serait domine par quelques profils. 3 est un compromis entre expression et equite. |
| `MIN_ACCOUNT_AGE_HOURS` | **24h** | Barriere anti-spam : un compte cree il y a 5 minutes est probablement un bot ou un troll. 24h force une intention reelle. |
| `WISH_REPORT_THRESHOLD` | **5** | Seuil d'auto-moderation communautaire. Trop bas (2) provoquerait des faux positifs par grief ; trop haut (20) laisserait du contenu problematique visible trop longtemps. |
| `MAX_REPORTS_PER_USER_PER_DAY` | **10** | Empecher un utilisateur malveillant de signaler massivement pour provoquer la moderation de besoins legitimes. |
| `MAX_REOPEN_COUNT` | **2** | Limiter les boucles de reouverture abusives. Apres 2 tentatives, seul un admin peut intervenir. |
| `REOPEN_COOLDOWN_HOURS` | **24h** | Forcer la reflexion avant de reouvrir. Eviter les reopens impulsifs en boucle. |
| `CACHE_TTL_SECS` (wishes) | **60s** | Le mur est plus dynamique que les envies personnelles. Un TTL court (1 min) garantit la fraicheur sans surcharger la DB. |

### 3.2 Envies (Items)

| Constante | Valeur | Justification |
|-----------|--------|---------------|
| `ALLOWED_SORTS` | `created_at`, `priority`, `name` | Whitelist stricte pour empecher l'injection SQL via le parametre de tri. |
| `MAX_PAGE` | **1000** | Eviter les OFFSET enormes qui degradent les performances PostgreSQL. |
| `LIST_CACHE_TTL_SECS` | **300s** (5 min) | Les listes d'envies changent moins souvent que le mur communautaire. Un TTL plus long reduit la charge DB. |
| Priorite | **1 a 3** | 1 = faible, 2 = normal, 3 = haute. Defaut a 2. Trois niveaux suffisent pour exprimer l'urgence sans complexite inutile. |
| Batch delete max | **100** | Proteger la base de donnees contre les suppressions massives en un seul appel. |

### 3.3 Authentification

| Constante | Valeur | Justification |
|-----------|--------|---------------|
| `MAX_REFRESH_TOKENS_PER_USER` | **5** | Un utilisateur a typiquement 1-3 appareils (iPhone, iPad, Mac). 5 laisse une marge pour les navigateurs web tout en limitant la surface d'attaque. Les tokens les plus anciens sont revoques automatiquement. |
| Login rate limit | **10 tentatives / 5 min** | Standard OWASP pour le brute-force. Apres 10 echecs, le compte est temporairement bloque. |
| Password reset rate | **1/min**, **3/5min**, **10/jour** | Triple layer de protection : empecher le flood de mails (cout), le spam (UX), et le harcelement par email. |
| Reset code TTL | **30 min** | Assez long pour consulter sa boite mail, assez court pour qu'un code intercepte ne soit pas exploitable longtemps. |
| Reset code brute-force | **5 tentatives** | Un code a 6 chiffres (1M combinaisons) est casse en 200 000 tentatives en moyenne. Limiter a 5 rend le brute-force impossible. |
| Email verification cooldown | **5 min** | Eviter de surcharger le service d'envoi d'emails. |
| Dummy hash (timing attack) | `DUMMY_HASH` | Quand un email n'existe pas en base, on verifie quand meme un hash bidon pour que le temps de reponse soit identique a celui d'un "mauvais mot de passe". Empeche l'enumeration d'emails par timing. |
| Token versioning | `token_version` | Chaque changement de mot de passe ou operation sensible incremente la version. Les tokens emis avec une version anterieure sont immediatement refuses. Permet la revocation de masse sans blacklister individuellement chaque token. |

### 3.4 Upload

| Constante | Valeur | Justification |
|-----------|--------|---------------|
| `MAX_UPLOAD_BYTES` | **5 Mo** | Equilibre entre qualite d'image et bande passante mobile. Au-dela, le temps d'upload degrade l'UX sur 4G. |
| `WEBP_QUALITY` | **75** | Qualite lossy WebP. 75 offre un bon ratio qualite/taille (reduction ~70% vs JPEG a qualite equivalente). |
| `ALLOWED_TYPES` | `jpeg`, `png`, `webp`, `heic`, `heif` | Supporter les formats courants iOS (HEIC) et Android (JPEG/PNG) + WebP natif. |
| Avatar/circle resize | **400x400** | Crop carre puis reduction. 400px est suffisant pour les avatars Retina (200pt x @2x) sans gaspiller du stockage. |
| Item resize | **max 800px largeur** | Assez large pour un affichage plein ecran mobile, assez petit pour un chargement rapide. |
| EXIF orientation | Appliquee avant crop | Les photos iPhone ont souvent l'orientation en metadonnees EXIF plutot qu'en pixels. Sans correction, les images apparaissent pivotees. |

---

## 4. Cercles et partage

Les **cercles** sont le mecanisme de partage d'Offrii. Un cercle est un groupe d'utilisateurs qui voient les envies des autres membres.

### 4.1 Types de cercles

| Type | `is_direct` | Creation | Suppression | Renommage |
|------|-------------|----------|-------------|-----------|
| **Cercle direct** | `true` | Automatique a l'acceptation d'une amitie | Automatique a la suppression de l'amitie | Interdit (affiche le nom de l'ami) |
| **Cercle de groupe** | `false` | Manuelle par un utilisateur | Par le proprietaire uniquement | Par tout membre |

### 4.2 Cercles directs (max 2 participants)

Quand une amitie est acceptee, le systeme cree automatiquement un cercle direct entre les deux amis. Ce cercle :

- Ne contient que 2 membres (les deux amis) ;
- N'a pas de nom (l'interface affiche le nom de l'autre membre) ;
- Ne peut pas etre supprime manuellement (il suit le cycle de vie de l'amitie) ;
- Ne peut pas avoir de membres supplementaires.

**Pourquoi** : simplifier le partage un-a-un. L'utilisateur ajoute un ami et peut immediatement partager
ses envies avec lui, sans creer manuellement un cercle. Le cercle direct materialise la relation d'amitie
dans le systeme de partage.

### 4.3 Modes de partage (`circle_share_rules`)

Chaque membre d'un cercle definit ses propres regles de partage pour ce cercle :

| Mode | Comportement | Cas d'usage |
|------|-------------|-------------|
| `none` | Aucune envie n'est partagee avec ce cercle | "Je veux voir les envies des autres mais pas montrer les miennes" |
| `all` | Toutes les envies actives et non-privees sont automatiquement partagees | "Ma famille voit tout" |
| `categories` | Seules les envies des categories selectionnees (`category_ids`) sont partagees | "Mes collegues ne voient que ma categorie Tech" |
| `selection` | Partage explicite envie par envie via `circle_items` | "Je choisis manuellement ce que je partage" (ancien mode, retrocompatibilite) |

**Retrocompatibilite** : si un membre n'a aucune regle (`NULL`), le systeme retombe en mode `selection`
implicite basee sur les entrees `circle_items` existantes.

**Invariant critique** : les envies marquees `is_private = true` ne sont **jamais** partagees via les
regles automatiques (`all`, `categories`), meme si la regle correspond.

### 4.4 Invitations (token-based)

L'acces aux cercles de groupe se fait par invitation via un token :

| Propriete | Detail |
|-----------|--------|
| Format du token | 32 caracteres alphanumeriques aleatoires |
| Duree de validite | Configurable, defaut 24h, min 1h, max 30 jours (720h) |
| Nombre d'utilisations | Configurable, defaut 1, minimum 1 |
| Revocation | Par le createur de l'invite OU par le proprietaire du cercle |

Le join est **transactionnel** : l'increment du compteur d'utilisation, l'ajout du membre et la creation
de l'evenement se font dans une seule transaction. Si l'ajout echoue, l'invite n'est pas consommee.

**Pourquoi** : le systeme de tokens evite de devoir etre amis pour rejoindre un cercle de groupe
(cas d'usage : famille elargie, collegues). Le controle par utilisations et expiration limite
les risques de diffusion non souhaitee.

### 4.5 Evenements de cercle (audit log)

Chaque action significative dans un cercle genere un evenement dans `circle_events` :

| Type d'evenement | Declencheur | Donnees |
|-----------------|------------|---------|
| `item_shared` | Partage d'une envie | `actor_id`, `item_id` |
| `item_unshared` | Retrait d'une envie | `actor_id`, `item_id` |
| `item_claimed` | Reservation d'une envie | `actor_id`, `item_id` |
| `item_unclaimed` | Annulation de reservation | `actor_id`, `item_id` |
| `member_joined` | Nouveau membre | `actor_id` |
| `member_left` | Depart d'un membre | `actor_id` |

**Anti-spoiler** : les evenements de type `item_claimed` et `item_unclaimed` sont masques au proprietaire
de l'envie. Cela preserve la surprise du cadeau.

**Pourquoi** : le log d'evenements sert a la fois a l'affichage de la derniere activite dans la liste
des cercles et a la tracabilite pour le debugging.

### 4.6 Reservation dans un cercle

Quand un membre reserve une envie dans un cercle :

1. L'envie est marquee `claimed_by` + `claimed_via` ("app" ou "web") ;
2. Un evenement `item_claimed` est enregistre ;
3. Tous les membres du cercle (sauf le proprietaire de l'envie) sont notifies ;
4. Le cache du proprietaire est invalide.

La distinction `claimed_via` est importante :
- **"app"** : reserve explicitement par un utilisateur identifie. Seul le reservataire peut annuler.
- **"web"** : reserve via le lien de partage web (anonyme). Le proprietaire peut aussi annuler.

---

## 5. Moderation

### 5.1 Moderation IA (OpenAI)

Chaque besoin communautaire passe par une verification IA **avant** publication :

1. Le besoin est cree en statut `pending` ;
2. Un `tokio::spawn` lance la verification via `ModerationService.check_wish()` ;
3. Le service envoie le titre, la description, la categorie, l'image et les liens a l'API OpenAI ;
4. **3 tentatives** avec backoff exponentiel (2s, 4s) en cas d'echec ;
5. Resultat :
   - `Approved` -> statut `open`, notification "Besoin publie !"
   - `Flagged { reason }` -> statut `flagged`, notification "Besoin en revision"
   - Echec apres 3 tentatives -> statut `flagged` (failsafe)

**Pourquoi le failsafe flagge plutot qu'approuve** : en cas de panne du service de moderation, il vaut
mieux retarder la publication (un admin validera manuellement) que de laisser passer du contenu
potentiellement dangereux.

### 5.2 Signalements communautaires

| Regle | Detail | Pourquoi |
|-------|--------|----------|
| Qui peut signaler | Tout utilisateur avec email verifie et compte age de 24h+ | Memes barrieres que pour la creation, pour eviter les signalements de spam-bots |
| Pas son propre besoin | Le proprietaire ne peut pas signaler son besoin | Incoherent et potentiellement abusif |
| Seuls les besoins `open` | On ne peut pas signaler un besoin deja en review ou ferme | Inutile et pourrait corrompre les compteurs |
| Limite journaliere | 10 signalements par utilisateur par jour | Empecher le "report bombing" (spam de signalements contre des besoins legitimes) |
| Un seul signalement par besoin | Un utilisateur ne peut signaler le meme besoin qu'une fois | Eviter le gonflement artificiel du compteur |
| Seuil automatique | 5 signalements -> passage en `review` | La communaute retire le contenu du mur ; un admin tranchera |

### 5.3 File d'attente admin

Les administrateurs disposent d'un endpoint dedie (`admin_list_flagged`) qui liste les besoins en statut
`flagged` et `review`, avec :

- Le contenu complet (titre, description, categorie, image, liens) ;
- La note de moderation IA (`moderation_note`) ;
- Le nombre de signalements ;
- L'identifiant du proprietaire.

Actions possibles :
- **Approuver** : passage a `open` + notification au proprietaire ;
- **Rejeter** : passage a `rejected` + notification au proprietaire.

### 5.4 Re-moderation a l'edition

Quand un proprietaire modifie le contenu d'un besoin :

1. Les signalements sont **remis a zero** (le contenu a change, les anciens signalements ne sont plus pertinents) ;
2. La `moderation_note` est effacee ;
3. Le statut passe a `pending` ;
4. Une nouvelle verification IA est lancee.

**Pourquoi** : un besoin initialement approuve pourrait devenir problematique apres edition. Il faut
re-verifier systematiquement.

---

## 6. Anti-spam et rate limiting

### 6.1 Rate limits par service

| Endpoint / Action | Limite | Fenetre | Stockage | Pourquoi |
|------------------|--------|---------|----------|----------|
| Login | 10 tentatives | 5 min | Redis INCR + EXPIRE | Protection brute-force standard OWASP |
| Password reset request | 1 / email | 60s | Redis SET NX | Empecher le flood de mails |
| Password reset resend | 3 / email | 5 min | Redis INCR | Limite supplementaire anti-abus |
| Password reset resend (jour) | 10 / email | 24h | Redis INCR | Plafond journalier pour empecher le harcelement par email |
| Reset code verification | 5 tentatives / email | 30 min | Redis INCR | Rendre le brute-force du code 6 chiffres impossible |
| Email verification resend | 1 | 5 min | DB (token `created_at`) | Eviter la surcharge du service d'envoi |
| Email change request | 1 | 5 min | DB (token `created_at`) | Idem |
| Signalements communautaires | 10 / utilisateur | 24h | DB (requete COUNT) | Empecher le report bombing |

### 6.2 Barrieres d'entree pour les actions communautaires

Certaines actions sur le mur d'entraide exigent des pre-requis :

| Action | Email verifie | Compte age de 24h+ |
|--------|:------------:|:------------------:|
| Creer un besoin | Oui | Oui |
| Proposer son aide (`offer`) | Oui | Oui |
| Signaler un besoin | Oui | Oui |
| Consulter le mur | Non | Non |

**Pourquoi la double barriere** : l'email verifie prouve que l'adresse est reelle (pas de spam) ; l'age
du compte empeche les bots de creer et agir dans la meme minute.

### 6.3 Recherche utilisateurs

La recherche d'amis par username est bornee :

- Requete vide ou > 50 caracteres -> resultat vide (pas d'erreur) ;
- Matching par prefixe (`ILIKE 'query%'`) ;
- Limite a 10 resultats ;
- Exclut l'appelant des resultats.

**Pourquoi** : eviter les requetes LIKE couteuses sur des patterns longs et limiter l'enumeration d'utilisateurs.

---

## 7. Notifications

### 7.1 Architecture

Les notifications suivent un pattern **fire-and-forget** : l'action metier (creation d'ami, reservation
d'envie, etc.) s'execute de maniere synchrone, puis la notification est envoyee via `tokio::spawn`
sans bloquer la reponse.

Deux canaux de livraison :

| Canal | Methode | Persistance |
|-------|---------|-------------|
| **In-app** | Insert dans la table `notifications` | Oui (consultable dans le centre de notifications) |
| **Push (APNs)** | Envoi via le service de notification, batch par appareil | Non (ephemere) |

### 7.2 Types de notifications

| Type | Declencheur | Destinataire |
|------|------------|-------------|
| `friend_request` | Envoi d'une demande d'ami | Destinataire de la demande |
| `friend_accepted` | Acceptation d'une demande | Emetteur de la demande |
| `item_shared` | Partage d'une envie dans un cercle | Tous les membres du cercle (sauf l'auteur) |
| `item_claimed` | Reservation d'une envie | Membres du cercle (sauf le proprietaire de l'envie -- anti-spoiler) |
| `item_unclaimed` | Annulation de reservation | Membres du cercle (sauf le proprietaire de l'envie) |
| `circle_member_joined` | Arrivee dans un cercle via invite | Tous les membres (sauf le nouveau) |
| `wish_moderation_approved` | Besoin approuve par l'IA | Proprietaire du besoin |
| `wish_moderation_flagged` | Besoin flagge par l'IA | Proprietaire du besoin |
| `wish_offer` | Un donneur propose son aide | Proprietaire du besoin |
| `wish_offer_withdrawn` | Le donneur retire son offre | Proprietaire du besoin |
| `wish_offer_rejected` | Le proprietaire decline l'offre | Donneur |
| `wish_confirmed` | Le proprietaire confirme le don | Donneur |
| `wish_closed` | Le proprietaire ferme un besoin matched | Donneur |
| `wish_reported` | Seuil de signalements atteint | Proprietaire du besoin |
| `wish_approved` | Admin approuve un besoin flagge/review | Proprietaire du besoin |
| `wish_rejected` | Admin rejette un besoin | Proprietaire du besoin |

### 7.3 Localisation des push

Les notifications push utilisent le mecanisme de **localisation cote client** d'APNs :

- `loc_key` : cle de traduction (ex: `push.friend_request.body`) ;
- `title_loc_key` : cle de traduction pour le titre ;
- `loc_args` : arguments dynamiques (typiquement le nom de l'acteur).

**Pourquoi** : l'application iOS affiche la notification dans la langue du systeme de l'utilisateur,
sans que le backend ait besoin de connaitre cette langue.

### 7.4 Nettoyage des notifications

- Quand une demande d'ami est **annulee**, la notification `friend_request` correspondante est supprimee ;
- Quand une amitie est **supprimee**, toutes les notifications liees (`friend_request`, `friend_accepted`,
  `friend_activity`) entre les deux utilisateurs sont purgees.

**Pourquoi** : eviter d'afficher des notifications obsoletes qui pourraient confondre l'utilisateur.

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
  [amitie creee + cercle direct cree]            |
```

### 8.2 Regles du flux

| Action | Qui | Pre-conditions | Effet |
|--------|-----|----------------|-------|
| **Envoyer une demande** | Emetteur | Pas a soi-meme, pas deja amis, pas de demande pending dans les deux sens | Cree un `friend_request` avec `status = pending`, notifie le destinataire |
| **Accepter** | Destinataire uniquement | Demande `pending`, demande dirigee vers lui | Dans une transaction : statut -> `accepted`, creation friendship, creation cercle direct |
| **Decliner** | Destinataire uniquement | Demande `pending` | Statut -> `declined`. **Pas de notification** a l'emetteur (pattern LinkedIn : silence poli) |
| **Annuler** | Emetteur uniquement | Demande `pending` | Statut -> `cancelled`, suppression de la notification envoyee au destinataire |
| **Re-demander** | Emetteur | Apres un decline/cancel, les anciennes demandes non-pending sont nettoyees | Possibilite de renvoyer une demande apres un refus |

### 8.3 Stockage canonique des amities

Les amities sont stockees dans la table `friendships` avec un invariant critique :

```
user_a_id < user_b_id  (toujours)
```

**Pourquoi** : cet ordre canonique garantit l'**unicite sans index compose bidirectionnel**. Pour verifier
si Alice et Bob sont amis, il suffit de chercher `(LEAST(alice, bob), GREATEST(alice, bob))`.
Sans cet invariant, il faudrait deux requetes ou un index bidirectionnel couteux.

Le verrouillage lors de l'envoi de demande utilise `SELECT ... FOR UPDATE` sur les `friend_requests`
existantes entre les deux utilisateurs pour eviter les race conditions (deux demandes simultanees
dans des sens opposes).

### 8.4 Suppression d'amitie

La suppression est une **operation atomique** en transaction unique qui :

1. Supprime la ligne `friendships` ;
2. Supprime les `friend_requests` entre les deux utilisateurs ;
3. Supprime le cercle direct (CASCADE supprime `circle_items`, `circle_events`, `circle_members`) ;
4. Purge les notifications liees.

**Pourquoi tout en transaction** : eviter un etat incoherent ou l'amitie est supprimee mais le cercle
direct subsiste (avec des envies potentiellement visibles par quelqu'un qui n'est plus ami).

---

## Glossaire

| Terme Offrii | Definition |
|-------------|-----------|
| **Envie** (Item) | Un souhait personnel ajoute par un utilisateur a sa liste |
| **Besoin** (Community Wish) | Une demande d'aide publiee sur le mur d'entraide communautaire |
| **Cercle** (Circle) | Un groupe d'utilisateurs qui partagent leurs envies entre eux |
| **Cercle direct** | Un cercle automatique a 2 personnes, cree lors d'une amitie |
| **Donneur** (Donor / matched_with) | L'utilisateur qui propose son aide sur un besoin communautaire |
| **Claim** | La reservation d'une envie par un ami (il compte l'offrir) |
| **Token version** | Compteur incremental qui invalide tous les JWT emis avant l'increment |
| **Fire-and-forget** | Pattern ou une operation non-critique (notification, email) est lancee en arriere-plan sans attendre son resultat |
| **Soft delete** | Suppression logique (champ `status = 'deleted'`) plutot que physique, preservant l'integrite referentielle |
