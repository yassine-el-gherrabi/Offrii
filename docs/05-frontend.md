# Frontend iOS

## Stack technique

| Composant | Choix | Details |
|---|---|---|
| Framework UI | **SwiftUI** | Declaratif, iOS 17+ |
| Langage | **Swift 5.9+** | Swift Concurrency (`async`/`await`) |
| Cible minimale | **iOS 17.0** | Macro `@Observable`, `NavigationStack` |
| Gestionnaire de paquets | **Swift Package Manager** | 3 dependances externes |
| Stockage securise | **Keychain** (via KeychainAccess) | Tokens d'authentification |
| Chargement d'images | **Nuke 12** | Cache et decodage asynchrone |
| SSO | **Google Sign-In 8** | Authentification Google |

Aucun autre framework tiers. Pas de RxSwift, pas de Combine explicite, pas d'Alamofire.

---

## Architecture

### Pattern View-ViewModel

Chaque ecran suit le pattern **View + ViewModel** :

- La **View** (SwiftUI) observe le ViewModel via `@Observable` (Observation framework, iOS 17).
- Le **ViewModel** contient la logique metier, appelle les services reseau, et expose les etats a la vue.
- Les **Services** (`ItemService`, `CircleService`, etc.) encapsulent les appels API par domaine.

### Objets d'environnement

| Objet | Role | Injection |
|---|---|---|
| `AuthManager` | Etat d'authentification, utilisateur courant | `@Environment(AuthManager.self)` |
| `AppRouter` | Navigation globale, ecran courant, deep links | `@Environment(AppRouter.self)` |
| `OnboardingTipManager` | Gestion des tips de premier lancement | `@Environment(OnboardingTipManager.self)` |

### Gestion de l'authentification

Les tokens (`accessToken`, `refreshToken`) sont stockes dans le **Keychain** via `KeychainService`. Le `APIClient` injecte automatiquement le header `Authorization: Bearer` sur les requetes authentifiees. En cas de **401**, un refresh transparent est tente une fois avant de deconnecter l'utilisateur.

---

## Modules fonctionnels

| Module | Repertoire | Description |
|---|---|---|
| **Launch** | `Features/Launch/` | Ecran de splash, verification de l'etat d'authentification |
| **Onboarding** | `Features/Onboarding/` | Ecrans de bienvenue (premier lancement), tips contextuels |
| **Auth** | `Features/Auth/` | Inscription, connexion (email + Google SSO), mot de passe oublie, setup post-inscription |
| **Home** | `Features/Home/` | Dashboard principal, progression du profil, vue d'ensemble |
| **Wishlist** | `Features/Wishlist/` | Liste d'envies, ajout/edition d'items, partage, detail d'item |
| **Circles** | `Features/Circles/` | Cercles (groupes prives), invitations, feed d'activite, reservations |
| **Entraide** | `Features/Entraide/` | Besoins communautaires, offres d'aide, messagerie, decouverte |
| **Friends** | `Features/Friends/` | Ajout d'amis, liste d'amis |
| **Profile** | `Features/Profile/` | Edition du profil, mot de passe, gestion des donnees, username |
| **Notifications** | `Features/Notifications/` | Centre de notifications in-app |
| **Categories** | `Features/Categories/` | Picker de categories (composant partage) |
| **Legal** | `Features/Legal/` | CGU, politique de confidentialite |
| **Placeholder** | `Features/Placeholder/` | Ecran "bientot disponible" (fonctionnalites a venir) |

---

## Design System

### OffriiTheme

Le design system est centralise dans `DesignSystem/Theme.swift`. Toutes les couleurs sont definies dans le Asset Catalog et referencees par nom.

| Token | Valeur | Usage |
|---|---|---|
| `primary` | **Coral #FF6B6B** | Couleur principale, boutons, liens |
| `secondary` | Bleu clair | Elements secondaires |
| `accent` | **Ambre** | Module Entraide, actions communautaires |
| `success` / `warning` / `danger` | Vert / Jaune / Rouge | Etats semantiques |
| `background` / `surface` / `card` | Blancs et gris clairs | Surfaces et conteneurs |

