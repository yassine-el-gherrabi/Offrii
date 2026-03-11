#!/usr/bin/env bash
#
# seed_demo.sh — Complete Offrii demo fixtures
#
# Covers ALL flows:
#   Auth (register/login), Wishlist (CRUD, categories, priorities, claimed),
#   Circles (group, items, claims, activity feed), Friends (accept/pending/sent),
#   Entraide (open/matched/fulfilled/closed, messages, reports), Profile
#
# Usage: bash scripts/seed_demo.sh
#
set -euo pipefail

API="http://localhost:3000"
PSQL="docker exec offrii-postgres psql -U offrii -d offrii -tAc"
PASSWORD="DemoPass123x"

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
RED='\033[0;31m'
NC='\033[0m'

step() { echo -e "\n${CYAN}━━━ $1 ━━━${NC}"; }
ok()   { echo -e "  ${GREEN}✓${NC} $1"; }
warn() { echo -e "  ${YELLOW}⚠${NC} $1"; }
fail() { echo -e "  ${RED}✗${NC} $1"; exit 1; }

# Helper: register user, return token
register() {
  local email="$1" display="$2"
  local resp
  resp=$(curl -s -X POST "$API/auth/register" \
    -H "Content-Type: application/json" \
    -d "{\"email\":\"$email\",\"password\":\"$PASSWORD\",\"display_name\":\"$display\"}")
  local token
  token=$(echo "$resp" | python3 -c "import sys,json; print(json.load(sys.stdin)['tokens']['access_token'])" 2>/dev/null) \
    || fail "Failed to register $email: $resp"
  echo "$token"
}

# Helper: login user, return token
login() {
  local email="$1"
  local resp
  resp=$(curl -s -X POST "$API/auth/login" \
    -H "Content-Type: application/json" \
    -d "{\"email\":\"$email\",\"password\":\"$PASSWORD\"}")
  local token
  token=$(echo "$resp" | python3 -c "import sys,json; print(json.load(sys.stdin)['tokens']['access_token'])" 2>/dev/null) \
    || fail "Failed to login $email: $resp"
  echo "$token"
}

# Helper: authenticated request
auth_post() {
  curl -s -X POST "$API$1" -H "Content-Type: application/json" -H "Authorization: Bearer $2" -d "$3"
}
auth_get() {
  curl -s -X GET "$API$1" -H "Authorization: Bearer $2"
}
auth_patch() {
  curl -s -X PATCH "$API$1" -H "Content-Type: application/json" -H "Authorization: Bearer $2" -d "$3"
}
auth_delete() {
  curl -s -X DELETE "$API$1" -H "Authorization: Bearer $2"
}

# Extract JSON field
jf() { echo "$1" | python3 -c "import sys,json; print(json.load(sys.stdin)$2)" 2>/dev/null; }

# Get a category ID by name for a user
get_cat() {
  local token="$1" name="$2"
  local resp
  resp=$(auth_get "/categories" "$token")
  echo "$resp" | python3 -c "import sys,json; cats=json.load(sys.stdin); print(next(c['id'] for c in cats if c['name']=='$name'))"
}

###############################################################################
step "1/9 — Clean database"
###############################################################################

$PSQL "TRUNCATE wish_messages, wish_reports, community_wishes, circle_events, circle_items, circle_invites, circle_members, circles, friend_requests, friendships, items, share_links, push_tokens, refresh_tokens, users, categories CASCADE;"
ok "All tables truncated"

# Re-seed system default categories (wiped by CASCADE)
$PSQL "
INSERT INTO categories (user_id, name, icon, is_default, position)
VALUES
    (NULL, 'Tech',    'laptop',  TRUE, 1),
    (NULL, 'Mode',    'tshirt',  TRUE, 2),
    (NULL, 'Maison',  'home',    TRUE, 3),
    (NULL, 'Loisirs', 'gamepad', TRUE, 4),
    (NULL, 'Santé',   'heart',   TRUE, 5),
    (NULL, 'Autre',   'tag',     TRUE, 6)
ON CONFLICT DO NOTHING;
"
ok "System default categories re-seeded"

