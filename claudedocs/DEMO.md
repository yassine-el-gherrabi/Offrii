# Guide de Test Offrii — Fixtures Complètes

**Mot de passe universel** : `DemoPass123x`

---

## Table des Comptes

| Email | @username | Rôle | Items | Amis | Cercles | Entraide |
|-------|-----------|------|-------|------|---------|----------|
| `admin@demo.com` | @admin | Admin | 0 | 0 | 0 | 0 |
| `yassine@demo.com` | @yassine | Power user | 8 (6+2p) | 5 + 2 pending | 4 | 2 wishes, 1 offer |
| `marie@demo.com` | @marie_d | Social active | 3 | 4 + 1 pending | 3 | 1 wish |
| `karim@demo.com` | @karim_b | Donneur actif | 2 | 4 | 2 | 4 offers |
| `sophie@demo.com` | @sophie_m | Wishes variés | 3 | 3 | 2 | 2 wishes |
| `lucas@demo.com` | @lucas_r | Pending friend | 2 | 0 + 1 pending | 0 | 0 |
| `emma@demo.com` | @emma_l | Semi-complet | 2 | 0 + 1 pending | 0 | 0 |
| `ahmed@demo.com` | @ahmed_t | Demandeur Entraide | 0 | 0 + 1 pending | 0 | 5 wishes |
| `chloe@demo.com` | @chloe_p | Poster anonyme | 1 | 0 + 1 pending | 0 | 3 wishes |
| `omar@demo.com` | @omar_f | Pagination test | 22 | 2 + 1 pending | 1 | 0 |
| `leila@demo.com` | @leila_s | Catégories custom | 4 | 2 | 1 | 1 wish, 1 offer |
| `newuser@demo.com` | @nouveau | Cold start | 0 | 0 | 0 | 0 |

---

## 1. Onboarding & Authentification

### 1.1 Cold Start (premier lancement)
**Compte** : `newuser@demo.com`

| # | Action | Vérifier |
|---|--------|----------|
| 1 | Lancer l'app (déconnecté) | Splash screen affiché |
| 2 | Passer les écrans onboarding | Swipe entre les pages, bouton CTA visible |
| 3 | Taper "Se connecter" | Écran login affiché |
| 4 | Se connecter avec `newuser@demo.com` | Login réussi, redirection dashboard |
| 5 | Vérifier le dashboard | **Empty state** : aucun item, aucun cercle, aucun ami |
| 6 | Vérifier ProfileProgressCard | Complétion ~17%, suggestions d'actions visibles |
| 7 | Vérifier QuickStartSection | Boutons "Ajouter un voeu", "Trouver des amis", etc. |

### 1.2 Login utilisateur existant
**Compte** : `yassine@demo.com`

| # | Action | Vérifier |
|---|--------|----------|
| 1 | Se connecter | Login réussi |
| 2 | Dashboard | Items visibles, cercles visibles, tout chargé |
| 3 | Pas de ProfileProgressCard | Profil ~100% → pas de barre de progression |

### 1.3 Login admin
**Compte** : `admin@demo.com`