### Typographie

`OffriiTypography` utilise **SF Pro** (police systeme) avec Dynamic Type pour l'accessibilite. Aucune police custom n'est embarquee.

### Composants reutilisables

| Composant | Fichier | Description |
|---|---|---|
| `OffriiButton` | `Components/OffriiButton.swift` | Bouton principal avec variantes (primary, secondary, danger) |
| `OffriiTextField` | `Components/OffriiTextField.swift` | Champ de texte avec validation et style unifie |
| `OffriiCard` | `Components/OffriiCard.swift` | Carte avec ombre et coins arrondis |
| `OffriiChip` | `Components/OffriiChip.swift` | Tag/chip selectionnable |
| `OffriiSpinner` | `Components/OffriiSpinner.swift` | Indicateur de chargement |
| `OffriiToast` | `Components/OffriiToast.swift` | Notification temporaire en overlay |
| `OffriiTooltip` | `Components/OffriiTooltip.swift` | Bulle d'aide contextuelle |
| `OffriiEmptyState` | `Components/OffriiEmptyState.swift` | Etat vide avec illustration et CTA |
| `OffriiErrorState` | `Components/OffriiErrorState.swift` | Etat d'erreur avec action de retry |
| `OffriiFloatingActionButton` | `Components/OffriiFloatingActionButton.swift` | FAB flottant |
| `OffriiGridCard` | `Components/OffriiGridCard.swift` | Carte pour affichage en grille |
| `OffriiImagePicker` | `Components/OffriiImagePicker.swift` | Selecteur d'image (galerie + camera) |
| `AvatarView` | `Components/AvatarView.swift` | Avatar utilisateur avec initiales en fallback |
| `AvatarStack` | `Components/AvatarStack.swift` | Pile d'avatars superposee |
| `OTPField` | `Components/OTPField.swift` | Champ de saisie code OTP |
| `SSOButton` | `Components/SSOButton.swift` | Bouton "Se connecter avec Google" |
| `TabBarView` | `Components/TabBarView.swift` | Barre d'onglets custom avec bouton central |
| `CategoryChipsBar` | `Components/CategoryChipsBar.swift` | Barre de filtres par categorie |
| `MessageBubble` | `Components/MessageBubble.swift` | Bulle de message (module Entraide) |
| `SectionHeader` | `Components/SectionHeader.swift` | En-tete de section avec degradé contextuel |
| `QuickCreateSheet` | `Components/QuickCreateSheet.swift` | Sheet de creation rapide (bouton +) |

### Autres tokens

| Categorie | Fichier | Exemples |
|---|---|---|
| Espacements | `Spacing.swift` | `spacingXS` (6pt) a `spacingGiant` (64pt) |
| Rayons | `Radius.swift` | `cornerRadiusSM` (8pt) a `cornerRadiusFull` (pill) |
| Ombres | `Shadows.swift` | SM, MD, LG avec adaptation light/dark |
| Animations | `Animations.swift` | Spring par defaut (response: 0.35, damping: 0.7) |
| Formes | `Shapes/BlobShape.swift` | Formes organiques animees (splash) |
| Squelettes | `Skeletons.swift` | Placeholders de chargement animes |

---

## Localisation

| Langue | Fichier | Cles |
|---|---|---|
| Francais | `Localization/fr.lproj/Localizable.strings` | ~847 |
| Anglais | `Localization/en.lproj/Localizable.strings` | ~847 |

Format standard Apple `Localizable.strings` (cle-valeur). Les cles sont organisees par module : `auth.*`, `home.*`, `wishlist.*`, `circles.*`, `entraide.*`, `profile.*`, `common.*`, `error.*`.

L'utilisation dans le code se fait via `NSLocalizedString(_:comment:)` et `String(localized:)`.

---

## Navigation

### Flux d'ecrans

L'`AppRouter` gere le flux de navigation global via un `enum AppScreen` :

```
splash --> [authentifie?]
  |-- oui --> main (TabView)
  |-- non, premier lancement --> welcome --> auth (inscription)
  |-- non, utilisateur connu --> auth (connexion)
```