# Ensure community_wishes has missing columns (not in migrations yet)
$PSQL "ALTER TABLE community_wishes ADD COLUMN IF NOT EXISTS image_url TEXT;" 2>/dev/null || true
$PSQL "DO \$\$ BEGIN IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name='community_wishes' AND column_name='links') THEN ALTER TABLE community_wishes ADD COLUMN links TEXT[] DEFAULT '{}'; END IF; END \$\$;" 2>/dev/null || true
ok "Schema patched (image_url, links)"

# Flush Redis
docker exec offrii-redis redis-cli FLUSHALL > /dev/null 2>&1
ok "Redis flushed"

###############################################################################
step "2/9 — Register 8 demo users"
###############################################################################

TOKEN_YASSINE=$(register "yassine@demo.com" "Yassine")
ok "Yassine registered"
TOKEN_MARIE=$(register "marie@demo.com" "Marie Dupont")
ok "Marie registered"
TOKEN_KARIM=$(register "karim@demo.com" "Karim Benali")
ok "Karim registered"
TOKEN_SOPHIE=$(register "sophie@demo.com" "Sophie Martin")
ok "Sophie registered"
TOKEN_LUCAS=$(register "lucas@demo.com" "Lucas Robert")
ok "Lucas registered"
TOKEN_EMMA=$(register "emma@demo.com" "Emma Laurent")
ok "Emma registered"
TOKEN_AHMED=$(register "ahmed@demo.com" "Ahmed Toumi")
ok "Ahmed registered"
TOKEN_CHLOE=$(register "chloe@demo.com" "Chloé Petit")
ok "Chloé registered"

###############################################################################
step "3/9 — Set usernames & profile settings"
###############################################################################

auth_patch "/users/me" "$TOKEN_YASSINE" '{"username":"yassine","reminder_freq":"weekly"}' > /dev/null
ok "Yassine → @yassine, weekly reminders"
auth_patch "/users/me" "$TOKEN_MARIE" '{"username":"marie_d"}' > /dev/null
ok "Marie → @marie_d"
auth_patch "/users/me" "$TOKEN_KARIM" '{"username":"karim_b"}' > /dev/null
ok "Karim → @karim_b"
auth_patch "/users/me" "$TOKEN_SOPHIE" '{"username":"sophie_m"}' > /dev/null
ok "Sophie → @sophie_m"
auth_patch "/users/me" "$TOKEN_LUCAS" '{"username":"lucas_r"}' > /dev/null
ok "Lucas → @lucas_r"
auth_patch "/users/me" "$TOKEN_EMMA" '{"username":"emma_l"}' > /dev/null
ok "Emma → @emma_l"
auth_patch "/users/me" "$TOKEN_AHMED" '{"username":"ahmed_t"}' > /dev/null
ok "Ahmed → @ahmed_t"
auth_patch "/users/me" "$TOKEN_CHLOE" '{"username":"chloe_p"}' > /dev/null
ok "Chloé → @chloe_p"

# Backdate ALL accounts to 3 days ago (needed for community wish creation: MIN_ACCOUNT_AGE_HOURS=24)
$PSQL "UPDATE users SET created_at = NOW() - INTERVAL '3 days';"
ok "All accounts backdated to 3 days ago"

###############################################################################
step "4/9 — Fetch category IDs"
###############################################################################

# Yassine's categories
Y_TECH=$(get_cat "$TOKEN_YASSINE" "Tech")
Y_MODE=$(get_cat "$TOKEN_YASSINE" "Mode")
Y_MAISON=$(get_cat "$TOKEN_YASSINE" "Maison")
Y_LOISIRS=$(get_cat "$TOKEN_YASSINE" "Loisirs")
Y_SANTE=$(get_cat "$TOKEN_YASSINE" "Santé")
ok "Yassine categories fetched"

# Sophie's categories
S_MODE=$(get_cat "$TOKEN_SOPHIE" "Mode")
S_AUTRE=$(get_cat "$TOKEN_SOPHIE" "Autre")
ok "Sophie categories fetched"

