# Entraide — Refonte complète

## Phase 1 : Corrections backend (gaps identifiés) ✅

- ✅ Re-modération sur update
- ✅ Intégration notification center
- ✅ Update + reopen contourne les signalements
- ✅ État orphelin si donneur supprime son compte
- ✅ Messages supprimés en cascade → SET NULL
- ⏭️ Paginer list_my_wishes (déprioritisé)
- ✅ Validation URL
- ✅ Dead code cleanup

## Phase 2 : Refonte frontend from scratch ✅

- ✅ Réécriture 19 fichiers → 12 nouveaux
- ✅ Layout aligné Envies/Proches (navigationTitle, searchable, chips, stats bar)
- ✅ Cards full-width text-first (icône catégorie + titre + description + auteur + photo optionnelle)
- ✅ Couleurs catégories vibrantes alignées sur Envies
- ✅ glassBadge style unifié
- ✅ Searchable natif sur les 3 pages (Envies, Proches, Entraide)
- ✅ Sous-titre italique "Des gestes simples, des impacts réels"

## Phase 3 : Bugs critiques P0 ✅

### 3.1 Sort → SortMenuView composant partagé ✅
**Problème** : `sortOrder` est un `@State` local jamais envoyé au backend.
**Fix** : Retirer le menu de tri. Toujours afficher par date décroissante (le backend le fait déjà). Un tri "Ancien d'abord" n'a pas de sens pour un mur d'entraide.
**Fichiers** : `EntraideView.swift`

### 3.2 ✅
**Problème** : `viewModel.myOffers` = `wishes.filter(\.isMatchedByMe)` — si on pagine ou filtre par catégorie, les offres disparaissent. Ne montre pas les fulfilled/closed.
**Fix** : Appel API dédié. Le backend `GET /community/wishes` avec auth retourne `is_matched_by_me`. On peut soit :
- Ajouter un query param `matched_by_me=true` côté backend
- Ou charger séparément dans un ViewModel dédié
**Fichiers** : Backend `handlers/community_wishes.rs` + `EntraideMyOffersContent.swift` + nouveau ViewModel ou param

### 3.3 ✅
**Problème** : Tap = match instant. Action irréversible sans filet.
**Fix** : Alert de confirmation : "Vous allez proposer votre aide à [auteur]. Vous pourrez échanger par messages." Cancel/Confirmer.
**Fichiers** : `WishDetailSheet.swift`

### 3.4 ✅
**Problème** : Si quelqu'un offre en même temps, erreur générique.
**Fix** : Mapper le 409 → "Quelqu'un a déjà proposé son aide pour ce besoin." + refresh du detail.
**Fichiers** : `WishDetailViewModel.swift`

### 3.5 ✅
**Problème** : L'action la plus importante est cachée dans le detail sheet.
**Fix** : Ajouter chip "Confirmer" (primary) comme première action pour les wishes matched dans `EntraideMyNeedsContent`.
**Fichiers** : `EntraideMyNeedsContent.swift`

### 3.6 ✅
**Problème** : Pas de bouton "Publier" quand le mur est vide.
**Fix** : Ajouter `ctaTitle` + `ctaAction` dans l'empty state de `EntraideDiscoverContent`.
**Fichiers** : `EntraideDiscoverContent.swift`

## Phase 4 : Améliorations P1 ⬜

### 4.1 Recherche sur description ⬜
**Problème** : Search filtre uniquement sur le titre.
**Fix** : Filtrer sur `title` ET `description` dans `EntraideDiscoverContent.displayedWishes`.
**Fichiers** : `EntraideDiscoverContent.swift`

### 4.2 Bouton "Modifier" pour l'owner ⬜
**Problème** : Le backend supporte `updateWish`, le service `CommunityWishService.updateWish()` existe, mais aucune UI.
**Fix** : Ajouter bouton "Modifier" dans `WishDetailSheet` (owner + open) qui ouvre `CreateWishSheet` pré-rempli. Ajouter chip "Modifier" dans `EntraideMyNeedsContent`.
**Fichiers** : `WishDetailSheet.swift`, `EntraideMyNeedsContent.swift`, `CreateWishSheet.swift` (mode édition)

### 4.3 Bouton "Rouvrir" pour review ⬜
**Problème** : `reopenWish()` existe côté backend et ViewModel mais l'UI ne l'affiche jamais.
**Fix** : Chip "Rouvrir" dans `EntraideMyNeedsContent` pour status `.review` (gate: `reopenCount < 2`). Bouton dans `WishDetailSheet` aussi.
**Fichiers** : `EntraideMyNeedsContent.swift`, `WishDetailSheet.swift`