| # | Action | Vérifier |
|---|--------|----------|
| 1 | Se connecter | Login réussi |
| 2 | Dashboard | Empty state (0 items, 0 cercles) |
| 3 | Accès modération | Section admin visible (si implémenté dans l'UI) |

---

## 2. Wishlist (Envies)

### 2.1 Liste complète avec toutes catégories
**Compte** : `yassine@demo.com` — 8 items (6 actifs + 2 purchased)

| Item | Catégorie | Priorité | Prix | URL | Statut |
|------|-----------|----------|------|-----|--------|
| MacBook Pro M4 | Tech | 3 (haute) | 2999€ | apple.com | actif |
| AirPods Pro 3 | Tech | 2 (moyenne) | 279€ | — | **Réservé** (par Marie) |
| Nike Air Max 90 | Mode | 2 | 149€ | nike.com | actif |
| Lampe de bureau LED | Maison | 1 (basse) | 45€ | — | actif |
| Zelda Echoes of Wisdom | Loisirs | 3 | 59.99€ | — | actif |
| Coffret vitamines | Santé | 1 | 35€ | — | actif |
| Pull en cachemire | Mode | 2 | 189€ | — | **purchased** |
| Enceinte Marshall Stanmore III | Tech | 3 | 349€ | marshall.com | **purchased** |

**Vérifications** :
- [ ] Les 6 catégories sont représentées (Tech, Mode, Maison, Loisirs, Santé)
- [ ] Les 3 niveaux de priorité sont visibles
- [ ] Les AirPods affichent "Réservé" (mais PAS qui a réservé — anti-spoiler)
- [ ] Les 2 items purchased sont dans un onglet/section "Achetés" ou masqués
- [ ] Les URLs s'ouvrent dans le navigateur
- [ ] Les prix s'affichent correctement

### 2.2 Pagination (22 items)
**Compte** : `omar@demo.com`

| # | Action | Vérifier |
|---|--------|----------|
| 1 | Ouvrir la wishlist | 20 premiers items affichés (page 1) |
| 2 | Scroller en bas | Chargement page 2 (2 items restants) |
| 3 | Vérifier la variété | 6 catégories différentes visibles |

**Edge cases dans les items d'Omar** :
- [ ] **Sans description** : "Hub USB-C Anker 7-en-1", "Bougie Diptyque Baies", "Carte cadeau Amazon 50€" → vérifier que l'absence de description ne casse pas l'UI
- [ ] **Sans prix** : "Ceinture cuir artisanale", "Puzzle 1000 pièces Van Gogh" → vérifier que l'absence de prix s'affiche correctement
- [ ] **URL longue** : "Abonnement Spotify Premium" → URL tronquée correctement

### 2.3 Catégories custom
**Compte** : `leila@demo.com` — 4 items

| Item | Catégorie | Type |
|------|-----------|------|
| Kit broderie traditionnelle | **Artisanat** | Custom (icon: scissors) |
| Moule tajine terre cuite | **Cuisine** | Custom (icon: utensils) |
| Robe kaftan été | Mode | Default |
| Diffuseur huiles essentielles | Santé | Default |

**Vérifications** :
- [ ] Les catégories custom "Artisanat" et "Cuisine" apparaissent
- [ ] Les icônes custom s'affichent (scissors, utensils)
- [ ] Le filtre par catégorie fonctionne avec les custom

### 2.4 Wishlist vide
**Compte** : `newuser@demo.com`
- [ ] Vérifier l'empty state de la wishlist
- [ ] CTA "Ajouter mon premier voeu" visible

---

## 3. Amis

### 3.1 Liste d'amis acceptés
**Compte** : `yassine@demo.com` — 5 amis

| Ami | @username |
|-----|-----------|
| Marie Dupont | @marie_d |
| Karim Benali | @karim_b |
| Sophie Martin | @sophie_m |
| Omar Farouq | @omar_f |
| *(+ 1 via Omar↔Yassine)* | |

**Vérifications** :
- [ ] La liste affiche 5 amis avec avatars/noms
- [ ] Taper sur un ami ouvre son profil
- [ ] On peut voir la wishlist d'un ami (items partagés dans un cercle commun)

### 3.2 Demandes en attente (incoming)
**Compte** : `yassine@demo.com`

| De | Statut |
|----|--------|
| Lucas Robert (@lucas_r) | **Reçue** — boutons Accepter / Refuser visibles |

**Vérifications** :
- [ ] Badge notification sur l'onglet Amis (ou section demandes)
- [ ] La demande de Lucas est visible dans "Demandes reçues"
- [ ] Boutons Accepter / Refuser fonctionnels

### 3.3 Demandes envoyées (outgoing)
**Compte** : `yassine@demo.com`

| Vers | Statut |
|------|--------|
| Emma Laurent (@emma_l) | **Envoyée** — en attente |

**Vérifications** :
- [ ] La demande envoyée à Emma est visible dans "Demandes envoyées"
- [ ] Bouton Annuler (si implémenté)

### 3.4 Autres pending à vérifier

| Compte | Demande reçue de |
|--------|-----------------|
| `marie@demo.com` | Ahmed (@ahmed_t) |
| `omar@demo.com` | Chloé (@chloe_p) |

### 3.5 Recherche d'amis
- [ ] Chercher "omar_f" → Omar Farouq trouvé
- [ ] Chercher "leila_s" → Leila Saadi trouvée
- [ ] Chercher "nouveau" → @nouveau trouvé (mais déjà aucun lien)

### 3.6 Utilisateur sans amis
**Compte** : `newuser@demo.com`
- [ ] Empty state "Aucun ami" affiché
- [ ] CTA "Trouver des amis" visible

---

## 4. Cercles

### 4.1 Vue multi-cercles
**Compte** : `yassine@demo.com` — 4 cercles

| Cercle | Owner | Membres | Items partagés |
|--------|-------|---------|---------------|
| **Famille** | Yassine | 4 (Yassine, Marie, Karim, Sophie) | 8 items |
| **Collègues** | Marie | 3 (Marie, Yassine, Karim) | 2 items |
| **Noël 2026** | Omar | 3 (Omar, Yassine, Marie) | 12 items |
| **Direct Marie** | Yassine | 2 (Yassine, Marie) | 0 items |

**Vérifications** :
- [ ] 4 cercles affichés dans la liste
- [ ] Chaque cercle montre le nombre de membres et d'items
- [ ] Le cercle direct affiche "Marie Dupont" (pas "Direct")

### 4.2 Cercle "Famille" — items partagés & claims
**Compte** : `yassine@demo.com` → ouvrir Famille

**Items visibles** (8 total) :

| Item | Propriétaire | Claim |
|------|-------------|-------|
| MacBook Pro M4 | Yassine | — |
| AirPods Pro 3 | Yassine | **Réservé par Marie** (visible pour Karim/Sophie, PAS pour Yassine) |
| Zelda Echoes of Wisdom | Yassine | — |
| Lampe de bureau LED | Yassine | — |
| Livre de cuisine japonaise | Sophie | — |
| Écharpe en soie | Sophie | **Réservé par Yassine** (visible pour Marie/Karim, PAS pour Sophie) |
| Cours de yoga | Marie | — |
| Sac à dos Fjällräven | Marie | **Réservé par Karim** (visible pour Yassine/Sophie, PAS pour Marie) |

**Vérifications anti-spoiler** :
- [ ] **Login Yassine** → ses propres items montrent "Réservé" sur AirPods, mais PAS le nom de Marie
- [ ] **Login Marie** → voit les items de Yassine, AirPods NON marqué "Réservé" (c'est elle qui a claim), Sac montre "Réservé" mais pas par qui
- [ ] **Login Karim** → voit "Réservé" sur AirPods (par Marie) et Écharpe (par Yassine), mais le Sac NON marqué (c'est lui)

### 4.3 Cercle "Noël 2026" — items d'Omar
**Compte** : `omar@demo.com` ou `yassine@demo.com`

| # | Vérification |
|---|-------------|
| 1 | 12 items d'Omar partagés dans le cercle |
| 2 | Yassine a claim l'iPhone 15 Pro Max |
| 3 | Marie a claim la Machine Nespresso |
| 4 | Omar voit "Réservé" sur iPhone et Nespresso (pas les noms) |
| 5 | Invite link actif visible (owner Omar uniquement) |

### 4.4 Cercle "Projet Asso" — cercle vide
**Compte** : `leila@demo.com`

- [ ] Le cercle existe avec Sophie comme membre
- [ ] 0 items partagés → empty state "Aucun item" affiché
- [ ] CTA pour partager un item visible

### 4.5 Cercle direct
**Compte** : `yassine@demo.com`

- [ ] Le cercle direct avec Marie apparaît dans la liste
- [ ] Affiche le nom "Marie Dupont" (pas "Direct")
- [ ] 0 items partagés actuellement

### 4.6 Invite link
**Compte** : `omar@demo.com`

- [ ] Ouvrir cercle "Noël 2026"
- [ ] Section invitations visible (owner uniquement)
- [ ] Lien d'invitation actif (max 5 uses, expire dans ~72h)
- [ ] Copier le lien → ouvrir dans navigateur → page de preview

### 4.7 Utilisateur sans cercle
**Compte** : `newuser@demo.com`
- [ ] Empty state "Aucun cercle"
- [ ] CTA "Créer un cercle" visible

---

## 5. Entraide (Community Wishes)

### 5.1 Browse — wishes ouvertes visibles par tous

**Compte** : n'importe lequel (ou déconnecté si supporté)

| Wish | Catégorie | Propriétaire | Particularité |
|------|-----------|-------------|---------------|
| Manuels scolaires CP-CE1 | education | Ahmed | Classique, offrable |
| Vêtements d'hiver taille M | clothing | Marie | Classique |
| Manteaux chauds pour enfants | clothing | Yassine | Association |
| Médicaments courants | health | Leila | **Avec image + 2 liens** |
| Petit meuble de rangement | home | **Anonyme** | Poster anonyme + 1 report |
| Aide ménage ponctuelle | home | Sophie | Offre rejetée → retour open |

**Vérifications** :
- [ ] 6 wishes open visibles dans le browse
- [ ] Filtrer par catégorie fonctionne
- [ ] Le wish anonyme n'affiche PAS le nom de Chloé
- [ ] Le wish de Leila affiche l'image et les liens
- [ ] Chaque wish a un bouton "Proposer de l'aide"

### 5.2 Mes wishes — vue owner
**Compte** : `ahmed@demo.com` — 5 wishes

| Wish | Statut | Détail |
|------|--------|--------|
| Manuels scolaires CP-CE1 | **open** | Visible, attend une offre |
| Tapis de prière | **closed** | Fermé manuellement |
| Fauteuil roulant temporaire | **matched** | Yassine a proposé, 3 messages échangés |
| Spa gratuit illimité | **rejected** | 3 reports, rejeté par admin |
| Livres scolaires 6ème | **flagged** | report_count=5, en attente de review admin |

**Vérifications** :
- [ ] Les 5 statuts différents sont visuellement distinguables
- [ ] Le wish "matched" montre la conversation (3 messages)
- [ ] Le wish "rejected" affiche un message d'explication
- [ ] Le wish "flagged" affiche un indicateur de modération

### 5.3 Wish matched — conversation
**Compte** : `ahmed@demo.com` → ouvrir "Fauteuil roulant temporaire"

| Message | De | Contenu |
|---------|-----|---------|
| 1 | Yassine | "J'ai un fauteuil roulant dans mon garage..." |
| 2 | Ahmed | "Merci infiniment ! On peut se retrouver où ?" |
| 3 | Yassine | "Je suis sur Paris 15e..." |

**Vérifications** :
- [ ] Les 3 messages sont affichés dans l'ordre
- [ ] Le champ de saisie permet d'écrire un nouveau message
- [ ] Boutons "Confirmer réception" et "Annuler" visibles

### 5.4 Wish fulfilled
**Compte** : `chloe@demo.com` → ouvrir "Jouets pour enfants 3-5 ans"

- [ ] Statut **fulfilled** visible
- [ ] Historique : Karim a proposé → 2 messages → Chloé a confirmé
- [ ] Messages de Karim ("J'ai plein de Duplo...") et Chloé visibles

### 5.5 Wish anonyme
**Compte** : `chloe@demo.com` → ouvrir "Petit meuble de rangement"

- [ ] Le wish est affiché comme **anonyme** dans le browse
- [ ] Chloé (owner) voit qu'il est anonyme
- [ ] 1 signalement (report) visible (si l'UI le montre)

### 5.6 Wish avec offre rejetée
**Compte** : `sophie@demo.com` → ouvrir "Aide ménage ponctuelle"

- [ ] Statut **open** (l'offre de Leila a été rejetée)
- [ ] Le wish est de nouveau visible dans le browse
- [ ] Sophie peut recevoir de nouvelles offres

### 5.7 Wish reopened (vélo enfant)
**Compte** : `chloe@demo.com` → ouvrir "Vélo enfant 14 pouces"

- [ ] Statut actuel : **fulfilled** (le reopen a peut-être échoué — vérifier le comportement)
- [ ] Si reopen a fonctionné : statut **open** avec indicateur "réouvert"

### 5.8 Donneur actif — vue des offres faites
**Compte** : `karim@demo.com`

Karim a fait **4 offres** d'aide :
| Wish | Owner | Statut résultant |
|------|-------|-----------------|
| Fauteuil roulant | Ahmed | matched (2 msgs Karim) |
| Jouets enfants | Chloé | fulfilled (confirmé) |
| Aide déménagement | Sophie | matched (1 msg) |
| Fournitures scolaires | Yassine | matched (2 msgs) |

- [ ] Karim voit ses offres dans "Mes offres" ou un onglet dédié
- [ ] Les conversations sont accessibles

---

## 6. Modération Admin

### 6.1 Wish rejeté
**Compte** : `admin@demo.com`

- [ ] Accéder à la section modération
- [ ] Le wish "Spa gratuit illimité" (Ahmed) est visible comme **rejected**
- [ ] Note de modération : "Rejeté : contenu non conforme aux valeurs d'entraide"
- [ ] 3 reports visibles (spam par Yassine, inappropriate par Marie, scam par Sophie)

### 6.2 Wish flagged
**Compte** : `admin@demo.com`

- [ ] Le wish "Livres scolaires 6ème" (Ahmed) est en **flagged**
- [ ] report_count = 5
- [ ] Boutons "Approuver" et "Rejeter" disponibles

### 6.3 Wish avec report
Le wish "Petit meuble de rangement" (Chloé, anonyme) a 1 report (spam par Yassine).
- [ ] Ce wish apparaît dans les signalements (si l'UI admin les liste)

---

## 7. Share Links (Liens de partage)

### 7.1 Lien Yassine
**URL** : `GET /shared/demo-yassine-share`

| # | Vérification |
|---|-------------|
| 1 | Ouvrir le lien dans un navigateur (ou via curl) |
| 2 | La wishlist de Yassine s'affiche (6 items actifs) |
| 3 | Pas besoin d'être connecté pour voir |
| 4 | Les items purchased NE sont PAS visibles |
| 5 | Les prix et descriptions sont visibles |

### 7.2 Lien Omar
**URL** : `GET /shared/demo-omar-share`

| # | Vérification |
|---|-------------|
| 1 | Les 22 items d'Omar s'affichent |
| 2 | La pagination fonctionne aussi en mode share |
| 3 | Edge cases (sans prix, sans description) s'affichent correctement |

### 7.3 Test claim via share link
- [ ] Ouvrir `demo-yassine-share` → tenter de claim un item (si permission view_and_claim)
- [ ] Vérifier que le claim fonctionne sans être ami

---

## 8. Profils & Préférences

### 8.1 Profil complet (~100%)
**Compte** : `yassine@demo.com`

| Champ | Valeur |
|-------|--------|
| Display name | Yassine |
| Username | @yassine |
| Reminder freq | weekly |
| Timezone | Europe/Paris |
| Locale | fr |

- [ ] Pas de ProfileProgressCard (profil complet)
- [ ] Paramètres de rappel visibles et modifiables

### 8.2 Profil minimal (~17%)
**Compte** : `newuser@demo.com`

| Champ | Valeur |
|-------|--------|
| Display name | Nouveau |
| Username | @nouveau |
| Items | 0 |
| Amis | 0 |
| Cercles | 0 |

- [ ] ProfileProgressCard visible avec barre de progression
- [ ] Suggestions : "Ajouter une envie", "Trouver des amis", etc.

### 8.3 Profil semi-complet (~50%)
**Compte** : `emma@demo.com`

- [ ] 2 items, 0 amis, 0 cercles, 1 demande reçue (de Yassine)
- [ ] ProfileProgressCard avec progression partielle

### 8.4 Préférences de rappel variées

| Compte | Fréquence | Timezone |
|--------|-----------|----------|
| `marie@demo.com` | **daily** | Europe/Paris |
| `leila@demo.com` | **daily** | Europe/Paris |
| `yassine@demo.com` | weekly | Europe/Paris |
| `karim@demo.com` | weekly | Europe/Paris |
| `ahmed@demo.com` | weekly | **Africa/Casablanca** |
| `omar@demo.com` | **monthly** | Europe/Paris |
| Tous les autres | weekly (défaut) | UTC |

---

## 9. Vérifications API (curl)

```bash
# Login
curl -s -X POST http://localhost:3000/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"yassine@demo.com","password":"DemoPass123x"}' | python3 -m json.tool

# Items Yassine (8 items)
TOKEN="<token_yassine>"
curl -s http://localhost:3000/items -H "Authorization: Bearer $TOKEN" | python3 -c "import sys,json; items=json.load(sys.stdin); print(f'{len(items)} items')"

# Items Omar pagination
TOKEN="<token_omar>"
curl -s "http://localhost:3000/items?page=1&limit=20" -H "Authorization: Bearer $TOKEN" | python3 -c "import sys,json; d=json.load(sys.stdin); print(f'Page 1: {len(d)} items')"
curl -s "http://localhost:3000/items?page=2&limit=20" -H "Authorization: Bearer $TOKEN" | python3 -c "import sys,json; d=json.load(sys.stdin); print(f'Page 2: {len(d)} items')"

# Cercles Yassine (4 cercles)
curl -s http://localhost:3000/circles -H "Authorization: Bearer $TOKEN" | python3 -c "import sys,json; c=json.load(sys.stdin); print(f'{len(c)} circles')"

# Amis Yassine
curl -s http://localhost:3000/me/friends -H "Authorization: Bearer $TOKEN" | python3 -c "import sys,json; f=json.load(sys.stdin); print(f'{len(f)} friends')"

# Friend requests Yassine (1 pending)
curl -s http://localhost:3000/me/friend-requests -H "Authorization: Bearer $TOKEN" | python3 -m json.tool

# Community wishes browse
curl -s http://localhost:3000/community/wishes | python3 -c "import sys,json; w=json.load(sys.stdin); print(f'{len(w.get(\"data\",w))} wishes')"

# Share link public
curl -s http://localhost:3000/shared/demo-yassine-share | python3 -m json.tool

# Admin: login + check pending
TOKEN_ADMIN="<token_admin>"
curl -s http://localhost:3000/admin/wishes/pending -H "Authorization: Bearer $TOKEN_ADMIN" | python3 -m json.tool
```

---

## 10. Checklist Globale

### Écrans & Navigation
- [ ] Splash → Onboarding → Login/Register
- [ ] TabBar : Accueil / Envies / Cercles / Entraide / Profil
- [ ] Chaque tab charge correctement ses données
- [ ] Pull-to-refresh fonctionne partout

### États vides
- [ ] Wishlist vide (`newuser`)
- [ ] Liste d'amis vide (`newuser`)
- [ ] Cercles vide (`newuser`)
- [ ] Cercle sans items (`leila` → Projet Asso)

### Edge cases
- [ ] Item sans description (Omar: Hub USB-C, Bougie Diptyque, Carte Amazon)
- [ ] Item sans prix (Omar: Ceinture cuir, Puzzle Van Gogh)
- [ ] Item sans URL (multiples)
- [ ] Wish anonyme (Chloé: Petit meuble)
- [ ] Wish avec image + liens (Leila: Médicaments)
- [ ] 22 items → pagination
- [ ] Catégories custom (Leila: Artisanat, Cuisine)

### Anti-spoiler (claims)
- [ ] L'owner d'un item voit "Réservé" mais PAS le nom du claimeur
- [ ] Le claimeur NE voit PAS "Réservé" sur l'item qu'il a lui-même claim
- [ ] Les autres membres du cercle voient "Réservé par [Nom]"

### Données réalistes
- [ ] Les dates sont étalées (items sur 14j, wishes sur 5j, messages sur 2j)
- [ ] Les noms/descriptions sont en français
- [ ] Les prix sont réalistes