# Marie's categories
M_SANTE=$(get_cat "$TOKEN_MARIE" "Santé")
M_MODE=$(get_cat "$TOKEN_MARIE" "Mode")
ok "Marie categories fetched"

###############################################################################
step "5/9 — Create wishlist items"
###############################################################################

# --- Yassine's items (6 active + 2 purchased) ---
RESP=$(auth_post "/items" "$TOKEN_YASSINE" "{\"name\":\"MacBook Pro M4\",\"description\":\"Le nouveau MacBook Pro avec puce M4 Max, 36 Go RAM\",\"url\":\"https://apple.com/macbook-pro\",\"estimated_price\":2999.00,\"priority\":3,\"category_id\":\"$Y_TECH\"}")
ITEM_MACBOOK=$(jf "$RESP" "['id']")
ok "Item: MacBook Pro M4 (Tech, high priority)"

RESP=$(auth_post "/items" "$TOKEN_YASSINE" "{\"name\":\"AirPods Pro 3\",\"description\":\"Écouteurs avec réduction de bruit adaptative\",\"estimated_price\":279.00,\"priority\":2,\"category_id\":\"$Y_TECH\"}")
ITEM_AIRPODS=$(jf "$RESP" "['id']")
ok "Item: AirPods Pro 3 (Tech, medium)"

RESP=$(auth_post "/items" "$TOKEN_YASSINE" "{\"name\":\"Nike Air Max 90\",\"description\":\"Coloris blanc/rouge, taille 43\",\"url\":\"https://nike.com/air-max-90\",\"estimated_price\":149.00,\"priority\":2,\"category_id\":\"$Y_MODE\"}")
ITEM_NIKE=$(jf "$RESP" "['id']")
ok "Item: Nike Air Max 90 (Mode, medium)"

RESP=$(auth_post "/items" "$TOKEN_YASSINE" "{\"name\":\"Lampe de bureau LED\",\"description\":\"Avec variateur et port USB-C\",\"estimated_price\":45.00,\"priority\":1,\"category_id\":\"$Y_MAISON\"}")
ITEM_LAMPE=$(jf "$RESP" "['id']")
ok "Item: Lampe de bureau LED (Maison, low)"

RESP=$(auth_post "/items" "$TOKEN_YASSINE" "{\"name\":\"Zelda Echoes of Wisdom\",\"description\":\"Nintendo Switch\",\"estimated_price\":59.99,\"priority\":3,\"category_id\":\"$Y_LOISIRS\"}")
ITEM_ZELDA=$(jf "$RESP" "['id']")
ok "Item: Zelda Echoes of Wisdom (Loisirs, high)"

RESP=$(auth_post "/items" "$TOKEN_YASSINE" "{\"name\":\"Coffret vitamines\",\"description\":\"Cure 3 mois, vitamines D + magnésium\",\"estimated_price\":35.00,\"priority\":1,\"category_id\":\"$Y_SANTE\"}")
ITEM_VITAMINES=$(jf "$RESP" "['id']")
ok "Item: Coffret vitamines (Santé, low)"

RESP=$(auth_post "/items" "$TOKEN_YASSINE" "{\"name\":\"Pull en cachemire\",\"description\":\"Col roulé bleu marine, taille M\",\"estimated_price\":189.00,\"priority\":2,\"category_id\":\"$Y_MODE\"}")
ITEM_PULL=$(jf "$RESP" "['id']")
ok "Item: Pull en cachemire (Mode, medium) — will be purchased"

RESP=$(auth_post "/items" "$TOKEN_YASSINE" "{\"name\":\"Enceinte Marshall Stanmore III\",\"description\":\"Bluetooth, noir\",\"url\":\"https://marshall.com/stanmore-iii\",\"estimated_price\":349.00,\"priority\":3,\"category_id\":\"$Y_TECH\"}")
ITEM_ENCEINTE=$(jf "$RESP" "['id']")
ok "Item: Enceinte Marshall (Tech, high) — will be purchased"

# Mark 2 items as purchased
$PSQL "UPDATE items SET status = 'purchased' WHERE id IN ('$ITEM_PULL', '$ITEM_ENCEINTE');"
ok "Pull + Enceinte → purchased"