Apres inscription, un ecran `postAuthSetup` permet de configurer le profil avant d'acceder a l'application.

### TabView

La barre d'onglets est un composant custom (`TabBarView`) avec 5 positions :

| Position | Onglet | Ecran | Icone |
|---|---|---|---|
| 0 | **Home** | `HomeView` | Maison |
| 1 | **Envies** | `WishlistView` | Coeur |
| 2 | **Creer** (+) | Sheet `QuickCreateSheet` | Bouton central flottant |
| 3 | **Cercles** | `CirclesListView` | Cercles |
| 4 | **Entraide** | `EntraideView` | Mains |

Chaque onglet encapsule son propre `NavigationStack`. Les vues de detail s'ouvrent en **sheet** (`.sheet`, `.presentationDetents`).

### Deep links

Le schema `offrii://join/{token}` est gere par `AppRouter.handleURL(_:)` pour les invitations de cercle. Le token est stocke dans `pendingInviteToken` et traite a l'affichage du `MainTabView`.

---

## Couche reseau

### APIClient

Singleton centralise (`APIClient.shared`) qui gere toutes les communications HTTP.

| Responsabilite | Detail |
|---|---|
| Construction des requetes | A partir d'un `enum APIEndpoint` (URL, methode, body, auth requise) |
| Injection Bearer | Automatique depuis le Keychain sur les endpoints authentifies |
| Refresh transparent | Retry automatique une fois sur 401 via `RefreshCoordinator` (actor) |
| Decodage | `JSONDecoder` avec ISO 8601 (fractional seconds), `snake_case` keys |
| Encodage | `JSONEncoder` avec `convertToSnakeCase` |
| Upload multipart | Images (avatar, item, cercle) via `multipart/form-data` |
| Error mapping | Parsing de l'enveloppe backend `{ "error": { "code", "message" } }` vers `APIError` |

### APIError

| Cas | Code HTTP | Description |
|---|---|---|
| `badRequest` | 400 | Requete invalide |
| `unauthorized` | 401 / 403 | Token invalide ou expire |
| `notFound` | 404 | Ressource introuvable |
| `conflict` | 409 | Conflit (ex : email deja utilise) |
| `tooManyRequests` | 429 | Rate limiting |
| `serverError` | 5xx | Erreur serveur |
| `networkError` | -- | Erreur reseau (pas de connexion) |
| `decodingError` | -- | Erreur de decodage JSON |

### Services

Chaque domaine metier dispose d'un service dedie :

| Service | Endpoints couverts |
|---|---|
| `UserService` | Profil utilisateur, mise a jour, avatar |
| `ItemService` | CRUD items de wishlist |
| `CircleService` | CRUD cercles, invitations, membres |
| `FriendService` | Ajout/suppression d'amis |
| `CommunityWishService` | Besoins Entraide, offres, decouverte |
| `WishMessageService` | Messagerie Entraide |
| `CategoryService` | Liste des categories |
| `NotificationCenterService` | Notifications in-app, compteur non-lus |
| `PushTokenService` | Enregistrement du token APNS |

---

## Points d'attention

| Sujet | Etat | Detail |
|---|---|---|
| Mode sombre | **Non supporte** | Choix intentionnel pour la V1. Le code prevoit un `@AppStorage("appearanceMode")` mais le design n'est valide qu'en mode clair. |
| iOS minimum | **17.0** | Requis pour `@Observable`, `NavigationStack`, `.presentationDetents`. Pas de fallback iOS 16. |
| Dependances SPM | **3 packages** | `KeychainAccess` (stockage securise), `Nuke` (images), `GoogleSignIn` (SSO). Volonte de rester minimaliste. |
| Accessibilite | **Partielle** | Dynamic Type supporte via SF Pro. Pas d'audit VoiceOver complet. |
| Tests unitaires | **Non presents** | La logique metier est testee cote backend (1 042 tests d'integration). Cote iOS, les tests restent a ecrire. |
| Xcode Cloud | **Configure** | Build et distribution automatises via Xcode Cloud. |
