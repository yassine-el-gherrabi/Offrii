# Share Rules Implementation — État d'avancement

## ✅ Fait — Backend
- Migration `circle_share_rules` (up + down) — `479294c`
- Model `CircleShareRule` (Rust struct) — `479294c`
- Repository `PgCircleShareRuleRepo` (get, upsert, delete) — `479294c`
- Trait `CircleShareRuleRepo` dans traits.rs — `479294c`
- DTO (SetShareRuleRequest + ShareRuleResponse) — `6c3d643`
- Handler (GET + PUT /circles/{id}/share-rule) — `6c3d643`
- AppState wiring (main.rs + test common) — `6c3d643`
- Service: list_direct_circle_items dynamique — `5b78d5f`
- Tests (5): all/private/dynamic/none/default — `5b78d5f`

## 🔄 À faire — Backend

### 1. DTO (ShareRuleRequest + ShareRuleResponse)
- `PUT /circles/{id}/share-rule` body: `{ share_mode, category_ids? }`
- `GET /circles/{id}/share-rule` response: `{ share_mode, category_ids, updated_at }`

### 2. Handler (circles.rs)
- Route `PUT /{id}/share-rule` → `set_share_rule`
- Route `GET /{id}/share-rule` → `get_share_rule`

### 3. Service (circle_service.rs)
- `set_share_rule()` : validation + upsert + notification si passage de none → autre
- `get_share_rule()` : simple get
- Modifier `list_circle_items()` pour cercles directs :
  - Si rule = all → query items WHERE user_id = sharer AND status = active AND is_private = false
  - Si rule = categories → idem + WHERE category_id = ANY(rule.category_ids)
  - Si rule = selection → comportement actuel (circle_items)
  - Si rule = none → vide

### 4. Modifier `count_shared_items_per_user` (friend_repo.rs)
- Actuellement : count circle_items WHERE is_direct = true
- Nouveau : aussi compter selon les share rules dynamiques

### 5. Wire dans AppState (main.rs, lib.rs)
- Ajouter `share_rules: Arc<dyn CircleShareRuleRepo>` à AppState

### 6. Tests
- set_share_rule_all → items visibles dynamiquement
- set_share_rule_categories → seuls les items des catégories
- set_share_rule_none → rien visible
- private_items_excluded_from_all
- add_item_after_rule_all → item apparaît
- shared_item_count_reflects_rule

## 🔄 À faire — Frontend

### 1. API (APIEndpoint + CircleService)
- `getShareRule(circleId:)` → GET
- `setShareRule(circleId:, mode:, categoryIds:)` → PUT

### 2. ShareWithFriendSheet.swift (CRÉER)
- Radio buttons : Toute ma liste / Par catégories / Sélection / Ne rien partager
- Catégories : charge les catégories de l'user, checkboxes
- Bouton enregistrer → appel API
- Section "déjà partagés" pour mode sélection

### 3. CircleDetailView.swift
- Onglet "Mes envies" : bouton "Gérer le partage" + badge mode actuel
- Empty state : CTA "Partager mes envies"

### 4. CircleDetailViewModel.swift
- Charger la share rule
- State pour le mode actuel

### 5. Localization
- EN + FR pour tous les nouveaux textes