# --- Sophie's items (for circle sharing) ---
RESP=$(auth_post "/items" "$TOKEN_SOPHIE" "{\"name\":\"Livre de cuisine japonaise\",\"description\":\"Recettes authentiques du Japon\",\"estimated_price\":32.00,\"priority\":2,\"category_id\":\"$S_AUTRE\"}")
ITEM_LIVRE=$(jf "$RESP" "['id']")
ok "Sophie: Livre de cuisine japonaise"

RESP=$(auth_post "/items" "$TOKEN_SOPHIE" "{\"name\":\"Écharpe en soie\",\"description\":\"Motif floral, tons pastel\",\"estimated_price\":85.00,\"priority\":2,\"category_id\":\"$S_MODE\"}")
ITEM_ECHARPE=$(jf "$RESP" "['id']")
ok "Sophie: Écharpe en soie"

# --- Marie's items (for circle sharing) ---
RESP=$(auth_post "/items" "$TOKEN_MARIE" "{\"name\":\"Cours de yoga (10 séances)\",\"description\":\"Studio Zen, Paris 11e\",\"estimated_price\":120.00,\"priority\":2,\"category_id\":\"$M_SANTE\"}")
ITEM_YOGA=$(jf "$RESP" "['id']")
ok "Marie: Cours de yoga"

RESP=$(auth_post "/items" "$TOKEN_MARIE" "{\"name\":\"Sac à dos Fjällräven\",\"description\":\"Kånken classique, coloris navy\",\"estimated_price\":95.00,\"priority\":1,\"category_id\":\"$M_MODE\"}")
ITEM_SAC=$(jf "$RESP" "['id']")
ok "Marie: Sac à dos Fjällräven"

###############################################################################
step "6/9 — Friends & friend requests"
###############################################################################

# Yassine ↔ Marie (friends)
RESP=$(auth_post "/me/friend-requests" "$TOKEN_YASSINE" '{"username":"marie_d"}')
REQ_ID=$(jf "$RESP" "['id']")
auth_post "/me/friend-requests/$REQ_ID/accept" "$TOKEN_MARIE" '{}' > /dev/null
ok "Yassine ↔ Marie: friends"

# Yassine ↔ Karim (friends)
RESP=$(auth_post "/me/friend-requests" "$TOKEN_YASSINE" '{"username":"karim_b"}')
REQ_ID=$(jf "$RESP" "['id']")
auth_post "/me/friend-requests/$REQ_ID/accept" "$TOKEN_KARIM" '{}' > /dev/null
ok "Yassine ↔ Karim: friends"

# Yassine ↔ Sophie (friends)
RESP=$(auth_post "/me/friend-requests" "$TOKEN_YASSINE" '{"username":"sophie_m"}')
REQ_ID=$(jf "$RESP" "['id']")
auth_post "/me/friend-requests/$REQ_ID/accept" "$TOKEN_SOPHIE" '{}' > /dev/null
ok "Yassine ↔ Sophie: friends"

# Marie ↔ Karim (friends — needed for Collègues circle)
RESP=$(auth_post "/me/friend-requests" "$TOKEN_MARIE" '{"username":"karim_b"}')
REQ_ID=$(jf "$RESP" "['id']")
auth_post "/me/friend-requests/$REQ_ID/accept" "$TOKEN_KARIM" '{}' > /dev/null
ok "Marie ↔ Karim: friends"

# Lucas → Yassine (pending incoming for Yassine)
auth_post "/me/friend-requests" "$TOKEN_LUCAS" '{"username":"yassine"}' > /dev/null
ok "Lucas → Yassine: pending incoming"

# Yassine → Emma (pending outgoing for Yassine)
auth_post "/me/friend-requests" "$TOKEN_YASSINE" '{"username":"emma_l"}' > /dev/null
ok "Yassine → Emma: pending outgoing"

