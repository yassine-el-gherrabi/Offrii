# Frontend iOS

## Stack technique

| Composant | Choix | Détails |
|---|---|---|
| Framework UI | **SwiftUI** | Déclaratif, iOS 17+ |
| Langage | **Swift 5.9+** | Swift Concurrency (`async`/`await`) |
| Cible minimale | **iOS 17.0** | Macro `@Observable`, `NavigationStack` |
| Gestionnaire de paquets | **Swift Package Manager** | 3 dépendances externes |
| Stockage sécurisé | **Keychain** (via KeychainAccess) | Tokens d'authentification |
| Chargement d'images | **Nuke 12** | Cache et décodage asynchrone |
| SSO | **Google Sign-In 8** | Authentification Google |

Aucun autre framework tiers. Pas de RxSwift, pas de Combine explicite, pas d'Alamofire.

---

## Architecture

### Pattern View-ViewModel

Chaque écran suit le pattern **View + ViewModel** :

- La **View** (SwiftUI) observe le ViewModel via `@Observable` (Observation framework, iOS 17).
- Le **ViewModel** contient la logique métier, appelle les services réseau, et expose les états à la vue.
- Les **Services** (`ItemService`, `CircleService`, etc.) encapsulent les appels API par domaine.

### Objets d'environnement

| Objet | Rôle | Injection |
|---|---|---|
| `AuthManager` | État d'authentification, utilisateur courant | `@Environment(AuthManager.self)` |
| `AppRouter` | Navigation globale, écran courant, deep links | `@Environment(AppRouter.self)` |
| `OnboardingTipManager` | Gestion des tips de premier lancement | `@Environment(OnboardingTipManager.self)` |

### Gestion de l'authentification

Les tokens (`accessToken`, `refreshToken`) sont stockés dans le **Keychain** via `KeychainService`. Le `APIClient` injecte automatiquement le header `Authorization: Bearer` sur les requêtes authentifiées. En cas de **401**, un refresh transparent est tenté une fois avant de déconnecter l'utilisateur.

---

## Modules fonctionnels

| Module | Répertoire | Description |
|---|---|---|
| **Launch** | `Features/Launch/` | Écran de splash, vérification de l'état d'authentification |
| **Onboarding** | `Features/Onboarding/` | Écrans de bienvenue (premier lancement), tips contextuels |
| **Auth** | `Features/Auth/` | Inscription, connexion (email + Google SSO), mot de passe oublié, setup post-inscription |
| **Home** | `Features/Home/` | Dashboard principal, progression du profil, vue d'ensemble |
| **Wishlist** | `Features/Wishlist/` | Liste d'envies, ajout/édition d'items, partage, détail d'item |
| **Circles** | `Features/Circles/` | Cercles (groupes privés), invitations, feed d'activité, réservations |
| **Entraide** | `Features/Entraide/` | Besoins communautaires, offres d'aide, messagerie, découverte |
| **Friends** | `Features/Friends/` | Ajout d'amis, liste d'amis |
| **Profile** | `Features/Profile/` | Édition du profil, mot de passe, gestion des données, username |
| **Notifications** | `Features/Notifications/` | Centre de notifications in-app |
| **Categories** | `Features/Categories/` | Picker de catégories (composant partagé) |
| **Legal** | `Features/Legal/` | CGU, politique de confidentialité |
| **Placeholder** | `Features/Placeholder/` | Écran "bientôt disponible" (fonctionnalités à venir) |

---

## Design System

### OffriiTheme

Le design system est centralisé dans `DesignSystem/Theme.swift`. Toutes les couleurs sont définies dans le Asset Catalog et référencées par nom.

| Token | Valeur | Usage |
|---|---|---|
| `primary` | **Coral #FF6B6B** | Couleur principale, boutons, liens |
| `secondary` | Bleu clair | Éléments secondaires |
| `accent` | **Ambre** | Module Entraide, actions communautaires |
| `success` / `warning` / `danger` | Vert / Jaune / Rouge | États sémantiques |
| `background` / `surface` / `card` | Blancs et gris clairs | Surfaces et conteneurs |

