# Roadmap : Finalisation Cercles & Social

## Statuts
- ⬜ À faire
- 🔄 En cours
- ✅ Terminé
- ⏭️ Reporté

---

## 1. Système d'invitation de cercle ✅

### Backend
- ✅ 4 endpoints (create, list, revoke, join)

### iOS
- ✅ API Layer + models
- ✅ InviteFriendsSheet redesignée (lien d'invitation, ShareLink, confirmation, nom créateur)
- ✅ Deep link offrii://join/{token} + HTML page GET /join/{token}
- ✅ Favicon "O" corail + avatar cercle sur page web

---

## 2. Centre de notifications complet ✅

### Backend
- ✅ Table `notifications` + repo + DTO + 4 endpoints
- ✅ Persistance dans circle_service et friend_service
- ✅ Custom data dans payload APNs (circle_id, item_id, type)

### iOS
- ✅ AppNotification model + NotificationCenterService
- ✅ NotificationCenterView (liste paginée, read/unread, mark all read)
- ✅ Badge cloche avec compteur non-lues
- ✅ Push foreground (banner + sound via UNUserNotificationCenterDelegate)
- ✅ Deep link au tap push notification → navigation vers cercle
- ✅ Tap notification dans le centre → navigation vers cercle

---

## 3. Feed d'activité ✅

- ✅ 6 types d'événements (item_shared, item_claimed, item_unclaimed, item_received, member_joined, member_left)
- ✅ Backend utilise display_name (avec fallback username) dans les events
- ✅ Anti-spoiler : claim/unclaim masqués pour le propriétaire de l'item
- ✅ Groupement temporel (today, yesterday, thisWeek, older)

---

## 4. Confirmations & Avatars ✅

### Confirmations
- ✅ Retirer membre, supprimer cercle, quitter cercle
- ✅ Supprimer lien de partage, départager item
- ✅ Supprimer envie (grid + detail)

### Avatars
- ✅ Member carousel, circle list cards, direct header
- ✅ Group header, members tab, item cards (owner)

---

## Prêt pour test

Toutes les fonctionnalités sont implémentées. Points à vérifier manuellement :
1. Effectuer une action (claim, partage) → vérifier notification dans la cloche
2. Vérifier badge compteur non-lues
3. Tap sur notification → navigation vers le cercle concerné
4. Push notification en foreground (banner)
5. Tap sur push notification → navigation vers le cercle
6. Feed d'activité affiche les bons noms (display_name)
7. Mark all read fonctionne

---

*Dernière mise à jour : 2026-03-17*