# Karim ↔ Sophie (friends — for circle dynamics)
RESP=$(auth_post "/me/friend-requests" "$TOKEN_KARIM" '{"username":"sophie_m"}')
REQ_ID=$(jf "$RESP" "['id']")
auth_post "/me/friend-requests/$REQ_ID/accept" "$TOKEN_SOPHIE" '{}' > /dev/null
ok "Karim ↔ Sophie: friends"

###############################################################################
step "7/9 — Circles, shared items, claims & events"
###############################################################################

# Re-login to get fresh tokens (originals may have expired)
TOKEN_YASSINE=$(login "yassine@demo.com")
TOKEN_MARIE=$(login "marie@demo.com")
TOKEN_KARIM=$(login "karim@demo.com")
TOKEN_SOPHIE=$(login "sophie@demo.com")
ok "Fresh tokens obtained"

# --- Circle "Famille" (owner: Yassine) ---
RESP=$(auth_post "/circles" "$TOKEN_YASSINE" '{"name":"Famille"}')
CIRCLE_FAMILLE=$(jf "$RESP" "['id']")
ok "Circle Famille created: $CIRCLE_FAMILLE"

# Get user IDs for adding members
YASSINE_ID=$($PSQL "SELECT id FROM users WHERE email='yassine@demo.com';")
MARIE_ID=$($PSQL "SELECT id FROM users WHERE email='marie@demo.com';")
KARIM_ID=$($PSQL "SELECT id FROM users WHERE email='karim@demo.com';")
SOPHIE_ID=$($PSQL "SELECT id FROM users WHERE email='sophie@demo.com';")
ok "User IDs fetched"

# Add members to Famille
auth_post "/circles/$CIRCLE_FAMILLE/members" "$TOKEN_YASSINE" "{\"user_id\":\"$MARIE_ID\"}" > /dev/null
auth_post "/circles/$CIRCLE_FAMILLE/members" "$TOKEN_YASSINE" "{\"user_id\":\"$KARIM_ID\"}" > /dev/null
auth_post "/circles/$CIRCLE_FAMILLE/members" "$TOKEN_YASSINE" "{\"user_id\":\"$SOPHIE_ID\"}" > /dev/null
ok "Famille members: Yassine, Marie, Karim, Sophie"

# Share Yassine's items to Famille
auth_post "/circles/$CIRCLE_FAMILLE/items" "$TOKEN_YASSINE" "{\"item_id\":\"$ITEM_MACBOOK\"}" > /dev/null
auth_post "/circles/$CIRCLE_FAMILLE/items" "$TOKEN_YASSINE" "{\"item_id\":\"$ITEM_AIRPODS\"}" > /dev/null
auth_post "/circles/$CIRCLE_FAMILLE/items" "$TOKEN_YASSINE" "{\"item_id\":\"$ITEM_ZELDA\"}" > /dev/null
auth_post "/circles/$CIRCLE_FAMILLE/items" "$TOKEN_YASSINE" "{\"item_id\":\"$ITEM_ENCEINTE\"}" > /dev/null
ok "Yassine shared 4 items to Famille"

# Share Sophie's items to Famille
auth_post "/circles/$CIRCLE_FAMILLE/items" "$TOKEN_SOPHIE" "{\"item_id\":\"$ITEM_LIVRE\"}" > /dev/null
auth_post "/circles/$CIRCLE_FAMILLE/items" "$TOKEN_SOPHIE" "{\"item_id\":\"$ITEM_ECHARPE\"}" > /dev/null
ok "Sophie shared 2 items to Famille"

# Share Marie's items to Famille
auth_post "/circles/$CIRCLE_FAMILLE/items" "$TOKEN_MARIE" "{\"item_id\":\"$ITEM_YOGA\"}" > /dev/null
auth_post "/circles/$CIRCLE_FAMILLE/items" "$TOKEN_MARIE" "{\"item_id\":\"$ITEM_SAC\"}" > /dev/null
ok "Marie shared 2 items to Famille"

# Claims in Famille:
# Marie claims Yassine's AirPods (Yassine sees "reserved")
auth_post "/items/$ITEM_AIRPODS/claim" "$TOKEN_MARIE" '{}' > /dev/null
ok "Marie claimed AirPods (Yassine sees 'Réservé')"

