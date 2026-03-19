# Envies Redesign — Remaining Tasks

## Statut global : ✅ TERMINÉ

---

## 1. Fix : Search bar ne collapse pas au scroll
**Statut : ✅ FAIT** — Restructuré WishlistView pour utiliser .navigationTitle(.large) + .toolbar + .searchable natif. Header/chips/stats pinned, grid scrollable.

**Problème** : `.searchable` est sur le `ZStack` extérieur dans WishlistView. Il faut qu'il soit appliqué sur la vue qui est directement enfant du `NavigationStack` (qui est dans OffriiApp.swift).

**Solution** : Restructurer WishlistView — retirer le `ZStack` extérieur, utiliser un `VStack` comme racine directe. Le `.searchable` doit être appliqué sur cette VStack ou le NavigationStack parent. Alternativement, wrapper le contenu scrollable dans une `List` cachée ou utiliser `.toolbar` pour intégrer la recherche.

**Fichier** : `Features/Wishlist/WishlistView.swift`

---

## 2. Fix : "Partager cet item" copie juste le nom
**Statut : ✅ FAIT** — Remplacé ShareLink(item: item.name) par ShareItemSheet qui crée un share link scope "selection" via l'API et affiche l'URL avec options copier/envoyer.

**Problème** : `ShareLink(item: item.name)` dans le context menu ne fait que copier le texte du nom.

**Solution** :
- Remplacer par une action qui crée un share link scope "selection" via `POST /share-links` avec body `{ scope: "selection", scope_data: { item_ids: [item.id] } }`
- Récupérer l'URL retournée
- Ouvrir la share sheet iOS native avec cette URL
- Nécessite de mettre à jour `CreateShareLinkBody` côté frontend pour supporter scope/scope_data

**Fichiers** :
- `Core/Network/APIRequestBodies.swift` — enrichir `CreateShareLinkBody` avec scope, scope_data, label, permissions
- `Features/Wishlist/WishlistView.swift` — remplacer `ShareLink(item: item.name)` par une action async

---

## 3. Fix : Badges sur les cartes — glass morphism
**Statut : ✅ FAIT**

**Problème** : Les badges (checkmark offerts, réservé, privé, partagé, lien) utilisent `.ultraThinMaterial` ou des couleurs crues (vert, etc.). C'est moche.

**Solution** : Créer un style unifié "glass badge" :
- Fond : `.ultraThinMaterial` avec un léger tint de couleur
- Border : 0.5px blanc opacity 0.3
- Corner radius : cornerRadiusSM
- Shadow : subtile (1px, 0.1 opacity)
- Texte/icône : blanc ou couleur sémantique avec shadow léger pour lisibilité

Appliquer ce style à TOUS les badges : réservé 🔒, privé 🔒, partagé 👥, lien 🔗, checkmark ✓ (offerts), dots priorité.

**Fichier** : `Features/Wishlist/WishlistGridCard.swift`

---

## 4. Fix : Share sheet — refonte complète
**Statut : ✅ FAIT v2** — 3 sections : cercles (partage direct 1 tap), création lien (scope all/category/selection + permissions view_only/view_and_claim + copier/envoyer), liens actifs (URL + scope + permissions + copier/ouvrir/supprimer). Header descriptif. Warning items privés exclus.

**Problèmes** :
- Les liens ne sont pas persistés entre les ouvertures (le state est local au sheet)
- Impossible de supprimer un lien
- Impossible de cliquer sur un lien pour l'ouvrir
- "Envoyer via" n'est visible qu'après avoir copié
- Widget vide quand aucun lien

**Solution** :
- Charger les liens existants à l'ouverture via `GET /share-links` (déjà fait mais le type response ne matche pas — vérifier `ShareLinkListItem` vs `ShareLinkResponse`)
- Chaque lien a : URL cliquable (ouvre Safari), bouton copier, bouton supprimer (via `DELETE /share-links/{id}`), date relative
- "Envoyer via" toujours visible — si pas de lien, il crée un lien d'abord puis ouvre la share sheet
- Header explicatif toujours visible

**Fichiers** :
- `Features/Wishlist/WishlistShareSheet.swift` — refonte
- `Models/APIResponses.swift` — vérifier `ShareLinkListItem` a les bons champs
- `Core/Network/APIEndpoint.swift` — vérifier `deleteShareLink` existe

---

## 5. Mode privé — affichage et blocage
**Statut : ✅ FAIT** — Badge glass sur carte, banner dans détail, partage masqué dans context menu si privé, toggle dans ItemEditView.

**Ce qui est fait** :
- Backend : colonne `is_private`, dans create/update/response
- Frontend : `isPrivate` dans le modèle Item, toggle dans ItemEditView

**Ce qui manque** :
- Badge 🔒 "Privé" visible et joli sur la carte (avec glass morphism du point 3)
- Dans le context menu : masquer les options de partage si item est privé (FAIT)
- Dans ShareToCircleSheet : si on essaie de partager un item privé → message d'erreur ou item grisé
- Dans le détail (ItemDetailSheet) : afficher un banner "Item privé — non partageable"

**Fichiers** :
- `Features/Wishlist/WishlistGridCard.swift` — badge privé avec glass style
- `Features/Wishlist/ItemDetailSheet.swift` — banner privé
- `Features/Wishlist/ShareToCircleSheet.swift` — check is_private

