#!/usr/bin/env bash
#
# seed_demo.sh — Complete Offrii demo fixtures (v2)
#
# Covers 100% of testable scenarios:
#   Auth (12 users, admin, cold-start, OAuth-sim)
#   Wishlist (47 items, pagination, custom categories, edge cases)
#   Friends (13 relations: accepted, pending in/out)
#   Circles (6: group, direct, invite link, empty)
#   Entraide (14 wishes: all statuses, messages, reports, moderation)
#   Share Links (2 public links)
#   Profiles (varied completion %, preferences, timezones, locales)
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
BOLD='\033[1m'
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

# Helper: authenticated requests
auth_post() {
  if [ $# -ge 3 ]; then
    curl -s -X POST "$API$1" -H "Content-Type: application/json" -H "Authorization: Bearer $2" -d "$3"
  else
    curl -s -X POST "$API$1" -H "Content-Type: application/json" -H "Authorization: Bearer $2" -d '{}'
  fi
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

# Make a friend (send request + accept)
make_friends() {
  local sender_token="$1" receiver_username="$2" receiver_token="$3" label="$4"
  local resp req_id
  resp=$(auth_post "/me/friend-requests" "$sender_token" "{\"username\":\"$receiver_username\"}")
  req_id=$(jf "$resp" "['id']")
  auth_post "/me/friend-requests/$req_id/accept" "$receiver_token" '{}' > /dev/null
  ok "$label"
}

###############################################################################
step "1/14 — Clean database"
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

# Ensure community_wishes has optional columns (may not be in migrations yet)
$PSQL "ALTER TABLE community_wishes ADD COLUMN IF NOT EXISTS image_url TEXT;" 2>/dev/null || true
$PSQL "DO \$\$ BEGIN IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name='community_wishes' AND column_name='links') THEN ALTER TABLE community_wishes ADD COLUMN links TEXT[] DEFAULT '{}'; END IF; END \$\$;" 2>/dev/null || true
$PSQL "ALTER TABLE community_wishes ADD COLUMN IF NOT EXISTS reopen_count INTEGER DEFAULT 0;" 2>/dev/null || true
$PSQL "ALTER TABLE community_wishes ADD COLUMN IF NOT EXISTS moderation_note TEXT;" 2>/dev/null || true
ok "Schema patched (image_url, links, reopen_count, moderation_note)"

# Flush Redis
docker exec offrii-redis redis-cli FLUSHALL > /dev/null 2>&1
ok "Redis flushed"

###############################################################################
step "2/14 — Register 12 demo users"
###############################################################################

TOKEN_ADMIN=$(register "admin@demo.com" "Admin Offrii")
ok "Admin registered"
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
TOKEN_OMAR=$(register "omar@demo.com" "Omar Farouq")
ok "Omar registered"
TOKEN_LEILA=$(register "leila@demo.com" "Leila Saadi")
ok "Leila registered"
TOKEN_NEWUSER=$(register "newuser@demo.com" "Nouveau")
ok "Nouveau registered"

###############################################################################
step "3/14 — Set usernames, profiles & preferences"
###############################################################################

auth_patch "/users/me" "$TOKEN_ADMIN" '{"username":"admin"}' > /dev/null
ok "Admin → @admin"
auth_patch "/users/me" "$TOKEN_YASSINE" '{"username":"yassine","reminder_freq":"weekly","timezone":"Europe/Paris","locale":"fr"}' > /dev/null
ok "Yassine → @yassine (weekly, Europe/Paris, fr)"
auth_patch "/users/me" "$TOKEN_MARIE" '{"username":"marie_d","reminder_freq":"daily","timezone":"Europe/Paris","locale":"fr"}' > /dev/null
ok "Marie → @marie_d (daily, Europe/Paris, fr)"
auth_patch "/users/me" "$TOKEN_KARIM" '{"username":"karim_b","reminder_freq":"weekly","timezone":"Europe/Paris","locale":"fr"}' > /dev/null
ok "Karim → @karim_b (weekly, Europe/Paris, fr)"
auth_patch "/users/me" "$TOKEN_SOPHIE" '{"username":"sophie_m"}' > /dev/null
ok "Sophie → @sophie_m"
auth_patch "/users/me" "$TOKEN_LUCAS" '{"username":"lucas_r"}' > /dev/null
ok "Lucas → @lucas_r"
auth_patch "/users/me" "$TOKEN_EMMA" '{"username":"emma_l"}' > /dev/null
ok "Emma → @emma_l"
auth_patch "/users/me" "$TOKEN_AHMED" '{"username":"ahmed_t","reminder_freq":"weekly","timezone":"Africa/Casablanca","locale":"fr"}' > /dev/null
ok "Ahmed → @ahmed_t (weekly, Africa/Casablanca, fr)"
auth_patch "/users/me" "$TOKEN_CHLOE" '{"username":"chloe_p"}' > /dev/null
ok "Chloé → @chloe_p"
auth_patch "/users/me" "$TOKEN_OMAR" '{"username":"omar_f","reminder_freq":"monthly","timezone":"Europe/Paris","locale":"fr"}' > /dev/null
ok "Omar → @omar_f (monthly, Europe/Paris, fr)"
auth_patch "/users/me" "$TOKEN_LEILA" '{"username":"leila_s","reminder_freq":"daily","timezone":"Europe/Paris","locale":"fr"}' > /dev/null
ok "Leila → @leila_s (daily, Europe/Paris, fr)"
auth_patch "/users/me" "$TOKEN_NEWUSER" '{"username":"nouveau"}' > /dev/null
ok "Nouveau → @nouveau (cold-start)"

###############################################################################
step "4/14 — Set admin flag"
###############################################################################

$PSQL "UPDATE users SET is_admin = TRUE WHERE email = 'admin@demo.com';"
ok "admin@demo.com → is_admin = true"

###############################################################################
step "5/14 — Backdate accounts (MIN_ACCOUNT_AGE_HOURS=24)"
###############################################################################

$PSQL "UPDATE users SET created_at = NOW() - INTERVAL '3 days';"
ok "All accounts backdated to 3 days ago"

###############################################################################
step "6/14 — Fetch category IDs"
###############################################################################

# Yassine's categories (all 6)
Y_TECH=$(get_cat "$TOKEN_YASSINE" "Tech")
Y_MODE=$(get_cat "$TOKEN_YASSINE" "Mode")
Y_MAISON=$(get_cat "$TOKEN_YASSINE" "Maison")
Y_LOISIRS=$(get_cat "$TOKEN_YASSINE" "Loisirs")
Y_SANTE=$(get_cat "$TOKEN_YASSINE" "Santé")
ok "Yassine: 5 categories fetched"

# Omar's categories (all 6)
O_TECH=$(get_cat "$TOKEN_OMAR" "Tech")
O_MODE=$(get_cat "$TOKEN_OMAR" "Mode")
O_MAISON=$(get_cat "$TOKEN_OMAR" "Maison")
O_LOISIRS=$(get_cat "$TOKEN_OMAR" "Loisirs")
O_SANTE=$(get_cat "$TOKEN_OMAR" "Santé")
O_AUTRE=$(get_cat "$TOKEN_OMAR" "Autre")
ok "Omar: 6 categories fetched"

# Sophie's categories
S_MODE=$(get_cat "$TOKEN_SOPHIE" "Mode")
S_AUTRE=$(get_cat "$TOKEN_SOPHIE" "Autre")
S_SANTE=$(get_cat "$TOKEN_SOPHIE" "Santé")
ok "Sophie: 3 categories fetched"

# Marie's categories
M_SANTE=$(get_cat "$TOKEN_MARIE" "Santé")
M_MODE=$(get_cat "$TOKEN_MARIE" "Mode")
M_TECH=$(get_cat "$TOKEN_MARIE" "Tech")
ok "Marie: 3 categories fetched"

# Emma's categories
E_TECH=$(get_cat "$TOKEN_EMMA" "Tech")
E_LOISIRS=$(get_cat "$TOKEN_EMMA" "Loisirs")
ok "Emma: 2 categories fetched"

# Lucas's categories
L_TECH=$(get_cat "$TOKEN_LUCAS" "Tech")
L_MAISON=$(get_cat "$TOKEN_LUCAS" "Maison")
ok "Lucas: 2 categories fetched"

# Karim's categories
K_TECH=$(get_cat "$TOKEN_KARIM" "Tech")
K_MAISON=$(get_cat "$TOKEN_KARIM" "Maison")
ok "Karim: 2 categories fetched"

# Leila's default categories
LE_MODE=$(get_cat "$TOKEN_LEILA" "Mode")
LE_SANTE=$(get_cat "$TOKEN_LEILA" "Santé")
ok "Leila: 2 default categories fetched"

# Chloé's categories
C_LOISIRS=$(get_cat "$TOKEN_CHLOE" "Loisirs")
ok "Chloé: 1 category fetched"

###############################################################################
step "7/14 — Create custom categories (Leila)"
###############################################################################

RESP=$(auth_post "/categories" "$TOKEN_LEILA" '{"name":"Artisanat","icon":"scissors"}')
LE_ARTISANAT=$(jf "$RESP" "['id']")
ok "Leila: custom category Artisanat"

RESP=$(auth_post "/categories" "$TOKEN_LEILA" '{"name":"Cuisine","icon":"utensils"}')
LE_CUISINE=$(jf "$RESP" "['id']")
ok "Leila: custom category Cuisine"

###############################################################################
step "8/14 — Create wishlist items (47 items)"
###############################################################################

# ═══════════════════════════════════════════════════════════════
# YASSINE — 8 items (6 active + 2 purchased)
# ═══════════════════════════════════════════════════════════════

RESP=$(auth_post "/items" "$TOKEN_YASSINE" "{\"name\":\"MacBook Pro M4\",\"description\":\"Le nouveau MacBook Pro avec puce M4 Max, 36 Go RAM\",\"url\":\"https://apple.com/macbook-pro\",\"estimated_price\":2999.00,\"priority\":3,\"category_id\":\"$Y_TECH\"}")
ITEM_MACBOOK=$(jf "$RESP" "['id']")
ok "Yassine: MacBook Pro M4 (Tech, p3)"

RESP=$(auth_post "/items" "$TOKEN_YASSINE" "{\"name\":\"AirPods Pro 3\",\"description\":\"Écouteurs avec réduction de bruit adaptative\",\"estimated_price\":279.00,\"priority\":2,\"category_id\":\"$Y_TECH\"}")
ITEM_AIRPODS=$(jf "$RESP" "['id']")
ok "Yassine: AirPods Pro 3 (Tech, p2)"

RESP=$(auth_post "/items" "$TOKEN_YASSINE" "{\"name\":\"Nike Air Max 90\",\"description\":\"Coloris blanc/rouge, taille 43\",\"url\":\"https://nike.com/air-max-90\",\"estimated_price\":149.00,\"priority\":2,\"category_id\":\"$Y_MODE\"}")
ITEM_NIKE=$(jf "$RESP" "['id']")
ok "Yassine: Nike Air Max 90 (Mode, p2)"

RESP=$(auth_post "/items" "$TOKEN_YASSINE" "{\"name\":\"Lampe de bureau LED\",\"description\":\"Avec variateur et port USB-C\",\"estimated_price\":45.00,\"priority\":1,\"category_id\":\"$Y_MAISON\"}")
ITEM_LAMPE=$(jf "$RESP" "['id']")
ok "Yassine: Lampe de bureau LED (Maison, p1)"

RESP=$(auth_post "/items" "$TOKEN_YASSINE" "{\"name\":\"Zelda Echoes of Wisdom\",\"description\":\"Nintendo Switch\",\"estimated_price\":59.99,\"priority\":3,\"category_id\":\"$Y_LOISIRS\"}")
ITEM_ZELDA=$(jf "$RESP" "['id']")
ok "Yassine: Zelda Echoes of Wisdom (Loisirs, p3)"

RESP=$(auth_post "/items" "$TOKEN_YASSINE" "{\"name\":\"Coffret vitamines\",\"description\":\"Cure 3 mois, vitamines D + magnésium\",\"estimated_price\":35.00,\"priority\":1,\"category_id\":\"$Y_SANTE\"}")
ITEM_VITAMINES=$(jf "$RESP" "['id']")
ok "Yassine: Coffret vitamines (Santé, p1)"

RESP=$(auth_post "/items" "$TOKEN_YASSINE" "{\"name\":\"Pull en cachemire\",\"description\":\"Col roulé bleu marine, taille M\",\"estimated_price\":189.00,\"priority\":2,\"category_id\":\"$Y_MODE\"}")
ITEM_PULL=$(jf "$RESP" "['id']")
ok "Yassine: Pull en cachemire (Mode, p2) → purchased"

RESP=$(auth_post "/items" "$TOKEN_YASSINE" "{\"name\":\"Enceinte Marshall Stanmore III\",\"description\":\"Bluetooth, noir\",\"url\":\"https://marshall.com/stanmore-iii\",\"estimated_price\":349.00,\"priority\":3,\"category_id\":\"$Y_TECH\"}")
ITEM_ENCEINTE=$(jf "$RESP" "['id']")
ok "Yassine: Enceinte Marshall (Tech, p3) → purchased"

# Mark 2 items as purchased
$PSQL "UPDATE items SET status = 'purchased' WHERE id IN ('$ITEM_PULL', '$ITEM_ENCEINTE');"
ok "Yassine: Pull + Enceinte → purchased"

# ═══════════════════════════════════════════════════════════════
# OMAR — 22 items (pagination test: 20/page → needs page 2)
# ═══════════════════════════════════════════════════════════════

# Tech (5)
RESP=$(auth_post "/items" "$TOKEN_OMAR" "{\"name\":\"iPhone 15 Pro Max\",\"description\":\"256 Go, Titane naturel\",\"url\":\"https://apple.com/iphone-15-pro\",\"estimated_price\":1299.00,\"priority\":3,\"category_id\":\"$O_TECH\"}")
OMAR_ITEM_1=$(jf "$RESP" "['id']")

RESP=$(auth_post "/items" "$TOKEN_OMAR" "{\"name\":\"iPad Air M2\",\"description\":\"11 pouces, Wi-Fi, 256 Go\",\"url\":\"https://apple.com/ipad-air\",\"estimated_price\":799.00,\"priority\":2,\"category_id\":\"$O_TECH\"}")
OMAR_ITEM_2=$(jf "$RESP" "['id']")

RESP=$(auth_post "/items" "$TOKEN_OMAR" "{\"name\":\"Casque gaming HyperX Cloud III\",\"description\":\"Son surround 7.1, micro amovible\",\"estimated_price\":79.00,\"priority\":1,\"category_id\":\"$O_TECH\"}")
OMAR_ITEM_3=$(jf "$RESP" "['id']")

RESP=$(auth_post "/items" "$TOKEN_OMAR" "{\"name\":\"Hub USB-C Anker 7-en-1\",\"url\":\"https://anker.com/products/usb-c-hub\",\"estimated_price\":45.00,\"priority\":1,\"category_id\":\"$O_TECH\"}")
OMAR_ITEM_4=$(jf "$RESP" "['id']")
# Edge case: no description

RESP=$(auth_post "/items" "$TOKEN_OMAR" "{\"name\":\"Webcam 4K Logitech Brio\",\"description\":\"Streaming et visio haute qualité\",\"url\":\"https://logitech.com/brio\",\"estimated_price\":129.00,\"priority\":2,\"category_id\":\"$O_TECH\"}")
OMAR_ITEM_5=$(jf "$RESP" "['id']")

ok "Omar: 5 Tech items"

# Mode (4)
RESP=$(auth_post "/items" "$TOKEN_OMAR" "{\"name\":\"Sneakers Adidas Ultraboost\",\"description\":\"Taille 44, blanc\",\"url\":\"https://adidas.fr/ultraboost\",\"estimated_price\":189.00,\"priority\":2,\"category_id\":\"$O_MODE\"}")
OMAR_ITEM_6=$(jf "$RESP" "['id']")

RESP=$(auth_post "/items" "$TOKEN_OMAR" "{\"name\":\"Montre Casio G-Shock\",\"description\":\"GA-2100, noire\",\"estimated_price\":99.00,\"priority\":1,\"category_id\":\"$O_MODE\"}")
OMAR_ITEM_7=$(jf "$RESP" "['id']")

RESP=$(auth_post "/items" "$TOKEN_OMAR" "{\"name\":\"Ceinture cuir artisanale\",\"description\":\"Cuir pleine fleur, boucle laiton\",\"priority\":1,\"category_id\":\"$O_MODE\"}")
OMAR_ITEM_8=$(jf "$RESP" "['id']")
# Edge case: no price

RESP=$(auth_post "/items" "$TOKEN_OMAR" "{\"name\":\"Lunettes Ray-Ban Aviator\",\"description\":\"Verres polarisés, monture dorée\",\"url\":\"https://ray-ban.com/aviator\",\"estimated_price\":159.00,\"priority\":2,\"category_id\":\"$O_MODE\"}")
OMAR_ITEM_9=$(jf "$RESP" "['id']")

ok "Omar: 4 Mode items"

# Maison (4)
RESP=$(auth_post "/items" "$TOKEN_OMAR" "{\"name\":\"Machine Nespresso Vertuo Next\",\"description\":\"Avec mousseur à lait\",\"url\":\"https://nespresso.com/vertuo-next\",\"estimated_price\":149.00,\"priority\":3,\"category_id\":\"$O_MAISON\"}")
OMAR_ITEM_10=$(jf "$RESP" "['id']")

RESP=$(auth_post "/items" "$TOKEN_OMAR" "{\"name\":\"Aspirateur robot iRobot Roomba\",\"description\":\"Cartographie laser, vidange automatique\",\"url\":\"https://irobot.com/roomba\",\"estimated_price\":399.00,\"priority\":2,\"category_id\":\"$O_MAISON\"}")
OMAR_ITEM_11=$(jf "$RESP" "['id']")

RESP=$(auth_post "/items" "$TOKEN_OMAR" "{\"name\":\"Coussin ergonomique bureau\",\"description\":\"Mousse à mémoire de forme\",\"estimated_price\":35.00,\"priority\":1,\"category_id\":\"$O_MAISON\"}")
OMAR_ITEM_12=$(jf "$RESP" "['id']")

RESP=$(auth_post "/items" "$TOKEN_OMAR" "{\"name\":\"Bougie Diptyque Baies\",\"estimated_price\":65.00,\"priority\":1,\"category_id\":\"$O_MAISON\"}")
OMAR_ITEM_13=$(jf "$RESP" "['id']")
# Edge case: no description

ok "Omar: 4 Maison items"

# Loisirs (4)
RESP=$(auth_post "/items" "$TOKEN_OMAR" "{\"name\":\"Manette PS5 DualSense\",\"description\":\"Coloris Cosmic Red\",\"url\":\"https://playstation.com/dualsense\",\"estimated_price\":69.00,\"priority\":2,\"category_id\":\"$O_LOISIRS\"}")
OMAR_ITEM_14=$(jf "$RESP" "['id']")

RESP=$(auth_post "/items" "$TOKEN_OMAR" "{\"name\":\"Jeu de société Catan\",\"description\":\"Edition de base + extension 5-6 joueurs\",\"estimated_price\":35.00,\"priority\":1,\"category_id\":\"$O_LOISIRS\"}")
OMAR_ITEM_15=$(jf "$RESP" "['id']")

RESP=$(auth_post "/items" "$TOKEN_OMAR" "{\"name\":\"Dune de Frank Herbert\",\"description\":\"Edition collector reliée\",\"estimated_price\":12.00,\"priority\":1,\"category_id\":\"$O_LOISIRS\"}")
OMAR_ITEM_16=$(jf "$RESP" "['id']")

RESP=$(auth_post "/items" "$TOKEN_OMAR" "{\"name\":\"Puzzle 1000 pièces Van Gogh\",\"description\":\"Nuit étoilée\",\"priority\":1,\"category_id\":\"$O_LOISIRS\"}")
OMAR_ITEM_17=$(jf "$RESP" "['id']")
# Edge case: no price

ok "Omar: 4 Loisirs items"

# Santé (3)
RESP=$(auth_post "/items" "$TOKEN_OMAR" "{\"name\":\"Tapis de yoga Manduka\",\"description\":\"Pro 6mm, noir\",\"estimated_price\":25.00,\"priority\":1,\"category_id\":\"$O_SANTE\"}")
OMAR_ITEM_18=$(jf "$RESP" "['id']")

RESP=$(auth_post "/items" "$TOKEN_OMAR" "{\"name\":\"Gourde isotherme Stanley\",\"description\":\"1.2L, acier inoxydable\",\"url\":\"https://stanley.com/classic-bottle\",\"estimated_price\":30.00,\"priority\":1,\"category_id\":\"$O_SANTE\"}")
OMAR_ITEM_19=$(jf "$RESP" "['id']")

RESP=$(auth_post "/items" "$TOKEN_OMAR" "{\"name\":\"Balance Withings Body+\",\"description\":\"Composition corporelle, Wi-Fi\",\"url\":\"https://withings.com/body-plus\",\"estimated_price\":99.00,\"priority\":2,\"category_id\":\"$O_SANTE\"}")
OMAR_ITEM_20=$(jf "$RESP" "['id']")

ok "Omar: 3 Santé items"

# Autre (2)
RESP=$(auth_post "/items" "$TOKEN_OMAR" "{\"name\":\"Carte cadeau Amazon 50EUR\",\"estimated_price\":50.00,\"priority\":1,\"category_id\":\"$O_AUTRE\"}")
OMAR_ITEM_21=$(jf "$RESP" "['id']")
# Edge case: no description, no URL

RESP=$(auth_post "/items" "$TOKEN_OMAR" "{\"name\":\"Abonnement Spotify Premium\",\"description\":\"12 mois\",\"url\":\"https://spotify.com/premium/offer/annual-subscription-gift-card-2026\",\"estimated_price\":9.99,\"priority\":1,\"category_id\":\"$O_AUTRE\"}")
OMAR_ITEM_22=$(jf "$RESP" "['id']")
# Edge case: long URL

ok "Omar: 2 Autre items — TOTAL: 22 items (pagination test)"

# ═══════════════════════════════════════════════════════════════
# SOPHIE — 3 items (Mode + Autre + Santé)
# ═══════════════════════════════════════════════════════════════

RESP=$(auth_post "/items" "$TOKEN_SOPHIE" "{\"name\":\"Écharpe en soie\",\"description\":\"Motif floral, tons pastel\",\"estimated_price\":85.00,\"priority\":2,\"category_id\":\"$S_MODE\"}")
ITEM_ECHARPE=$(jf "$RESP" "['id']")
ok "Sophie: Écharpe en soie (Mode)"

RESP=$(auth_post "/items" "$TOKEN_SOPHIE" "{\"name\":\"Livre de cuisine japonaise\",\"description\":\"Recettes authentiques du Japon\",\"estimated_price\":32.00,\"priority\":2,\"category_id\":\"$S_AUTRE\"}")
ITEM_LIVRE=$(jf "$RESP" "['id']")
ok "Sophie: Livre de cuisine japonaise (Autre)"

RESP=$(auth_post "/items" "$TOKEN_SOPHIE" "{\"name\":\"Tapis d acupression\",\"description\":\"Soulage le dos et améliore la circulation\",\"estimated_price\":29.00,\"priority\":1,\"category_id\":\"$S_SANTE\"}")
ITEM_ACUPRESSION=$(jf "$RESP" "['id']")
ok "Sophie: Tapis acupression (Santé)"

# ═══════════════════════════════════════════════════════════════
# MARIE — 3 items (Santé + Mode + Tech)
# ═══════════════════════════════════════════════════════════════

RESP=$(auth_post "/items" "$TOKEN_MARIE" "{\"name\":\"Cours de yoga (10 séances)\",\"description\":\"Studio Zen, Paris 11e\",\"estimated_price\":120.00,\"priority\":2,\"category_id\":\"$M_SANTE\"}")
ITEM_YOGA=$(jf "$RESP" "['id']")
ok "Marie: Cours de yoga (Santé)"

RESP=$(auth_post "/items" "$TOKEN_MARIE" "{\"name\":\"Sac à dos Fjällräven\",\"description\":\"Kånken classique, coloris navy\",\"estimated_price\":95.00,\"priority\":1,\"category_id\":\"$M_MODE\"}")
ITEM_SAC=$(jf "$RESP" "['id']")
ok "Marie: Sac à dos Fjällräven (Mode)"

RESP=$(auth_post "/items" "$TOKEN_MARIE" "{\"name\":\"Apple Watch SE\",\"description\":\"40mm, GPS, bracelet sport\",\"url\":\"https://apple.com/apple-watch-se\",\"estimated_price\":249.00,\"priority\":2,\"category_id\":\"$M_TECH\"}")
ITEM_WATCH=$(jf "$RESP" "['id']")
ok "Marie: Apple Watch SE (Tech)"

# ═══════════════════════════════════════════════════════════════
# EMMA — 2 items (Tech + Loisirs)
# ═══════════════════════════════════════════════════════════════

RESP=$(auth_post "/items" "$TOKEN_EMMA" "{\"name\":\"Kindle Paperwhite\",\"description\":\"Liseuse pour lire dans le noir\",\"estimated_price\":139.00,\"priority\":2,\"category_id\":\"$E_TECH\"}")
ITEM_KINDLE=$(jf "$RESP" "['id']")
ok "Emma: Kindle Paperwhite (Tech)"

RESP=$(auth_post "/items" "$TOKEN_EMMA" "{\"name\":\"Cours de poterie\",\"description\":\"Atelier découverte 5 séances\",\"estimated_price\":75.00,\"priority\":1,\"category_id\":\"$E_LOISIRS\"}")
ITEM_POTERIE=$(jf "$RESP" "['id']")
ok "Emma: Cours de poterie (Loisirs)"

# ═══════════════════════════════════════════════════════════════
# LUCAS — 2 items (Tech + Maison)
# ═══════════════════════════════════════════════════════════════

RESP=$(auth_post "/items" "$TOKEN_LUCAS" "{\"name\":\"Clavier mécanique\",\"description\":\"Cherry MX Brown, rétroéclairé RGB\",\"estimated_price\":89.00,\"priority\":2,\"category_id\":\"$L_TECH\"}")
ITEM_CLAVIER=$(jf "$RESP" "['id']")
ok "Lucas: Clavier mécanique (Tech)"

RESP=$(auth_post "/items" "$TOKEN_LUCAS" "{\"name\":\"Plante d intérieur\",\"description\":\"Monstera deliciosa, pot inclus\",\"estimated_price\":35.00,\"priority\":1,\"category_id\":\"$L_MAISON\"}")
ITEM_PLANTE=$(jf "$RESP" "['id']")
ok "Lucas: Plante intérieur (Maison)"

# ═══════════════════════════════════════════════════════════════
# KARIM — 2 items (Tech + Maison)
# ═══════════════════════════════════════════════════════════════

RESP=$(auth_post "/items" "$TOKEN_KARIM" "{\"name\":\"Écran 27 pouces 4K\",\"description\":\"IPS, USB-C, réglable en hauteur\",\"url\":\"https://dell.com/u2723qe\",\"estimated_price\":449.00,\"priority\":2,\"category_id\":\"$K_TECH\"}")
ITEM_ECRAN=$(jf "$RESP" "['id']")
ok "Karim: Écran 27 4K (Tech)"

RESP=$(auth_post "/items" "$TOKEN_KARIM" "{\"name\":\"Kit outils Bosch\",\"description\":\"Perceuse, visseuse, embouts, boîte complète\",\"estimated_price\":129.00,\"priority\":1,\"category_id\":\"$K_MAISON\"}")
ITEM_OUTILS=$(jf "$RESP" "['id']")
ok "Karim: Kit outils Bosch (Maison)"

# ═══════════════════════════════════════════════════════════════
# LEILA — 4 items (2 custom + 2 default)
# ═══════════════════════════════════════════════════════════════

RESP=$(auth_post "/items" "$TOKEN_LEILA" "{\"name\":\"Kit broderie traditionnelle\",\"description\":\"Fils, toile et motifs berbères\",\"estimated_price\":45.00,\"priority\":2,\"category_id\":\"$LE_ARTISANAT\"}")
ITEM_BRODERIE=$(jf "$RESP" "['id']")
ok "Leila: Kit broderie (Artisanat — custom)"

RESP=$(auth_post "/items" "$TOKEN_LEILA" "{\"name\":\"Moule tajine terre cuite\",\"description\":\"Artisanal, diamètre 30cm\",\"estimated_price\":35.00,\"priority\":1,\"category_id\":\"$LE_CUISINE\"}")
ITEM_TAJINE=$(jf "$RESP" "['id']")
ok "Leila: Moule tajine (Cuisine — custom)"

RESP=$(auth_post "/items" "$TOKEN_LEILA" "{\"name\":\"Robe kaftan été\",\"description\":\"Coton léger, motif traditionnel\",\"estimated_price\":65.00,\"priority\":2,\"category_id\":\"$LE_MODE\"}")
ITEM_KAFTAN=$(jf "$RESP" "['id']")
ok "Leila: Robe kaftan (Mode)"

RESP=$(auth_post "/items" "$TOKEN_LEILA" "{\"name\":\"Diffuseur huiles essentielles\",\"description\":\"Ultrasonique, LED, 300ml\",\"estimated_price\":28.00,\"priority\":1,\"category_id\":\"$LE_SANTE\"}")
ITEM_DIFFUSEUR=$(jf "$RESP" "['id']")
ok "Leila: Diffuseur huiles essentielles (Santé)"

# ═══════════════════════════════════════════════════════════════
# CHLOÉ — 1 item (Loisirs)
# ═══════════════════════════════════════════════════════════════

RESP=$(auth_post "/items" "$TOKEN_CHLOE" "{\"name\":\"Appareil photo Instax Mini 12\",\"description\":\"Fujifilm, rose pastel + 20 films\",\"estimated_price\":89.00,\"priority\":1,\"category_id\":\"$C_LOISIRS\"}")
ITEM_INSTAX=$(jf "$RESP" "['id']")
ok "Chloé: Appareil photo Instax (Loisirs)"

echo -e "  ${YELLOW}Total: 47 items created${NC}"

###############################################################################
step "9/14 — Friends & friend requests (13 relations)"
###############################################################################

# Re-login for fresh tokens
TOKEN_YASSINE=$(login "yassine@demo.com")
TOKEN_MARIE=$(login "marie@demo.com")
TOKEN_KARIM=$(login "karim@demo.com")
TOKEN_SOPHIE=$(login "sophie@demo.com")
TOKEN_LUCAS=$(login "lucas@demo.com")
TOKEN_EMMA=$(login "emma@demo.com")
TOKEN_AHMED=$(login "ahmed@demo.com")
TOKEN_CHLOE=$(login "chloe@demo.com")
TOKEN_OMAR=$(login "omar@demo.com")
TOKEN_LEILA=$(login "leila@demo.com")
ok "Fresh tokens obtained"

# ── Accepted friendships ──
make_friends "$TOKEN_YASSINE" "marie_d"  "$TOKEN_MARIE"  "Yassine ↔ Marie: friends"
make_friends "$TOKEN_YASSINE" "karim_b"  "$TOKEN_KARIM"  "Yassine ↔ Karim: friends"
make_friends "$TOKEN_YASSINE" "sophie_m" "$TOKEN_SOPHIE" "Yassine ↔ Sophie: friends"
make_friends "$TOKEN_MARIE"   "karim_b"  "$TOKEN_KARIM"  "Marie ↔ Karim: friends"
make_friends "$TOKEN_KARIM"   "sophie_m" "$TOKEN_SOPHIE" "Karim ↔ Sophie: friends"
make_friends "$TOKEN_OMAR"    "marie_d"  "$TOKEN_MARIE"  "Omar ↔ Marie: friends"
make_friends "$TOKEN_OMAR"    "yassine"  "$TOKEN_YASSINE" "Omar ↔ Yassine: friends"
make_friends "$TOKEN_LEILA"   "sophie_m" "$TOKEN_SOPHIE" "Leila ↔ Sophie: friends"
make_friends "$TOKEN_LEILA"   "marie_d"  "$TOKEN_MARIE"  "Leila ↔ Marie: friends"

# ── Pending requests ──
auth_post "/me/friend-requests" "$TOKEN_LUCAS" '{"username":"yassine"}' > /dev/null
ok "Lucas → Yassine: pending incoming (for Yassine)"

auth_post "/me/friend-requests" "$TOKEN_YASSINE" '{"username":"emma_l"}' > /dev/null
ok "Yassine → Emma: pending outgoing (for Yassine)"

auth_post "/me/friend-requests" "$TOKEN_AHMED" '{"username":"marie_d"}' > /dev/null
ok "Ahmed → Marie: pending incoming (for Marie)"

auth_post "/me/friend-requests" "$TOKEN_CHLOE" '{"username":"omar_f"}' > /dev/null
ok "Chloé → Omar: pending incoming (for Omar)"

###############################################################################
step "10/14 — Circles, members, shared items & claims"
###############################################################################

# Get user IDs
ADMIN_ID=$($PSQL "SELECT id FROM users WHERE email='admin@demo.com';")
YASSINE_ID=$($PSQL "SELECT id FROM users WHERE email='yassine@demo.com';")
MARIE_ID=$($PSQL "SELECT id FROM users WHERE email='marie@demo.com';")
KARIM_ID=$($PSQL "SELECT id FROM users WHERE email='karim@demo.com';")
SOPHIE_ID=$($PSQL "SELECT id FROM users WHERE email='sophie@demo.com';")
OMAR_ID=$($PSQL "SELECT id FROM users WHERE email='omar@demo.com';")
LEILA_ID=$($PSQL "SELECT id FROM users WHERE email='leila@demo.com';")
ok "User IDs fetched"

# ── Circle "Famille" (owner: Yassine) ──
RESP=$(auth_post "/circles" "$TOKEN_YASSINE" '{"name":"Famille"}')
CIRCLE_FAMILLE=$(jf "$RESP" "['id']")

auth_post "/circles/$CIRCLE_FAMILLE/members" "$TOKEN_YASSINE" "{\"user_id\":\"$MARIE_ID\"}" > /dev/null
auth_post "/circles/$CIRCLE_FAMILLE/members" "$TOKEN_YASSINE" "{\"user_id\":\"$KARIM_ID\"}" > /dev/null
auth_post "/circles/$CIRCLE_FAMILLE/members" "$TOKEN_YASSINE" "{\"user_id\":\"$SOPHIE_ID\"}" > /dev/null
ok "Circle Famille: Yassine + Marie + Karim + Sophie"

# Share items to Famille
auth_post "/circles/$CIRCLE_FAMILLE/items" "$TOKEN_YASSINE" "{\"item_id\":\"$ITEM_MACBOOK\"}" > /dev/null
auth_post "/circles/$CIRCLE_FAMILLE/items" "$TOKEN_YASSINE" "{\"item_id\":\"$ITEM_AIRPODS\"}" > /dev/null
auth_post "/circles/$CIRCLE_FAMILLE/items" "$TOKEN_YASSINE" "{\"item_id\":\"$ITEM_ZELDA\"}" > /dev/null
auth_post "/circles/$CIRCLE_FAMILLE/items" "$TOKEN_YASSINE" "{\"item_id\":\"$ITEM_LAMPE\"}" > /dev/null
ok "Yassine shared 4 items to Famille"

auth_post "/circles/$CIRCLE_FAMILLE/items" "$TOKEN_SOPHIE" "{\"item_id\":\"$ITEM_LIVRE\"}" > /dev/null
auth_post "/circles/$CIRCLE_FAMILLE/items" "$TOKEN_SOPHIE" "{\"item_id\":\"$ITEM_ECHARPE\"}" > /dev/null
ok "Sophie shared 2 items to Famille"

auth_post "/circles/$CIRCLE_FAMILLE/items" "$TOKEN_MARIE" "{\"item_id\":\"$ITEM_YOGA\"}" > /dev/null
auth_post "/circles/$CIRCLE_FAMILLE/items" "$TOKEN_MARIE" "{\"item_id\":\"$ITEM_SAC\"}" > /dev/null
ok "Marie shared 2 items to Famille"

# Claims in Famille (3)
auth_post "/items/$ITEM_AIRPODS/claim" "$TOKEN_MARIE" '{}' > /dev/null
ok "Marie claimed AirPods (Yassine sees Réservé)"
auth_post "/items/$ITEM_ECHARPE/claim" "$TOKEN_YASSINE" '{}' > /dev/null
ok "Yassine claimed Sophie's Écharpe"
auth_post "/items/$ITEM_SAC/claim" "$TOKEN_KARIM" '{}' > /dev/null
ok "Karim claimed Marie's Sac"

# ── Circle "Collègues" (owner: Marie) ──
RESP=$(auth_post "/circles" "$TOKEN_MARIE" '{"name":"Collègues"}')
CIRCLE_COLLEGUES=$(jf "$RESP" "['id']")

auth_post "/circles/$CIRCLE_COLLEGUES/members" "$TOKEN_MARIE" "{\"user_id\":\"$YASSINE_ID\"}" > /dev/null
auth_post "/circles/$CIRCLE_COLLEGUES/members" "$TOKEN_MARIE" "{\"user_id\":\"$KARIM_ID\"}" > /dev/null
ok "Circle Collègues: Marie + Yassine + Karim"

auth_post "/circles/$CIRCLE_COLLEGUES/items" "$TOKEN_MARIE" "{\"item_id\":\"$ITEM_WATCH\"}" > /dev/null
auth_post "/circles/$CIRCLE_COLLEGUES/items" "$TOKEN_YASSINE" "{\"item_id\":\"$ITEM_NIKE\"}" > /dev/null
ok "Collègues: Marie shared Watch, Yassine shared Nike"

# ── Circle "Noël 2026" (owner: Omar) ──
RESP=$(auth_post "/circles" "$TOKEN_OMAR" '{"name":"Noël 2026"}')
CIRCLE_NOEL=$(jf "$RESP" "['id']")

auth_post "/circles/$CIRCLE_NOEL/members" "$TOKEN_OMAR" "{\"user_id\":\"$YASSINE_ID\"}" > /dev/null
auth_post "/circles/$CIRCLE_NOEL/members" "$TOKEN_OMAR" "{\"user_id\":\"$MARIE_ID\"}" > /dev/null
auth_post "/circles/$CIRCLE_NOEL/members" "$TOKEN_OMAR" "{\"user_id\":\"$SOPHIE_ID\"}" > /dev/null
auth_post "/circles/$CIRCLE_NOEL/members" "$TOKEN_OMAR" "{\"user_id\":\"$LEILA_ID\"}" > /dev/null
ok "Circle Noël 2026: Omar + Yassine + Marie + Sophie + Leila"

# Omar shares 12 items to Noël 2026
auth_post "/circles/$CIRCLE_NOEL/items" "$TOKEN_OMAR" "{\"item_id\":\"$OMAR_ITEM_1\"}" > /dev/null
auth_post "/circles/$CIRCLE_NOEL/items" "$TOKEN_OMAR" "{\"item_id\":\"$OMAR_ITEM_2\"}" > /dev/null
auth_post "/circles/$CIRCLE_NOEL/items" "$TOKEN_OMAR" "{\"item_id\":\"$OMAR_ITEM_6\"}" > /dev/null
auth_post "/circles/$CIRCLE_NOEL/items" "$TOKEN_OMAR" "{\"item_id\":\"$OMAR_ITEM_9\"}" > /dev/null
auth_post "/circles/$CIRCLE_NOEL/items" "$TOKEN_OMAR" "{\"item_id\":\"$OMAR_ITEM_10\"}" > /dev/null
auth_post "/circles/$CIRCLE_NOEL/items" "$TOKEN_OMAR" "{\"item_id\":\"$OMAR_ITEM_11\"}" > /dev/null
auth_post "/circles/$CIRCLE_NOEL/items" "$TOKEN_OMAR" "{\"item_id\":\"$OMAR_ITEM_14\"}" > /dev/null
auth_post "/circles/$CIRCLE_NOEL/items" "$TOKEN_OMAR" "{\"item_id\":\"$OMAR_ITEM_16\"}" > /dev/null
auth_post "/circles/$CIRCLE_NOEL/items" "$TOKEN_OMAR" "{\"item_id\":\"$OMAR_ITEM_18\"}" > /dev/null
auth_post "/circles/$CIRCLE_NOEL/items" "$TOKEN_OMAR" "{\"item_id\":\"$OMAR_ITEM_20\"}" > /dev/null
auth_post "/circles/$CIRCLE_NOEL/items" "$TOKEN_OMAR" "{\"item_id\":\"$OMAR_ITEM_21\"}" > /dev/null
auth_post "/circles/$CIRCLE_NOEL/items" "$TOKEN_OMAR" "{\"item_id\":\"$OMAR_ITEM_22\"}" > /dev/null
ok "Omar shared 12 items to Noël 2026"

# Claims in Noël 2026 (2)
auth_post "/items/$OMAR_ITEM_1/claim" "$TOKEN_YASSINE" '{}' > /dev/null
ok "Yassine claimed Omar's iPhone 15 Pro"
auth_post "/items/$OMAR_ITEM_10/claim" "$TOKEN_MARIE" '{}' > /dev/null
ok "Marie claimed Omar's Nespresso"

# ── Circle "Projet Asso" (owner: Leila) — empty circle ──
RESP=$(auth_post "/circles" "$TOKEN_LEILA" '{"name":"Projet Asso"}')
CIRCLE_ASSO=$(jf "$RESP" "['id']")

auth_post "/circles/$CIRCLE_ASSO/members" "$TOKEN_LEILA" "{\"user_id\":\"$SOPHIE_ID\"}" > /dev/null
ok "Circle Projet Asso: Leila + Sophie (empty, no items)"

###############################################################################
step "11/14 — Direct circle + circle invite"
###############################################################################

# ── Direct circle: Yassine ↔ Marie ──
auth_post "/circles/direct/$MARIE_ID" "$TOKEN_YASSINE" > /dev/null
ok "Direct circle: Yassine ↔ Marie"

# ── Circle invite on Noël 2026 ──
RESP=$(auth_post "/circles/$CIRCLE_NOEL/invite" "$TOKEN_OMAR" '{"max_uses":5,"expires_in_hours":72}')
INVITE_TOKEN=$(jf "$RESP" "['token']") 2>/dev/null || INVITE_TOKEN="(check response)"
ok "Circle invite on Noël 2026: token=$INVITE_TOKEN (max 5 uses, 72h)"

###############################################################################
step "12/14 — Community wishes (14 wishes, all statuses)"
###############################################################################

# Re-login for fresh tokens
TOKEN_YASSINE=$(login "yassine@demo.com")
TOKEN_MARIE=$(login "marie@demo.com")
TOKEN_KARIM=$(login "karim@demo.com")
TOKEN_SOPHIE=$(login "sophie@demo.com")
TOKEN_AHMED=$(login "ahmed@demo.com")
TOKEN_CHLOE=$(login "chloe@demo.com")
TOKEN_LEILA=$(login "leila@demo.com")
TOKEN_ADMIN=$(login "admin@demo.com")
ok "Fresh tokens obtained"

# ── WISH 1: Manuels scolaires (Ahmed, education, open) ──
RESP=$(auth_post "/community/wishes" "$TOKEN_AHMED" '{"title":"Manuels scolaires CP-CE1","description":"Mon fils entre au CP en septembre. Je recherche des manuels de lecture et mathématiques en bon état.","category":"education"}')
WISH_1=$(jf "$RESP" "['id']")
ok "W1: Ahmed — Manuels scolaires (education, open)"

# ── WISH 2: Tapis de prière (Ahmed, religion, closed) ──
RESP=$(auth_post "/community/wishes" "$TOKEN_AHMED" '{"title":"Tapis de prière","description":"Je cherche un tapis de prière pour mon père qui vient d'\''arriver en France.","category":"religion"}')
WISH_2=$(jf "$RESP" "['id']")
$PSQL "UPDATE community_wishes SET status = 'closed' WHERE id = '$WISH_2';"
ok "W2: Ahmed — Tapis de prière (religion, closed)"

# ── WISH 3: Fauteuil roulant (Ahmed, health, matched — offer Yassine + 3 messages) ──
RESP=$(auth_post "/community/wishes" "$TOKEN_AHMED" '{"title":"Fauteuil roulant temporaire","description":"Suite à une opération du genou, j'\''ai besoin d'\''un fauteuil roulant pour 2 mois environ. Prêt ou don.","category":"health"}')
WISH_3=$(jf "$RESP" "['id']")
auth_post "/community/wishes/$WISH_3/offer" "$TOKEN_YASSINE" > /dev/null
auth_post "/community/wishes/$WISH_3/messages" "$TOKEN_YASSINE" '{"body":"Bonjour Ahmed ! J'\''ai un fauteuil roulant dans mon garage qui ne sert plus. Je peux vous le prêter."}' > /dev/null
auth_post "/community/wishes/$WISH_3/messages" "$TOKEN_AHMED" '{"body":"Merci infiniment ! C'\''est exactement ce qu'\''il me faut. On peut se retrouver où ?"}' > /dev/null
auth_post "/community/wishes/$WISH_3/messages" "$TOKEN_YASSINE" '{"body":"Je suis sur Paris 15e. Je peux vous le déposer ce weekend si ça vous va ?"}' > /dev/null
ok "W3: Ahmed — Fauteuil roulant (health, matched by Yassine, 3 msgs)"

# ── WISH 4: Vêtements d'hiver (Marie, clothing, open) ──
RESP=$(auth_post "/community/wishes" "$TOKEN_MARIE" '{"title":"Vêtements d'\''hiver taille M","description":"Je recherche des manteaux et pulls chauds pour l'\''hiver. Taille M, homme ou femme.","category":"clothing"}')
WISH_4=$(jf "$RESP" "['id']")
ok "W4: Marie — Vêtements d'hiver (clothing, open)"

# ── WISH 5: Jouets enfants (Chloé, children, fulfilled — offer Karim → confirmed + 2 msgs) ──
RESP=$(auth_post "/community/wishes" "$TOKEN_CHLOE" '{"title":"Jouets pour enfants 3-5 ans","description":"Ma fille de 4 ans adore les jeux de construction et les puzzles. Tout don est bienvenu !","category":"children"}')
WISH_5=$(jf "$RESP" "['id']")
auth_post "/community/wishes/$WISH_5/offer" "$TOKEN_KARIM" > /dev/null
auth_post "/community/wishes/$WISH_5/messages" "$TOKEN_KARIM" '{"body":"J'\''ai plein de Duplo et puzzles que mes enfants n'\''utilisent plus !"}' > /dev/null
auth_post "/community/wishes/$WISH_5/messages" "$TOKEN_CHLOE" '{"body":"Oh c'\''est génial merci ! Ma fille va être ravie."}' > /dev/null
auth_post "/community/wishes/$WISH_5/confirm" "$TOKEN_CHLOE" > /dev/null
ok "W5: Chloé — Jouets enfants (children, fulfilled via Karim, 2 msgs)"

# ── WISH 6: Meuble rangement (Chloé, home, open, anonymous + 1 report) ──
RESP=$(auth_post "/community/wishes" "$TOKEN_CHLOE" '{"title":"Petit meuble de rangement","description":"Je cherche une commode ou étagère pour la chambre de ma fille.","category":"home","is_anonymous":true}')
WISH_6=$(jf "$RESP" "['id']")
auth_post "/community/wishes/$WISH_6/report" "$TOKEN_YASSINE" '{"reason":"spam"}' > /dev/null
ok "W6: Chloé — Meuble rangement (home, anonymous, 1 report)"

# ── WISH 7: Aide déménagement (Sophie, other, matched — offer Karim + 1 msg) ──
RESP=$(auth_post "/community/wishes" "$TOKEN_SOPHIE" '{"title":"Aide pour déménagement","description":"Je déménage le 20 mars et j'\''aurais besoin d'\''aide pour porter des cartons. Paris 12e → Paris 20e.","category":"other"}')
WISH_7=$(jf "$RESP" "['id']")
auth_post "/community/wishes/$WISH_7/offer" "$TOKEN_KARIM" > /dev/null
auth_post "/community/wishes/$WISH_7/messages" "$TOKEN_KARIM" '{"body":"Salut Sophie ! Je suis dispo le 20, j'\''ai un utilitaire si besoin."}' > /dev/null
ok "W7: Sophie — Aide déménagement (other, matched by Karim, 1 msg)"

# ── WISH 8: Fournitures scolaires (Yassine, education, matched — offer Karim + 2 msgs) ──
RESP=$(auth_post "/community/wishes" "$TOKEN_YASSINE" '{"title":"Fournitures scolaires","description":"Mon petit frère entre au collège. On cherche des cahiers, stylos, calculatrice.","category":"education"}')
WISH_8=$(jf "$RESP" "['id']")
auth_post "/community/wishes/$WISH_8/offer" "$TOKEN_KARIM" > /dev/null
auth_post "/community/wishes/$WISH_8/messages" "$TOKEN_KARIM" '{"body":"J'\''ai récupéré pas mal de fournitures de mes neveux, presque neuves."}' > /dev/null
auth_post "/community/wishes/$WISH_8/messages" "$TOKEN_YASSINE" '{"body":"Super, merci Karim ! On se retrouve quand tu veux."}' > /dev/null
ok "W8: Yassine — Fournitures scolaires (education, matched by Karim, 2 msgs)"

# ── WISH 9: Manteaux enfants (Yassine, clothing, open) ──
RESP=$(auth_post "/community/wishes" "$TOKEN_YASSINE" '{"title":"Manteaux chauds pour enfants","description":"Association de quartier cherche des manteaux taille 6-10 ans pour la prochaine collecte.","category":"clothing"}')
WISH_9=$(jf "$RESP" "['id']")
ok "W9: Yassine — Manteaux enfants (clothing, open)"

# ── WISH 10: Médicaments courants (Leila, health, open — with image_url + links) ──
RESP=$(auth_post "/community/wishes" "$TOKEN_LEILA" '{"title":"Médicaments courants non périmés","description":"Je cherche du paracétamol, ibuprofène et pansements pour une famille dans le besoin.","category":"health","image_url":"https://images.unsplash.com/photo-1584308666744-24d5c474f2ae?w=400","links":["https://pharmacie-en-ligne.fr/paracetamol","https://ameli.fr/aide-medicaments"]}')
WISH_10=$(jf "$RESP" "['id']")
ok "W10: Leila — Médicaments (health, open, with image + links)"

# ── WISH 11: Spa gratuit suspect (Ahmed, other, rejected by admin + 3 reports) ──
RESP=$(auth_post "/community/wishes" "$TOKEN_AHMED" '{"title":"Spa gratuit illimité","description":"Cherche accès gratuit à un spa de luxe toute l'\''année sans conditions.","category":"other"}')
WISH_11=$(jf "$RESP" "['id']")
# 3 reports from different users
auth_post "/community/wishes/$WISH_11/report" "$TOKEN_YASSINE" '{"reason":"spam"}' > /dev/null
auth_post "/community/wishes/$WISH_11/report" "$TOKEN_MARIE" '{"reason":"inappropriate"}' > /dev/null
auth_post "/community/wishes/$WISH_11/report" "$TOKEN_SOPHIE" '{"reason":"scam"}' > /dev/null
# Flag via SQL (simulates auto-flag from report threshold), then admin rejects
$PSQL "UPDATE community_wishes SET status = 'flagged' WHERE id = '$WISH_11';"
auth_post "/admin/wishes/$WISH_11/reject" "$TOKEN_ADMIN" > /dev/null
# Set moderation note via SQL
$PSQL "UPDATE community_wishes SET moderation_note = 'Rejeté : contenu non conforme aux valeurs d'\''entraide de la plateforme.' WHERE id = '$WISH_11';" 2>/dev/null || true
ok "W11: Ahmed — Spa gratuit (other, 3 reports, flagged → rejected by admin)"

# ── WISH 12: Vélo enfant (Chloé, children, open — reopened after fulfill) ──
RESP=$(auth_post "/community/wishes" "$TOKEN_CHLOE" '{"title":"Vélo enfant 14 pouces","description":"Ma fille a besoin d'\''un vélo pour apprendre, avec petites roues si possible.","category":"children"}')
WISH_12=$(jf "$RESP" "['id']")
# Match → fulfill → reopen
auth_post "/community/wishes/$WISH_12/offer" "$TOKEN_KARIM" > /dev/null
auth_post "/community/wishes/$WISH_12/confirm" "$TOKEN_CHLOE" > /dev/null
auth_post "/community/wishes/$WISH_12/reopen" "$TOKEN_CHLOE" > /dev/null
ok "W12: Chloé — Vélo enfant (children, fulfilled → reopened, reopen_count=1)"

# ── WISH 13: Livres scolaires (Ahmed, education, flagged via high report_count) ──
RESP=$(auth_post "/community/wishes" "$TOKEN_AHMED" '{"title":"Livres scolaires 6ème","description":"Recherche manuels de 6ème pour mon neveu : français, maths, histoire.","category":"education"}')
WISH_13=$(jf "$RESP" "['id']")
# Simulate high report_count + flagged status (auto-flag threshold)
$PSQL "UPDATE community_wishes SET report_count = 5, status = 'flagged' WHERE id = '$WISH_13';"
ok "W13: Ahmed — Livres scolaires (education, report_count=5, flagged)"

# ── WISH 14: Aide ménage (Sophie, home, open — offer by Leila then rejected) ──
RESP=$(auth_post "/community/wishes" "$TOKEN_SOPHIE" '{"title":"Aide ménage ponctuelle","description":"Cherche quelqu'\''un pour m'\''aider à faire le ménage de printemps dans mon appartement.","category":"home"}')
WISH_14=$(jf "$RESP" "['id']")
auth_post "/community/wishes/$WISH_14/offer" "$TOKEN_LEILA" > /dev/null
auth_post "/community/wishes/$WISH_14/reject" "$TOKEN_SOPHIE" > /dev/null
ok "W14: Sophie — Aide ménage (home, offer by Leila → rejected → open)"

echo -e "  ${YELLOW}Total: 14 community wishes created${NC}"

###############################################################################
step "13/14 — Share links"
###############################################################################

# Yassine's share link
RESP=$(auth_post "/share-links" "$TOKEN_YASSINE" '{"label":"Ma wishlist Yassine","scope":"all"}')
SL_YASSINE_ID=$(jf "$RESP" "['id']")
SL_YASSINE_TOKEN=$(jf "$RESP" "['token']")
# Override token for predictable demo access
$PSQL "UPDATE share_links SET token = 'demo-yassine-share' WHERE id = '$SL_YASSINE_ID';" 2>/dev/null || true
ok "Share link: Yassine → token=demo-yassine-share"

# Omar's share link
RESP=$(auth_post "/share-links" "$TOKEN_OMAR" '{"label":"Wishlist Omar","scope":"all"}')
SL_OMAR_ID=$(jf "$RESP" "['id']")
SL_OMAR_TOKEN=$(jf "$RESP" "['token']")
$PSQL "UPDATE share_links SET token = 'demo-omar-share' WHERE id = '$SL_OMAR_ID';" 2>/dev/null || true
ok "Share link: Omar → token=demo-omar-share"

###############################################################################
step "14/14 — Backdate events for realistic timeline"
###############################################################################

# Spread circle events over the past week
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

# Spread friendships
$PSQL "
UPDATE friendships SET created_at = NOW() - (random() * INTERVAL '10 days')
WHERE TRUE;
"
ok "Friendships spread over past 10 days"

# Spread items
$PSQL "
UPDATE items SET created_at = NOW() - (random() * INTERVAL '14 days')
WHERE TRUE;
"
ok "Items spread over past 14 days"

###############################################################################
# SUMMARY OUTPUT
###############################################################################
echo ""
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}  DEMO FIXTURES LOADED SUCCESSFULLY${NC}"
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo -e "  ${BOLD}Password for all accounts:${NC} $PASSWORD"
echo ""
echo -e "  ${YELLOW}━━━ COMPTES & RÔLES ━━━${NC}"
echo ""
echo -e "  ${CYAN}admin@demo.com${NC}    — ${BOLD}Admin${NC} (modération wishes)"
echo -e "  ${CYAN}yassine@demo.com${NC}  — ${BOLD}Power user${NC} (toutes fonctionnalités)"
echo -e "  ${CYAN}marie@demo.com${NC}    — Social active, owner cercle Collègues"
echo -e "  ${CYAN}karim@demo.com${NC}    — Donneur actif (3 offers Entraide)"
echo -e "  ${CYAN}sophie@demo.com${NC}   — Wishes matched, 3 items variés"
echo -e "  ${CYAN}lucas@demo.com${NC}    — Pending friend request → Yassine"
echo -e "  ${CYAN}emma@demo.com${NC}     — Profil semi-complet (~50%)"
echo -e "  ${CYAN}ahmed@demo.com${NC}    — Demandeur Entraide principal"
echo -e "  ${CYAN}chloe@demo.com${NC}    — Poster anonyme, fulfilled wish"
echo -e "  ${CYAN}omar@demo.com${NC}     — ${BOLD}22 items${NC} (test pagination)"
echo -e "  ${CYAN}leila@demo.com${NC}    — Catégories custom (Artisanat, Cuisine)"
echo -e "  ${CYAN}newuser@demo.com${NC}  — ${BOLD}Cold start${NC} (aucune donnée)"
echo ""
echo -e "  ${YELLOW}━━━ ROADMAP DE TEST ━━━${NC}"
echo ""
echo -e "  ${BOLD}ONBOARDING & AUTH${NC}"
echo -e "    → newuser@demo.com : splash → welcome → register → setup"
echo -e "    → yassine@demo.com : login direct → dashboard complet"
echo -e "    → admin@demo.com   : login → accès admin modération"
echo ""
echo -e "  ${BOLD}WISHLIST (ENVIES)${NC}"
echo -e "    → yassine@demo.com : 6 actifs + 2 purchased, toutes catégories, 1 réservé (AirPods)"
echo -e "    → omar@demo.com    : 22 items = test pagination (page 1 + page 2)"
echo -e "    → leila@demo.com   : 2 catégories custom (Artisanat, Cuisine) + 2 default"
echo -e "    → newuser@demo.com : liste vide → empty state"
echo -e "    → Edge cases        : items sans description, sans URL, sans prix"
echo ""
echo -e "  ${BOLD}AMIS${NC}"
echo -e "    → yassine@demo.com : 5 amis + 1 pending in (Lucas) + 1 pending out (Emma)"
echo -e "    → lucas@demo.com   : a envoyé request à Yassine"
echo -e "    → marie@demo.com   : 1 pending in (Ahmed)"
echo -e "    → omar@demo.com    : 1 pending in (Chloé)"
echo -e "    → Search            : chercher \"omar_f\" ou \"leila_s\""
echo ""
echo -e "  ${BOLD}CERCLES${NC}"
echo -e "    → yassine@demo.com : 3 cercles (Famille, Collègues, Noël) + 1 direct (Marie)"
echo -e "    → omar@demo.com    : owner Noël 2026 (12 items, invite link actif)"
echo -e "    → leila@demo.com   : owner Projet Asso (cercle vide)"
echo -e "    → Anti-spoiler       : login Marie → Famille → items Yassine = claim masqué"
echo -e "    → Invite link        : Noël 2026 (max 5 uses, 72h expiry)"
echo ""
echo -e "  ${BOLD}ENTRAIDE${NC}"
echo -e "    → Browse : 5+ wishes open (toutes catégories visibles)"
echo -e "    → ahmed@demo.com  : 2 open + 1 matched + 1 closed + 1 rejected + 1 flagged"
echo -e "    → chloe@demo.com  : 1 fulfilled + 1 anonymous + 1 reopened"
echo -e "    → sophie@demo.com : 1 matched + 1 offre rejetée (wish redevient open)"
echo -e "    → leila@demo.com  : 1 open (avec image + liens)"
echo -e "    → admin@demo.com  : login → 1 rejected (wish #11) + 1 flagged (wish #13)"
echo ""
echo -e "  ${BOLD}SHARE LINKS${NC}"
echo -e "    → GET /shared/demo-yassine-share : wishlist Yassine (6 active)"
echo -e "    → GET /shared/demo-omar-share    : wishlist Omar (22 items)"
echo ""
echo -e "  ${BOLD}PROFILS${NC}"
echo -e "    → yassine@demo.com : ~100% complétion (all fields, items, circles, friends)"
echo -e "    → newuser@demo.com : ~17% complétion → progress card visible"
echo -e "    → Reminder freq     : daily (Marie, Leila), weekly (Yassine, Karim, Ahmed), monthly (Omar)"
echo -e "    → Timezone          : Europe/Paris (most), Africa/Casablanca (Ahmed)"
echo ""