# Yassine claims Sophie's Écharpe
auth_post "/items/$ITEM_ECHARPE/claim" "$TOKEN_YASSINE" '{}' > /dev/null
ok "Yassine claimed Sophie's Écharpe"

# Karim claims Marie's Sac
auth_post "/items/$ITEM_SAC/claim" "$TOKEN_KARIM" '{}' > /dev/null
ok "Karim claimed Marie's Sac"

# --- Circle "Collègues" (owner: Marie) ---
RESP=$(auth_post "/circles" "$TOKEN_MARIE" '{"name":"Collègues"}')
CIRCLE_COLLEGUES=$(jf "$RESP" "['id']")
ok "Circle Collègues created: $CIRCLE_COLLEGUES"

auth_post "/circles/$CIRCLE_COLLEGUES/members" "$TOKEN_MARIE" "{\"user_id\":\"$YASSINE_ID\"}" > /dev/null
auth_post "/circles/$CIRCLE_COLLEGUES/members" "$TOKEN_MARIE" "{\"user_id\":\"$KARIM_ID\"}" > /dev/null
ok "Collègues members: Marie, Yassine, Karim"

# Yassine shares Lampe to Collègues
auth_post "/circles/$CIRCLE_COLLEGUES/items" "$TOKEN_YASSINE" "{\"item_id\":\"$ITEM_LAMPE\"}" > /dev/null
ok "Yassine shared Lampe to Collègues"

###############################################################################
step "8/9 — Community wishes (Entraide)"
###############################################################################

# Re-login for fresh tokens
TOKEN_YASSINE=$(login "yassine@demo.com")
TOKEN_MARIE=$(login "marie@demo.com")
TOKEN_KARIM=$(login "karim@demo.com")
TOKEN_SOPHIE=$(login "sophie@demo.com")
TOKEN_AHMED=$(login "ahmed@demo.com")
TOKEN_CHLOE=$(login "chloe@demo.com")
ok "Fresh tokens obtained"

# --- Ahmed's wishes ---
# 1. Open wish (education) — visitors can offer
RESP=$(auth_post "/community/wishes" "$TOKEN_AHMED" '{"title":"Manuels scolaires CP-CE1","description":"Mon fils entre au CP en septembre. Je recherche des manuels de lecture et mathématiques en bon état.","category":"education"}')
WISH_MANUELS=$(jf "$RESP" "['id']")
ok "Ahmed: Manuels scolaires (education, open)"