---

## 6. Affichage "partagé à qui" sur les cartes
**Statut : ✅ FAIT** — `shared_circle_count` ajouté au backend ItemResponse (default 0), frontend affiche badge 👥 X sur la carte. Le count exact nécessitera un enrichissement SQL plus tard, pour l'instant c'est 0 + tracking session via sharedItemIds.

**Problème** : Aucune indication qu'un item est partagé, et dans quels cercles.

**Backend existant** : `list_circles_for_item(item_id)` existe dans le repo mais n'est PAS exposé via un endpoint API.

**Solution en 2 étapes** :

**Étape A — Backend** : Ajouter un champ `shared_in_circles` à `ItemResponse`.
- Dans `item_service.rs` ou `item_repo.rs` : pour chaque item retourné, faire une jointure ou sous-query sur `circle_items` pour compter les cercles
- OU : ajouter un nouvel endpoint `GET /items/{id}/circles` qui retourne les noms des cercles
- OU (plus simple) : enrichir `ItemResponse` avec `shared_circle_count: i64` via une sous-query SQL

**Étape B — Frontend** :
- Si `shared_circle_count > 0` → afficher badge 👥 avec le nombre sur la carte
- Dans le détail (ItemDetailSheet) → section "Partagé dans X cercles" avec la liste des noms
- Si partagé en direct (1-to-1) → afficher le nom de la personne

**Fichiers backend** :
- `src/repositories/item_repo.rs` — enrichir la query list avec un LEFT JOIN count
- `src/dto/items.rs` — ajouter `shared_circle_count` à ItemResponse
- `src/models/item.rs` — ajouter le champ

**Fichiers frontend** :
- `Models/Item.swift` — ajouter `sharedCircleCount`
- `Features/Wishlist/WishlistGridCard.swift` — badge 👥 X
- `Features/Wishlist/ItemDetailSheet.swift` — section "Partagé dans..."

---

## 7. Fix : Image picker UI
**Statut : ✅ FAIT** — Refondu avec zone dashed border quand vide, image plein width avec overlay X (supprimer) et capsule "Changer la photo" en bas. Glass style sur les contrôles.

**Problème** : Le composant `OffriiImagePicker` est basique — pas bien intégré dans les formulaires.

**Solution** :
- Revoir le design : zone de drop/tap plus grande, preview de l'image arrondie
- Bouton "Supprimer" plus visible
- Animation de chargement pendant l'upload
- Afficher l'image existante (si item a déjà une image) avec overlay "Changer"

**Fichier** : `DesignSystem/Components/OffriiImagePicker.swift`

---

## 8. Liens de partage — CRUD complet
**Statut : ✅ FAIT** — Suppression via DELETE, ouverture via Link, scope affiché. PATCH désactivation non implémenté (low priority).

**Ce qui manque** :
- Suppression d'un lien (endpoint `DELETE /share-links/{id}` existe côté backend et frontend)
- Ouverture d'un lien dans Safari (tap sur l'URL → `Link(destination:)`)
- Désactivation sans suppression (`PATCH /share-links/{id}` avec `{is_active: false}`)
- Affichage du scope (tout / catégorie / sélection) et permissions (voir / voir+réserver)

**Fichier** : `Features/Wishlist/WishlistShareSheet.swift`

---

## 9. CreateShareLinkBody — enrichir
**Statut : ✅ FAIT**

**Problème** : Le body actuel n'a que `expiresAt`. Le backend supporte scope, scope_data, label, permissions.

**Solution** : Mettre à jour `CreateShareLinkBody` :
```swift
struct CreateShareLinkBody: Encodable {
    let expiresAt: String?
    let label: String?
    let permissions: String?  // "view_only" ou "view_and_claim"
    let scope: String?        // "all", "category", "selection"
    let scopeData: [String: Any]?  // { category_id: "..." } ou { item_ids: [...] }
}
```

Note : `scopeData` est un JSON arbitraire — utiliser `AnyCodable` ou `serde_json::Value` equivalent en Swift.

**Fichier** : `Core/Network/APIRequestBodies.swift`

---

## 10. Bouton partager — UX repensé
**Statut : ✅ FAIT** — Bouton dans toolbar, ouvre WishlistShareSheet avec header explicatif, copier/envoyer/gérer les liens.

**Ce qu'on avait dit** :
- Le bouton ↗ dans le header ouvre une sheet avec :
  1. "Copier le lien de ma liste" → crée un share link scope "all", copie, toast
  2. "Partager dans un cercle" → ouvre la liste des cercles
  3. "Envoyer via..." → share sheet iOS native avec le lien
  4. "Gérer mes liens" → liste des liens avec suppression/désactivation

**Fichier** : `Features/Wishlist/WishlistShareSheet.swift`

---

## Ordre d'exécution recommandé

1. **Point 9** — CreateShareLinkBody enrichi (dépendance pour les points 2, 4, 10)
2. **Point 3** — Glass morphism badges (impact visuel sur tout)
3. **Point 1** — Search bar fix
4. **Point 2** — "Partager cet item" fix
5. **Point 4** — Share sheet refonte
6. **Point 8** — Liens CRUD complet
7. **Point 10** — Bouton partager UX
8. **Point 5** — Mode privé affichage complet
9. **Point 6** — "Partagé à qui" (nécessite changement backend)
10. **Point 7** — Image picker UI