### Typographie

`OffriiTypography` utilise **SF Pro** (police système) avec Dynamic Type pour l'accessibilité. Aucune police custom n'est embarquée.

### Composants réutilisables

| Composant | Fichier | Description |
|---|---|---|
| `OffriiButton` | `Components/OffriiButton.swift` | Bouton principal avec variantes (primary, secondary, danger) |
| `OffriiTextField` | `Components/OffriiTextField.swift` | Champ de texte avec validation et style unifié |
| `OffriiCard` | `Components/OffriiCard.swift` | Carte avec ombre et coins arrondis |
| `OffriiChip` | `Components/OffriiChip.swift` | Tag/chip sélectionnable |
| `OffriiSpinner` | `Components/OffriiSpinner.swift` | Indicateur de chargement |
| `OffriiToast` | `Components/OffriiToast.swift` | Notification temporaire en overlay |
| `OffriiTooltip` | `Components/OffriiTooltip.swift` | Bulle d'aide contextuelle |
| `OffriiEmptyState` | `Components/OffriiEmptyState.swift` | État vide avec illustration et CTA |
| `OffriiErrorState` | `Components/OffriiErrorState.swift` | État d'erreur avec action de retry |
| `OffriiFloatingActionButton` | `Components/OffriiFloatingActionButton.swift` | FAB flottant |
| `OffriiGridCard` | `Components/OffriiGridCard.swift` | Carte pour affichage en grille |
| `OffriiImagePicker` | `Components/OffriiImagePicker.swift` | Sélecteur d'image (galerie + caméra) |
| `AvatarView` | `Components/AvatarView.swift` | Avatar utilisateur avec initiales en fallback |
| `AvatarStack` | `Components/AvatarStack.swift` | Pile d'avatars superposée |
| `OTPField` | `Components/OTPField.swift` | Champ de saisie code OTP |
| `SSOButton` | `Components/SSOButton.swift` | Bouton "Se connecter avec Google" |
| `TabBarView` | `Components/TabBarView.swift` | Barre d'onglets custom avec bouton central |
| `CategoryChipsBar` | `Components/CategoryChipsBar.swift` | Barre de filtres par catégorie |
| `MessageBubble` | `Components/MessageBubble.swift` | Bulle de message (module Entraide) |
| `SectionHeader` | `Components/SectionHeader.swift` | En-tête de section avec dégradé contextuel |
| `QuickCreateSheet` | `Components/QuickCreateSheet.swift` | Sheet de création rapide (bouton +) |

### Autres tokens

| Catégorie | Fichier | Exemples |
|---|---|---|
| Espacements | `Spacing.swift` | `spacingXS` (6pt) à `spacingGiant` (64pt) |
| Rayons | `Radius.swift` | `cornerRadiusSM` (8pt) à `cornerRadiusFull` (pill) |
| Ombres | `Shadows.swift` | SM, MD, LG avec adaptation light/dark |
| Animations | `Animations.swift` | Spring par défaut (response: 0.35, damping: 0.7) |
| Formes | `Shapes/BlobShape.swift` | Formes organiques animées (splash) |
| Squelettes | `Skeletons.swift` | Placeholders de chargement animés |

---

## Localisation

| Langue | Fichier | Clés |
|---|---|---|
| Français | `Localization/fr.lproj/Localizable.strings` | ~847 |
| Anglais | `Localization/en.lproj/Localizable.strings` | ~847 |

Format standard Apple `Localizable.strings` (clé-valeur). Les clés sont organisées par module : `auth.*`, `home.*`, `wishlist.*`, `circles.*`, `entraide.*`, `profile.*`, `common.*`, `error.*`.

L'utilisation dans le code se fait via `NSLocalizedString(_:comment:)` et `String(localized:)`.

---

## Navigation

### Flux d'écrans

L'`AppRouter` gère le flux de navigation global via un `enum AppScreen` :

```
splash --> [authentifié?]
  |-- oui --> main (TabView)
  |-- non, premier lancement --> welcome --> auth (inscription)
  |-- non, utilisateur connu --> auth (connexion)
```

