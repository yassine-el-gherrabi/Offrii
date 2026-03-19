# Entraide — Refonte complète

## Phase 1 : Corrections backend (gaps identifiés)

### 1. Re-modération sur update ✅
**Problème** : Un user peut modifier le contenu d'un wish après approbation sans re-modération.
**Fix** : Quand `update_wish` est appelé sur un wish `open` ou `review`, relancer la modération async (même flow que création : passer en `pending` → modération → `open`/`flagged`).
**Tests** :
- [ ] Update wish content triggers re-moderation
- [ ] Wish goes to pending during re-moderation
- [ ] Wish returns to open after approved re-moderation
- [ ] Wish goes to flagged if re-moderation flags it
- [ ] Update from review status also triggers re-moderation

### 2. Intégration notification center ✅
**Problème** : Les push notifications entraide sont envoyées mais pas stockées dans la table `notifications` → invisibles dans la cloche in-app.
**Fix** : À chaque envoi de push dans `community_wish_service` et `wish_message_service`, aussi créer un record dans `notifications` via `NotificationRepo::create`.
**Tests** :
- [ ] Offer creates notification for owner
- [ ] Withdraw creates notification for owner
- [ ] Reject creates notification for donor
- [ ] Confirm creates notification for donor
- [ ] Close (when matched) creates notification for donor
- [ ] Report threshold creates notification for owner
- [ ] Moderation result creates notification for owner
- [ ] New message creates notification for recipient
- [ ] Notifications appear in GET /me/notifications

### 3. Update + reopen contourne les signalements ✅
**Problème** : Un user peut modifier le contenu d'un wish en `review`, puis le réouvrir — les signalements sont reset mais le nouveau contenu n'est pas modéré.
**Fix** : `reopen_wish` doit relancer la modération. Au lieu de passer directement en `open`, passer en `pending` → modération async → `open`/`flagged`.
**Tests** :
- [ ] Reopen triggers re-moderation (status goes to pending, not open)
- [ ] Reopen with clean content → eventually open
- [ ] Reopen with flagged content → eventually flagged
- [ ] Reopen still respects max 2 reopens + cooldown

### 4. État orphelin si donneur supprime son compte ✅
**Problème** : Si le donneur supprime son compte, le wish reste `matched` avec `matched_with = NULL`.
**Fix** : Ajouter un trigger SQL ou un check dans `delete_account` qui clear les matchs ouverts (set wishes back to `open` where `matched_with = user_id`).
**Tests** :
- [ ] Delete donor account → matched wish returns to open
- [ ] Owner can see wish is back to open
- [ ] Another donor can now offer

### 5. Messages supprimés en cascade ✅
**Problème** : `wish_messages.sender_id` a `ON DELETE CASCADE` — si un user est supprimé, ses messages disparaissent.
**Fix** : Changer en `ON DELETE SET NULL` + gérer les messages avec sender_id NULL côté affichage (afficher "Utilisateur supprimé").
**Tests** :
- [ ] Delete user → messages preserved with sender_id NULL
- [ ] List messages after sender deletion returns messages with null sender
- [ ] Message response handles null sender gracefully

### 6. Paginer list_my_wishes ⏭️ (déprioritisé — max 3 wishes actifs, faible impact)
**Problème** : `GET /community/wishes/mine` retourne tout sans pagination.
**Fix** : Ajouter pagination (page/limit) comme `list_wishes`.
**Tests** :
- [ ] list_my_wishes respects page/limit
- [ ] list_my_wishes returns correct total count
- [ ] list_my_wishes default pagination works

### 7. Pas de validation URL ✅
**Problème** : Les liens et image_url ne sont pas validés comme URLs.
**Fix** : Ajouter validation regex ou url::Url parse sur image_url et chaque lien.
**Tests** :
- [ ] Valid URLs accepted
- [ ] Invalid URLs rejected (400)
- [ ] Empty string link rejected
- [ ] Protocol-less URLs rejected

### 8. Dead code cleanup ✅
**Problème** : `find_user_is_admin` dans le repo est inutilisé.
**Fix** : Supprimer la méthode du trait et de l'impl.
**Tests** :
- [ ] Compilation passes after removal

---

## Phase 2 : Refonte frontend ✅

- Réécriture from scratch des 19 fichiers → 12 nouveaux
- Layout aligné sur Envies/Proches (.navigationTitle, .searchable, chips, stats bar)
- Cards avec glassBadge style Envies, gradients vibrants
- Recherche native iOS sur les 3 pages

## Phase 3 : Polish UX cards (en cours) ⬜

### Frontend (pas de changement backend)
- [x] Supprimer badge "Ouvert" sur Découvrir (bruit visuel)
- [x] Supprimer badge "Anonyme" sur les cards (contre-productif)
- [x] Ajouter aperçu description 1 ligne sur les cards
- [x] Afficher nom auteur dans le sous-titre (humaniser)
- [x] Afficher sous-titre "Des gestes simples, des impacts réels"
- [ ] Hint première visite (card contextuelle dismissable)

## Phase 4 : Améliorations backend engagement ⬜

### 1. Compteur d'offres sur les wishes ⬜
**Besoin** : Ajouter `offer_count` au `WishResponse` DTO pour afficher "0 offres" sur les cards.
**Pourquoi** : Le plus gros levier d'engagement. "Soyez le premier à aider" est plus motivant que rien.
**Backend** : Ajouter une colonne ou un COUNT dans la query `list_open`. Incrémenter quand `offer_wish`, décrémenter quand `withdraw_offer` / `reject_offer`.
**Frontend** : Afficher dans le sous-titre de la card.
**Tests** :
- [ ] offer increments count
- [ ] withdraw decrements count
- [ ] reject decrements count
- [ ] count visible in list response

### 2. Stats communauté ⬜
**Besoin** : Endpoint `GET /community/wishes/stats` retournant le nombre de wishes fulfilled cette semaine.
**Pourquoi** : Preuve sociale que le système fonctionne.
**Backend** : Simple COUNT WHERE status='fulfilled' AND fulfilled_at > NOW() - 7 days.
**Frontend** : Banner en haut du Discover "X souhaits offerts cette semaine".
**Tests** :
- [ ] Stats endpoint returns correct count
- [ ] Count resets weekly

### 3. Wishes récemment comblés ⬜
**Besoin** : Endpoint `GET /community/wishes/recent-fulfilled` (limit 3, last 7 days).
**Pourquoi** : Montre que le système marche, inspire confiance.
**Backend** : Query WHERE status='fulfilled' ORDER BY fulfilled_at DESC LIMIT 3.
**Frontend** : Section horizontale "Récemment comblés" en haut du Discover.
**Tests** :
- [ ] Returns only fulfilled wishes
- [ ] Respects 7-day window
- [ ] Limited to 3 results