# 2. Open wish (religion)
RESP=$(auth_post "/community/wishes" "$TOKEN_AHMED" '{"title":"Tapis de prière","description":"Je cherche un tapis de prière pour mon père qui vient d'\''arriver en France.","category":"religion"}')
WISH_TAPIS=$(jf "$RESP" "['id']")
ok "Ahmed: Tapis de prière (religion, open)"

# 3. Matched wish (health) — Yassine offered, messages exchanged
RESP=$(auth_post "/community/wishes" "$TOKEN_AHMED" '{"title":"Fauteuil roulant temporaire","description":"Suite à une opération du genou, j'\''ai besoin d'\''un fauteuil roulant pour 2 mois environ. Prêt ou don.","category":"health"}')
WISH_FAUTEUIL=$(jf "$RESP" "['id']")
ok "Ahmed: Fauteuil roulant (health, will be matched)"

# Yassine offers on fauteuil
auth_post "/community/wishes/$WISH_FAUTEUIL/offer" "$TOKEN_YASSINE" '{}' > /dev/null
ok "Yassine offered on fauteuil → matched"

# Messages between Ahmed & Yassine on fauteuil
auth_post "/community/wishes/$WISH_FAUTEUIL/messages" "$TOKEN_YASSINE" '{"body":"Bonjour Ahmed ! J'\''ai un fauteuil roulant dans mon garage qui ne sert plus. Je peux vous le prêter."}' > /dev/null
auth_post "/community/wishes/$WISH_FAUTEUIL/messages" "$TOKEN_AHMED" '{"body":"Merci infiniment ! C'\''est exactement ce qu'\''il me faut. On peut se retrouver où ?"}' > /dev/null
auth_post "/community/wishes/$WISH_FAUTEUIL/messages" "$TOKEN_YASSINE" '{"body":"Je suis sur Paris 15e. Je peux vous le déposer ce weekend si ça vous va ?"}' > /dev/null
ok "3 messages exchanged on fauteuil wish"

# --- Marie's wish ---
# Open wish (clothing)
RESP=$(auth_post "/community/wishes" "$TOKEN_MARIE" '{"title":"Vêtements d'\''hiver taille M","description":"Je recherche des manteaux et pulls chauds pour l'\''hiver. Taille M, homme ou femme.","category":"clothing"}')
WISH_VETEMENTS=$(jf "$RESP" "['id']")
ok "Marie: Vêtements d'hiver (clothing, open)"

# --- Chloé's wishes ---
# 1. Fulfilled wish (children) — Karim offered & Chloé confirmed
RESP=$(auth_post "/community/wishes" "$TOKEN_CHLOE" '{"title":"Jouets pour enfants 3-5 ans","description":"Ma fille de 4 ans adore les jeux de construction et les puzzles. Tout don est bienvenu !","category":"children"}')
WISH_JOUETS=$(jf "$RESP" "['id']")
ok "Chloé: Jouets enfants (children, will be fulfilled)"

auth_post "/community/wishes/$WISH_JOUETS/offer" "$TOKEN_KARIM" '{}' > /dev/null
auth_post "/community/wishes/$WISH_JOUETS/messages" "$TOKEN_KARIM" '{"body":"J'\''ai plein de Duplo et puzzles que mes enfants n'\''utilisent plus !"}' > /dev/null
auth_post "/community/wishes/$WISH_JOUETS/messages" "$TOKEN_CHLOE" '{"body":"Oh c'\''est génial merci ! Ma fille va être ravie 😊"}' > /dev/null
auth_post "/community/wishes/$WISH_JOUETS/confirm" "$TOKEN_CHLOE" '{}' > /dev/null
ok "Jouets → matched by Karim → fulfilled (2 messages)"

# 2. Open wish (home) — anonymous
RESP=$(auth_post "/community/wishes" "$TOKEN_CHLOE" '{"title":"Petit meuble de rangement","description":"Je cherche une commode ou étagère pour la chambre de ma fille.","category":"home","is_anonymous":true}')
WISH_MEUBLE=$(jf "$RESP" "['id']")
ok "Chloé: Meuble rangement (home, open, anonymous)"

# --- Sophie's wish ---
# Matched by Karim (Sophie can see messages, confirm, reject)
RESP=$(auth_post "/community/wishes" "$TOKEN_SOPHIE" '{"title":"Aide pour déménagement","description":"Je déménage le 20 mars et j'\''aurais besoin d'\''aide pour porter des cartons. Paris 12e → Paris 20e.","category":"other"}')
WISH_DEMENAGEMENT=$(jf "$RESP" "['id']")
ok "Sophie: Aide déménagement (other, will be matched)"

auth_post "/community/wishes/$WISH_DEMENAGEMENT/offer" "$TOKEN_KARIM" '{}' > /dev/null
auth_post "/community/wishes/$WISH_DEMENAGEMENT/messages" "$TOKEN_KARIM" '{"body":"Salut Sophie ! Je suis dispo le 20, j'\''ai un utilitaire si besoin."}' > /dev/null
ok "Sophie's déménagement → matched by Karim (1 message)"

# --- Yassine's community wishes ---
# 1. Matched wish (education) — Karim offered, messages
RESP=$(auth_post "/community/wishes" "$TOKEN_YASSINE" '{"title":"Fournitures scolaires","description":"Mon petit frère entre au collège. On cherche des cahiers, stylos, calculatrice.","category":"education"}')
WISH_FOURNITURES=$(jf "$RESP" "['id']")
ok "Yassine: Fournitures scolaires (education, will be matched)"

auth_post "/community/wishes/$WISH_FOURNITURES/offer" "$TOKEN_KARIM" '{}' > /dev/null
auth_post "/community/wishes/$WISH_FOURNITURES/messages" "$TOKEN_KARIM" '{"body":"J'\''ai récupéré pas mal de fournitures de mes neveux, presque neuves."}' > /dev/null
auth_post "/community/wishes/$WISH_FOURNITURES/messages" "$TOKEN_YASSINE" '{"body":"Super, merci Karim ! On se retrouve quand tu veux."}' > /dev/null
ok "Fournitures → matched by Karim (2 messages)"

# 2. Open wish (clothing)
RESP=$(auth_post "/community/wishes" "$TOKEN_YASSINE" '{"title":"Manteaux chauds pour enfants","description":"Association de quartier cherche des manteaux taille 6-10 ans pour la prochaine collecte.","category":"clothing"}')
WISH_MANTEAUX=$(jf "$RESP" "['id']")
ok "Yassine: Manteaux enfants (clothing, open)"

###############################################################################
step "9/9 — Backdate events for realistic timeline"
###############################################################################

# Spread circle events over the past week for realistic activity feed
$PSQL "
UPDATE circle_events SET created_at = NOW() - (random() * INTERVAL '7 days')
WHERE TRUE;
"
ok "Circle events spread over past 7 days"

# Spread community wish creation dates
$PSQL "
UPDATE community_wishes SET created_at = NOW() - (random() * INTERVAL '5 days')
WHERE TRUE;
"
ok "Community wishes spread over past 5 days"

# Spread messages
$PSQL "
UPDATE wish_messages SET created_at = created_at - (random() * INTERVAL '2 days')
WHERE TRUE;
"
ok "Messages spread over past 2 days"

###############################################################################
echo ""
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}  DEMO FIXTURES LOADED SUCCESSFULLY${NC}"
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo -e "  ${CYAN}Login:${NC}    yassine@demo.com"
echo -e "  ${CYAN}Password:${NC} $PASSWORD"
echo ""
echo -e "  ${YELLOW}What to test:${NC}"
echo ""
echo -e "  ${CYAN}Tab Envies:${NC}"
echo -e "    • 6 active items (Tech, Mode, Maison, Loisirs, Santé)"
echo -e "    • 2 purchased items (Pull, Enceinte)"
echo -e "    • AirPods marked 'Réservé' (claimed by Marie)"
echo -e "    • Filter by category, sort by date/priority/name"
echo -e "    • Swipe left=delete, right=mark received"
echo -e "    • FAB → quick add"
echo ""
echo -e "  ${CYAN}Tab Cercles:${NC}"
echo -e "    • 'Famille' (4 members, 8 shared items, claims, activity feed)"
echo -e "    • 'Collègues' (3 members, 1 shared item)"
echo -e "    • Claim/unclaim items in circles"
echo -e "    • Invite friends to circles"
echo ""
echo -e "  ${CYAN}Tab Entraide:${NC}"
echo -e "    • 5 open wishes to browse (education, clothing, religion, home, other)"
echo -e "    • Filter by category chips"
echo -e "    • Tap wish → offer help"
echo -e "    • 'Mes souhaits': 1 matched (Fournitures, with messages) + 1 open (Manteaux)"
echo -e "    • Fauteuil roulant: Yassine as donor, can send messages or withdraw"
echo ""
echo -e "  ${CYAN}Tab Profil:${NC}"
echo -e "    • Username: @yassine"
echo -e "    • 3 friends (Marie, Karim, Sophie)"
echo -e "    • 1 incoming request (from Lucas) → accept/decline"
echo -e "    • 1 sent request (to Emma) → cancel"
echo -e "    • Community: 2 wishes"
echo -e "    • Reminders: weekly"
echo ""
echo -e "  ${CYAN}Other users to test with:${NC}"
echo -e "    • marie@demo.com  — friends, circle owner (Collègues)"
echo -e "    • karim@demo.com  — donor on multiple wishes"
echo -e "    • sophie@demo.com — has matched wish (déménagement)"
echo -e "    • ahmed@demo.com  — community wish owner"
echo ""