Après inscription, un écran `postAuthSetup` permet de configurer le profil avant d'accéder à l'application.

### TabView

La barre d'onglets est un composant custom (`TabBarView`) avec 5 positions :

| Position | Onglet | Écran | Icône |
|---|---|---|---|
| 0 | **Home** | `HomeView` | Maison |
| 1 | **Envies** | `WishlistView` | Cœur |
| 2 | **Créer** (+) | Sheet `QuickCreateSheet` | Bouton central flottant |
| 3 | **Cercles** | `CirclesListView` | Cercles |
| 4 | **Entraide** | `EntraideView` | Mains |

Chaque onglet encapsule son propre `NavigationStack`. Les vues de détail s'ouvrent en **sheet** (`.sheet`, `.presentationDetents`).

### Deep links

Le schéma `offrii://join/{token}` est géré par `AppRouter.handleURL(_:)` pour les invitations de cercle. Le token est stocké dans `pendingInviteToken` et traité à l'affichage du `MainTabView`.

---

## Couche réseau

### APIClient

Singleton centralisé (`APIClient.shared`) qui gère toutes les communications HTTP.

| Responsabilité | Détail |
|---|---|
| Construction des requêtes | À partir d'un `enum APIEndpoint` (URL, méthode, body, auth requise) |
| Injection Bearer | Automatique depuis le Keychain sur les endpoints authentifiés |
| Refresh transparent | Retry automatique une fois sur 401 via `RefreshCoordinator` (actor) |
| Décodage | `JSONDecoder` avec ISO 8601 (fractional seconds), `snake_case` keys |
| Encodage | `JSONEncoder` avec `convertToSnakeCase` |
| Upload multipart | Images (avatar, item, cercle) via `multipart/form-data` |
| Error mapping | Parsing de l'enveloppe backend `{ "error": { "code", "message" } }` vers `APIError` |

### APIError

| Cas | Code HTTP | Description |
|---|---|---|
| `badRequest` | 400 | Requête invalide |
| `unauthorized` | 401 / 403 | Token invalide ou expiré |
| `notFound` | 404 | Ressource introuvable |
| `conflict` | 409 | Conflit (ex : email déjà utilisé) |
| `tooManyRequests` | 429 | Rate limiting |
| `serverError` | 5xx | Erreur serveur |
| `networkError` | -- | Erreur réseau (pas de connexion) |
| `decodingError` | -- | Erreur de décodage JSON |

### Services

Chaque domaine métier dispose d'un service dédié :

| Service | Endpoints couverts |
|---|---|
| `UserService` | Profil utilisateur, mise à jour, avatar |
| `ItemService` | CRUD items de wishlist |
| `CircleService` | CRUD cercles, invitations, membres |
| `FriendService` | Ajout/suppression d'amis |
| `CommunityWishService` | Besoins Entraide, offres, découverte |
| `WishMessageService` | Messagerie Entraide |
| `CategoryService` | Liste des catégories |
| `NotificationCenterService` | Notifications in-app, compteur non-lus |
| `PushTokenService` | Enregistrement du token APNS |

---

## Points d'attention

| Sujet | État | Détail |
|---|---|---|
| Mode sombre | **Non supporté** | Choix intentionnel pour la V1. Le code prévoit un `@AppStorage("appearanceMode")` mais le design n'est validé qu'en mode clair. |
| iOS minimum | **17.0** | Requis pour `@Observable`, `NavigationStack`, `.presentationDetents`. Pas de fallback iOS 16. |
| Dépendances SPM | **3 packages** | `KeychainAccess` (stockage sécurisé), `Nuke` (images), `GoogleSignIn` (SSO). Volonté de rester minimaliste. |
| Accessibilité | **Partielle** | Dynamic Type supporté via SF Pro. Pas d'audit VoiceOver complet. |
| Tests unitaires | **Non présents** | La logique métier est testée côté backend (1 042 tests d'intégration). Côté iOS, les tests restent à écrire. |
| Xcode Cloud | **Configuré** | Build et distribution automatisés via Xcode Cloud. |