### 4.4 Message initial optionnel à l'offre ⬜
**Problème** : Quand on propose son aide, le thread de messages est vide.
**Fix** : Dans le dialog de confirmation (3.3), ajouter un TextField optionnel "Laissez un premier message (facultatif)". Si rempli, envoyer le message après le match.
**Fichiers** : `WishDetailSheet.swift`

### 4.5 Confirmation fulfillment avec gratitude ⬜
**Problème** : Confirmer = toast générique.
**Fix** : Dialog avant : "Confirmer que [donneur] vous a aidé ?" + TextArea "Remercier [donneur] (facultatif)" qui envoie un dernier message.
**Fichiers** : `WishDetailSheet.swift`, `EntraideMyNeedsContent.swift`

### 4.6 Célébration post-confirmation ⬜
**Problème** : Après confirmation, retour plat.
**Fix** : Animation brève (confetti ou illustration warm) + "Merci ! Ce geste a été enregistré." pendant 2-3s.
**Fichiers** : `WishDetailSheet.swift`

### 4.7 Polling adaptatif messages ⬜
**Problème** : 10s fixe — trop lent quand actif, gaspillage quand inactif.
**Fix** : 3s après envoi/réception → 10s après 30s d'inactivité → 30s après 2min.
**Fichiers** : `WishMessagesSheet.swift`

### 4.8 Preview photo dans CreateWishSheet ⬜
**Problème** : Après upload, juste un checkmark vert. L'user ne voit pas ce qu'il a uploadé.
**Fix** : Afficher thumbnail de l'image uploadée avec bouton X pour supprimer.
**Fichiers** : `CreateWishSheet.swift`

### 4.9 Card d'onboarding première visite ⬜
**Problème** : Nouveau user arrive sans contexte.
**Fix** : Card dismissable au-dessus du feed : "Bienvenue dans l'Entraide" + 3 bullets + bouton "Compris". UserDefaults pour ne plus afficher.
**Fichiers** : `EntraideDiscoverContent.swift`

### 4.10 Feedback post-signalement ⬜
**Problème** : Après signalement, le sheet se ferme sans feedback.
**Fix** : Avant fermeture, afficher "Merci pour votre signalement. Notre équipe va examiner ce besoin." pendant 2s.
**Fichiers** : `ReportWishSheet.swift`

### 4.11 FAB désactivé à 3 wishes ⬜
**Problème** : L'user peut taper le FAB alors qu'il a déjà 3 wishes actifs (erreur 409 du backend).
**Fix** : Vérifier le count dans le ViewModel, désactiver/masquer le FAB à 3. Ou montrer un tooltip explicatif.
**Fichiers** : `EntraideView.swift`, `EntraideMyNeedsViewModel.swift`

## Phase 5 : Polish P2 ⬜

### 5.1 Section "Récemment comblés" ⬜
**Backend** : Endpoint `GET /community/wishes/recent-fulfilled` (limit 5, last 7 days).
**Frontend** : Section horizontale compacte au-dessus du feed Discover.
**Fichiers** : Backend handler + `EntraideDiscoverContent.swift`

### 5.2 Indicateur d'âge sur vieux wishes ⬜
**Fix** : Pour wishes open > 48h, afficher "Publié il y a 3j — toujours en attente" en subtle.
**Fichiers** : `EntraideWishCard.swift`

### 5.3 Extraire couleurs/icônes catégorie ⬜
**Problème** : Mapping catégorie → couleur/icône dupliqué dans 3 fichiers.
**Fix** : Extension sur `WishCategory` avec `var color`, `var icon`, `var gradient`.
**Fichiers** : `CommunityWish.swift` (extension) + refactor 3 fichiers

### 5.4 Contexte temporel dans badges status Mes besoins ⬜
**Fix** : "En cours depuis 3j" au lieu de juste "En cours".
**Fichiers** : `EntraideMyNeedsContent.swift`

### 5.5 Pagination messages ⬜
**Fix** : Scroll-to-load-more en haut du thread pour les vieilles conversations.
**Fichiers** : `WishMessagesSheet.swift`

### 5.6 Champ libre pour signalement "Autre" ⬜
**Fix** : Quand "Autre" sélectionné, afficher TextField pour détailler.
**Fichiers** : `ReportWishSheet.swift`

### 5.7 Comparaison messages par last ID ⬜
**Problème** : `refreshMessages()` compare par count au lieu du dernier ID.
**Fix** : Comparer `response.data.last?.id != messages.last?.id`.
**Fichiers** : `WishMessagesSheet.swift`

## À ne PAS faire ❌

- ❌ Stats communauté (déprimant si nombre bas au lancement)
- ❌ Read receipts (pression sociale inappropriée)
- ❌ Partage de localisation (liability, hors scope)
- ❌ Description assistée par IA (inauthentique pour l'entraide)
- ❌ Compteur d'offres (pas pertinent avec le modèle 1:1)
- ❌ Visibilité post-report pour le reporter (incentive au report weaponisé)
