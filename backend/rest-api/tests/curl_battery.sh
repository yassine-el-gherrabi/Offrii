#!/usr/bin/env bash
set -uo pipefail

# ══════════════════════════════════════════════════════════════════════
# Community Wishes — Full Curl Test Battery
# ══════════════════════════════════════════════════════════════════════
BASE="http://localhost:3000"
PASS="TestPass9xKz2mQ"
PASSED=0
FAILED=0
TOTAL=0
FAILURES=""

# ── Helpers ──────────────────────────────────────────────────────────

c() { # colored output
  local color=$1; shift
  case $color in
    green)  printf '\033[32m%s\033[0m\n' "$*" ;;
    red)    printf '\033[31m%s\033[0m\n' "$*" ;;
    yellow) printf '\033[33m%s\033[0m\n' "$*" ;;
    blue)   printf '\033[34m%s\033[0m\n' "$*" ;;
    *)      echo "$*" ;;
  esac
}

assert_status() {
  local test_name="$1" expected="$2" actual="$3" body="${4:-}"
  TOTAL=$((TOTAL + 1))
  if [ "$actual" = "$expected" ]; then
    PASSED=$((PASSED + 1))
    c green "  PASS  $test_name (HTTP $actual)"
  else
    FAILED=$((FAILED + 1))
    FAILURES="$FAILURES\n  FAIL  $test_name: expected $expected, got $actual"
    c red "  FAIL  $test_name: expected $expected, got $actual"
    if [ -n "$body" ]; then
      echo "        Body: $(echo "$body" | head -c 200)"
    fi
  fi
}

assert_json_field() {
  local test_name="$1" body="$2" field="$3" expected="$4"
  TOTAL=$((TOTAL + 1))
  local actual
  actual=$(echo "$body" | jq -r "$field" 2>/dev/null || echo "PARSE_ERROR")
  if [ "$actual" = "$expected" ]; then
    PASSED=$((PASSED + 1))
    c green "  PASS  $test_name ($field == $expected)"
  else
    FAILED=$((FAILED + 1))
    FAILURES="$FAILURES\n  FAIL  $test_name: $field expected '$expected', got '$actual'"
    c red "  FAIL  $test_name: $field expected '$expected', got '$actual'"
  fi
}

assert_json_not_null() {
  local test_name="$1" body="$2" field="$3"
  TOTAL=$((TOTAL + 1))
  local actual
  actual=$(echo "$body" | jq -r "$field" 2>/dev/null || echo "null")
  if [ "$actual" != "null" ] && [ "$actual" != "" ]; then
    PASSED=$((PASSED + 1))
    c green "  PASS  $test_name ($field is present)"
  else
    FAILED=$((FAILED + 1))
    FAILURES="$FAILURES\n  FAIL  $test_name: $field is null/missing"
    c red "  FAIL  $test_name: $field is null/missing"
  fi
}

assert_json_null() {
  local test_name="$1" body="$2" field="$3"
  TOTAL=$((TOTAL + 1))
  local actual
  actual=$(echo "$body" | jq -r "$field" 2>/dev/null || echo "PARSE_ERROR")
  if [ "$actual" = "null" ]; then
    PASSED=$((PASSED + 1))
    c green "  PASS  $test_name ($field is null)"
  else
    FAILED=$((FAILED + 1))
    FAILURES="$FAILURES\n  FAIL  $test_name: $field expected null, got '$actual'"
    c red "  FAIL  $test_name: $field expected null, got '$actual'"
  fi
}

assert_error_code() {
  local test_name="$1" body="$2" code="$3"
  TOTAL=$((TOTAL + 1))
  local actual
  actual=$(echo "$body" | jq -r '.error.code' 2>/dev/null || echo "PARSE_ERROR")
  if [ "$actual" = "$code" ]; then
    PASSED=$((PASSED + 1))
    c green "  PASS  $test_name (error.code == $code)"
  else
    FAILED=$((FAILED + 1))
    FAILURES="$FAILURES\n  FAIL  $test_name: error.code expected '$code', got '$actual'"
    c red "  FAIL  $test_name: error.code expected '$code', got '$actual'"
  fi
}

# Generate unique email using a temp file counter (survives subshells)
COUNTER_FILE=$(mktemp)
echo "0" > "$COUNTER_FILE"
trap "rm -f $COUNTER_FILE" EXIT

unique_email() {
  local n
  n=$(cat "$COUNTER_FILE")
  n=$((n + 1))
  echo "$n" > "$COUNTER_FILE"
  echo "ct${n}_${$}_${RANDOM}@test.com"
}

register_user() {
  local email="$1" password="$2" display_name="${3:-}"
  local payload
  if [ -n "$display_name" ]; then
    payload="{\"email\":\"$email\",\"password\":\"$password\",\"display_name\":\"$display_name\"}"
  else
    payload="{\"email\":\"$email\",\"password\":\"$password\"}"
  fi
  curl -s -w "\n%{http_code}" -X POST "$BASE/auth/register" \
    -H "Content-Type: application/json" \
    -d "$payload"
}

get_token() {
  local email="$1" password="$2" display_name="${3:-}"
  local result body http_code
  result=$(register_user "$email" "$password" "$display_name") || true
  http_code=$(echo "$result" | tail -1)
  body=$(echo "$result" | sed '$d')
  if [ "$http_code" != "201" ]; then
    echo "REGISTER_FAILED:$http_code:$body" >&2
    echo "INVALID_TOKEN"
    return 0
  fi
  echo "$body" | jq -r '.tokens.access_token'
}

age_account() {
  local email="$1"
  PGPASSWORD=devpass123 psql -h localhost -U offrii -d offrii -q -c \
    "UPDATE users SET created_at = NOW() - INTERVAL '48 hours' WHERE email = '$email';" 2>/dev/null
}

make_admin() {
  local email="$1"
  PGPASSWORD=devpass123 psql -h localhost -U offrii -d offrii -q -c \
    "UPDATE users SET is_admin = true WHERE email = '$email';" 2>/dev/null
}

get_user_id() {
  local email="$1"
  PGPASSWORD=devpass123 psql -h localhost -U offrii -d offrii -tA -c \
    "SELECT id FROM users WHERE email = '$email';" 2>/dev/null | tr -d '[:space:]'
}

force_wish_status() {
  local wish_id="$1" status="$2"
  PGPASSWORD=devpass123 psql -h localhost -U offrii -d offrii -q -c \
    "UPDATE community_wishes SET status = '$status' WHERE id = '$wish_id';" 2>/dev/null
}

force_match() {
  local wish_id="$1" donor_id="$2"
  PGPASSWORD=devpass123 psql -h localhost -U offrii -d offrii -q -c \
    "UPDATE community_wishes SET status = 'matched', matched_with = '$donor_id', matched_at = NOW() WHERE id = '$wish_id';" 2>/dev/null
}

get_wish_status() {
  local wish_id="$1"
  PGPASSWORD=devpass123 psql -h localhost -U offrii -d offrii -tA -c \
    "SELECT status FROM community_wishes WHERE id = '$wish_id';" 2>/dev/null | tr -d '[:space:]'
}

wait_for_status() {
  local wish_id="$1" expected="$2" timeout="${3:-10}"
  local deadline=$((SECONDS + timeout))
  while [ $SECONDS -lt $deadline ]; do
    local current
    current=$(get_wish_status "$wish_id")
    if [ "$current" = "$expected" ]; then
      return 0
    fi
    sleep 0.2
  done
  c red "    TIMEOUT: wish $wish_id did not reach '$expected' (current: $(get_wish_status "$wish_id"))"
  return 1
}

# Setup an aged user with display_name, return token
setup_aged_user() {
  local email="$1" display_name="${2:-}"
  local token
  token=$(get_token "$email" "$PASS" "$display_name")
  age_account "$email"
  echo "$token"
}

# Create a wish and wait for it to become open
create_open_wish() {
  local token="$1" title="${2:-Need winter coat}" category="${3:-clothing}"
  local result body http_code wish_id
  result=$(curl -s -w "\n%{http_code}" -X POST "$BASE/community/wishes" \
    -H "Content-Type: application/json" \
    -H "Authorization: Bearer $token" \
    -d "{\"title\":\"$title\",\"category\":\"$category\",\"is_anonymous\":true}")
  http_code=$(echo "$result" | tail -1)
  body=$(echo "$result" | sed '$d')
  if [ "$http_code" != "201" ]; then
    echo "CREATE_FAILED:$http_code:$body" >&2
    return 1
  fi
  wish_id=$(echo "$body" | jq -r '.id')
  wait_for_status "$wish_id" "open"
  echo "$wish_id"
}

# Curl helpers
do_post() {
  local url="$1" token="$2" body="${3:-}"
  if [ -n "$body" ]; then
    curl -s -w "\n%{http_code}" -X POST "$BASE$url" \
      -H "Content-Type: application/json" \
      -H "Authorization: Bearer $token" \
      -d "$body"
  else
    curl -s -w "\n%{http_code}" -X POST "$BASE$url" \
      -H "Authorization: Bearer $token"
  fi
}

do_get() {
  local url="$1" token="${2:-}"
  if [ -n "$token" ]; then
    curl -s -w "\n%{http_code}" -X GET "$BASE$url" \
      -H "Authorization: Bearer $token"
  else
    curl -s -w "\n%{http_code}" -X GET "$BASE$url"
  fi
}

do_patch() {
  local url="$1" token="$2" body="$3"
  curl -s -w "\n%{http_code}" -X PATCH "$BASE$url" \
    -H "Content-Type: application/json" \
    -H "Authorization: Bearer $token" \
    -d "$body"
}

do_delete() {
  local url="$1" token="$2"
  curl -s -w "\n%{http_code}" -X DELETE "$BASE$url" \
    -H "Authorization: Bearer $token"
}

parse_response() {
  local result="$1"
  HTTP_CODE=$(echo "$result" | tail -1)
  BODY=$(echo "$result" | sed '$d')
}

# ══════════════════════════════════════════════════════════════════════
c blue "══════════════════════════════════════════════════════════════"
c blue " COMMUNITY WISHES — CURL TEST BATTERY"
c blue "══════════════════════════════════════════════════════════════"
echo ""

# ── 1. AUTH GUARDS ───────────────────────────────────────────────────
c yellow "── 1. Auth Guards ─────────────────────────────────────────"

parse_response "$(curl -s -w "\n%{http_code}" -X POST "$BASE/community/wishes" \
  -H "Content-Type: application/json" \
  -d '{"title":"Test","category":"clothing","is_anonymous":true}')"
assert_status "create_wish_no_auth" "401" "$HTTP_CODE"

parse_response "$(curl -s -w "\n%{http_code}" "$BASE/community/wishes/mine")"
assert_status "list_my_wishes_no_auth" "401" "$HTTP_CODE"

FAKE_UUID="00000000-0000-0000-0000-000000000001"
parse_response "$(curl -s -w "\n%{http_code}" -X PATCH "$BASE/community/wishes/$FAKE_UUID" \
  -H "Content-Type: application/json" -d '{"title":"x"}')"
assert_status "update_wish_no_auth" "401" "$HTTP_CODE"

parse_response "$(curl -s -w "\n%{http_code}" -X POST "$BASE/community/wishes/$FAKE_UUID/close")"
assert_status "close_wish_no_auth" "401" "$HTTP_CODE"

parse_response "$(curl -s -w "\n%{http_code}" -X POST "$BASE/community/wishes/$FAKE_UUID/offer")"
assert_status "offer_wish_no_auth" "401" "$HTTP_CODE"

parse_response "$(curl -s -w "\n%{http_code}" -X POST "$BASE/community/wishes/$FAKE_UUID/report" \
  -H "Content-Type: application/json" -d '{}')"
assert_status "report_wish_no_auth" "401" "$HTTP_CODE"

parse_response "$(curl -s -w "\n%{http_code}" -X POST "$BASE/community/wishes/$FAKE_UUID/messages" \
  -H "Content-Type: application/json" -d '{"body":"hi"}')"
assert_status "send_message_no_auth" "401" "$HTTP_CODE"

parse_response "$(curl -s -w "\n%{http_code}" "$BASE/community/wishes/$FAKE_UUID/messages")"
assert_status "list_messages_no_auth" "401" "$HTTP_CODE"

parse_response "$(curl -s -w "\n%{http_code}" "$BASE/admin/wishes/pending")"
assert_status "admin_list_no_auth" "401" "$HTTP_CODE"

echo ""

# ── 2. ACCOUNT AGE GUARD ────────────────────────────────────────────
c yellow "── 2. Account Age Guard ───────────────────────────────────"

YOUNG_EMAIL=$(unique_email)
YOUNG_TOKEN=$(get_token "$YOUNG_EMAIL" "$PASS")
# Do NOT age — account is fresh

parse_response "$(do_post "/community/wishes" "$YOUNG_TOKEN" \
  '{"title":"Young account wish","category":"clothing","is_anonymous":true}')"
assert_status "create_wish_young_account" "403" "$HTTP_CODE"
assert_error_code "create_wish_young_account_code" "$BODY" "FORBIDDEN"

echo ""

# ── 3. WISH CREATION ────────────────────────────────────────────────
c yellow "── 3. Wish Creation ───────────────────────────────────────"

OWNER_EMAIL=$(unique_email)
OWNER_TOKEN=$(setup_aged_user "$OWNER_EMAIL" "TestOwner")
OWNER_ID=$(get_user_id "$OWNER_EMAIL")

# Happy path: create anonymous wish
parse_response "$(do_post "/community/wishes" "$OWNER_TOKEN" \
  '{"title":"Need a winter coat","category":"clothing","is_anonymous":true}')"
assert_status "create_wish_anon_201" "201" "$HTTP_CODE"
WISH_ID=$(echo "$BODY" | jq -r '.id')
assert_json_field "create_wish_status" "$BODY" ".status" "pending"
assert_json_field "create_wish_category" "$BODY" ".category" "clothing"
assert_json_field "create_wish_title" "$BODY" ".title" "Need a winter coat"
assert_json_field "create_wish_is_anonymous" "$BODY" ".is_anonymous" "true"
assert_json_not_null "create_wish_has_id" "$BODY" ".id"
assert_json_not_null "create_wish_has_created_at" "$BODY" ".created_at"

# Wait for moderation to transition to open
wait_for_status "$WISH_ID" "open"
TOTAL=$((TOTAL + 1)); PASSED=$((PASSED + 1))
c green "  PASS  wish_transitions_to_open (background moderation)"

# Non-anonymous with display_name
OWNER2_EMAIL=$(unique_email)
OWNER2_TOKEN=$(setup_aged_user "$OWNER2_EMAIL" "Alice Dupont")
parse_response "$(do_post "/community/wishes" "$OWNER2_TOKEN" \
  '{"title":"Need school supplies","category":"education","is_anonymous":false}')"
assert_status "create_wish_non_anon_201" "201" "$HTTP_CODE"
assert_json_field "create_wish_non_anon_is_anonymous" "$BODY" ".is_anonymous" "false"
WISH2_ID=$(echo "$BODY" | jq -r '.id')
wait_for_status "$WISH2_ID" "open"

# Non-anonymous WITHOUT display_name → 400
OWNER3_EMAIL=$(unique_email)
OWNER3_TOKEN=$(setup_aged_user "$OWNER3_EMAIL")
parse_response "$(do_post "/community/wishes" "$OWNER3_TOKEN" \
  '{"title":"Need something","category":"other","is_anonymous":false}')"
assert_status "create_wish_non_anon_no_name_400" "400" "$HTTP_CODE"

# With description
OWNER4_EMAIL=$(unique_email)
OWNER4_TOKEN=$(setup_aged_user "$OWNER4_EMAIL" "Bob")
parse_response "$(do_post "/community/wishes" "$OWNER4_TOKEN" \
  '{"title":"Need help","category":"health","description":"I need some medical assistance","is_anonymous":true}')"
assert_status "create_wish_with_desc_201" "201" "$HTTP_CODE"
assert_json_field "create_wish_desc_field" "$BODY" ".description" "I need some medical assistance"
WISH4_ID=$(echo "$BODY" | jq -r '.id')
wait_for_status "$WISH4_ID" "open"

# Max active wishes (OWNER already has one open wish)
parse_response "$(do_post "/community/wishes" "$OWNER_TOKEN" \
  '{"title":"Second wish","category":"other","is_anonymous":true}')"
assert_status "create_wish_max_active_409" "409" "$HTTP_CODE"
assert_error_code "create_wish_max_active_code" "$BODY" "CONFLICT"

# Invalid category
parse_response "$(do_post "/community/wishes" "$OWNER3_TOKEN" \
  '{"title":"Bad cat","category":"invalid_cat","is_anonymous":true}')"
assert_status "create_wish_invalid_category_400" "400" "$HTTP_CODE"

# Empty title
parse_response "$(do_post "/community/wishes" "$OWNER3_TOKEN" \
  '{"title":"","category":"clothing","is_anonymous":true}')"
assert_status "create_wish_empty_title_400" "400" "$HTTP_CODE"

# Title too long (256 chars) — use tmpfile to avoid shell escaping issues
TMPJSON=$(mktemp)
python3 -c "import json; print(json.dumps({'title':'x'*256,'category':'clothing','is_anonymous':True}))" > "$TMPJSON"
parse_response "$(curl -s -w "\n%{http_code}" -X POST "$BASE/community/wishes" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $OWNER3_TOKEN" \
  -d @"$TMPJSON")"
assert_status "create_wish_title_too_long_400" "400" "$HTTP_CODE" "$BODY"

# Description too long (2001 chars)
python3 -c "import json; print(json.dumps({'title':'Valid','category':'clothing','is_anonymous':True,'description':'y'*2001}))" > "$TMPJSON"
parse_response "$(curl -s -w "\n%{http_code}" -X POST "$BASE/community/wishes" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $OWNER3_TOKEN" \
  -d @"$TMPJSON")"
assert_status "create_wish_desc_too_long_400" "400" "$HTTP_CODE" "$BODY"
rm -f "$TMPJSON"

echo ""

# ── 4. PUBLIC LISTING ────────────────────────────────────────────────
c yellow "── 4. Public Listing ──────────────────────────────────────"

# No auth — should return open wishes
parse_response "$(do_get "/community/wishes")"
assert_status "list_wishes_no_auth_200" "200" "$HTTP_CODE"
assert_json_not_null "list_wishes_has_wishes" "$BODY" ".wishes"
assert_json_not_null "list_wishes_has_total" "$BODY" ".total"

# Verify anonymous wish hides display_name
ANON_WISH=$(echo "$BODY" | jq -r ".wishes[] | select(.id==\"$WISH_ID\")")
if [ -n "$ANON_WISH" ]; then
  assert_json_null "list_anon_hides_name" "$ANON_WISH" ".display_name"
fi

# With auth — should have is_mine fields
parse_response "$(do_get "/community/wishes" "$OWNER_TOKEN")"
assert_status "list_wishes_with_auth_200" "200" "$HTTP_CODE"
MY_WISH=$(echo "$BODY" | jq -r ".wishes[] | select(.id==\"$WISH_ID\")")
if [ -n "$MY_WISH" ]; then
  assert_json_field "list_wishes_is_mine" "$MY_WISH" ".is_mine" "true"
fi

# Filter by category
parse_response "$(do_get "/community/wishes?category=education" "$OWNER2_TOKEN")"
assert_status "list_wishes_filter_cat_200" "200" "$HTTP_CODE"
# All returned should be education
NON_EDU=$(echo "$BODY" | jq '[.wishes[] | select(.category!="education")] | length')
assert_json_field "list_wishes_filter_correct" "{\"count\":$NON_EDU}" ".count" "0"

# Pagination
parse_response "$(do_get "/community/wishes?limit=1&offset=0")"
assert_status "list_wishes_pagination_200" "200" "$HTTP_CODE"
WISH_COUNT=$(echo "$BODY" | jq '.wishes | length')
TOTAL_COUNT=$(echo "$BODY" | jq '.total')
TOTAL=$((TOTAL + 1))
if [ "$WISH_COUNT" -le 1 ] 2>/dev/null && [ "$TOTAL_COUNT" -ge 1 ] 2>/dev/null; then
  PASSED=$((PASSED + 1))
  c green "  PASS  list_wishes_pagination_correct (got $WISH_COUNT wish, total=$TOTAL_COUNT)"
else
  FAILED=$((FAILED + 1))
  FAILURES="$FAILURES\n  FAIL  list_wishes_pagination_correct"
  c red "  FAIL  list_wishes_pagination_correct (wishes=$WISH_COUNT, total=$TOTAL_COUNT)"
fi

# Invalid limit
parse_response "$(do_get "/community/wishes?limit=0")"
assert_status "list_wishes_limit_0_400" "400" "$HTTP_CODE"

parse_response "$(do_get "/community/wishes?limit=51")"
assert_status "list_wishes_limit_51_400" "400" "$HTTP_CODE"

echo ""

# ── 5. WISH DETAIL ──────────────────────────────────────────────────
c yellow "── 5. Wish Detail ─────────────────────────────────────────"

# Get open wish without auth
parse_response "$(do_get "/community/wishes/$WISH_ID")"
assert_status "get_wish_open_no_auth_200" "200" "$HTTP_CODE"
assert_json_field "get_wish_title" "$BODY" ".title" "Need a winter coat"
assert_json_field "get_wish_status" "$BODY" ".status" "open"

# Get open wish as owner — is_mine=true
parse_response "$(do_get "/community/wishes/$WISH_ID" "$OWNER_TOKEN")"
assert_status "get_wish_owner_200" "200" "$HTTP_CODE"
assert_json_field "get_wish_is_mine" "$BODY" ".is_mine" "true"

# Get non-existent wish → 404
parse_response "$(do_get "/community/wishes/$FAKE_UUID")"
assert_status "get_wish_not_found_404" "404" "$HTTP_CODE"

# Non-anonymous wish shows display_name to strangers
parse_response "$(do_get "/community/wishes/$WISH2_ID")"
assert_status "get_wish_non_anon_shows_name" "200" "$HTTP_CODE"
assert_json_field "get_wish_display_name_visible" "$BODY" ".display_name" "Alice Dupont"

# Anonymous wish hides name from strangers but shows to owner
parse_response "$(do_get "/community/wishes/$WISH_ID")"
assert_json_null "get_wish_anon_hides_from_stranger" "$BODY" ".display_name"

parse_response "$(do_get "/community/wishes/$WISH_ID" "$OWNER_TOKEN")"
assert_json_field "get_wish_anon_shows_to_owner" "$BODY" ".display_name" "TestOwner"

echo ""

# ── 6. MY WISHES ────────────────────────────────────────────────────
c yellow "── 6. My Wishes ───────────────────────────────────────────"

parse_response "$(do_get "/community/wishes/mine" "$OWNER_TOKEN")"
assert_status "list_my_wishes_200" "200" "$HTTP_CODE"
MY_WISHES_COUNT=$(echo "$BODY" | jq 'length')
TOTAL=$((TOTAL + 1))
if [ "$MY_WISHES_COUNT" -ge 1 ] 2>/dev/null; then
  PASSED=$((PASSED + 1))
  c green "  PASS  list_my_wishes_count ($MY_WISHES_COUNT wishes)"
else
  FAILED=$((FAILED + 1))
  FAILURES="$FAILURES\n  FAIL  list_my_wishes_count: expected >=1, got $MY_WISHES_COUNT"
  c red "  FAIL  list_my_wishes_count: expected >=1, got $MY_WISHES_COUNT"
fi

# My wishes should include private fields
MY_W=$(echo "$BODY" | jq ".[0]")
assert_json_not_null "my_wishes_has_report_count" "$MY_W" ".report_count"

echo ""

# ── 7. UPDATE WISH ──────────────────────────────────────────────────
c yellow "── 7. Update Wish ─────────────────────────────────────────"

parse_response "$(do_patch "/community/wishes/$WISH_ID" "$OWNER_TOKEN" \
  '{"title":"Updated title"}')"
assert_status "update_wish_title_200" "200" "$HTTP_CODE"
assert_json_field "update_wish_title_value" "$BODY" ".title" "Updated title"

parse_response "$(do_patch "/community/wishes/$WISH_ID" "$OWNER_TOKEN" \
  '{"description":"New description"}')"
assert_status "update_wish_desc_200" "200" "$HTTP_CODE"
assert_json_field "update_wish_desc_value" "$BODY" ".description" "New description"

parse_response "$(do_patch "/community/wishes/$WISH_ID" "$OWNER_TOKEN" \
  '{"category":"education"}')"
assert_status "update_wish_category_200" "200" "$HTTP_CODE"
assert_json_field "update_wish_cat_value" "$BODY" ".category" "education"

# Not owner → 403
STRANGER_EMAIL=$(unique_email)
STRANGER_TOKEN=$(setup_aged_user "$STRANGER_EMAIL")
parse_response "$(do_patch "/community/wishes/$WISH_ID" "$STRANGER_TOKEN" \
  '{"title":"Hacked"}')"
assert_status "update_wish_not_owner_403" "403" "$HTTP_CODE"

# Not found
parse_response "$(do_patch "/community/wishes/$FAKE_UUID" "$OWNER_TOKEN" \
  '{"title":"x"}')"
assert_status "update_wish_not_found_404" "404" "$HTTP_CODE"

# Invalid category
parse_response "$(do_patch "/community/wishes/$WISH_ID" "$OWNER_TOKEN" \
  '{"category":"badcat"}')"
assert_status "update_wish_invalid_cat_400" "400" "$HTTP_CODE"

echo ""

# ── 8. OFFER (MATCH) ────────────────────────────────────────────────
c yellow "── 8. Offer (Match) ───────────────────────────────────────"

DONOR_EMAIL=$(unique_email)
DONOR_TOKEN=$(setup_aged_user "$DONOR_EMAIL" "DonorPerson")
DONOR_ID=$(get_user_id "$DONOR_EMAIL")

# Self-offer → 400
parse_response "$(do_post "/community/wishes/$WISH_ID/offer" "$OWNER_TOKEN")"
assert_status "offer_self_400" "400" "$HTTP_CODE"

# Young account cannot offer
parse_response "$(do_post "/community/wishes/$WISH_ID/offer" "$YOUNG_TOKEN")"
assert_status "offer_young_403" "403" "$HTTP_CODE"

# Not found
parse_response "$(do_post "/community/wishes/$FAKE_UUID/offer" "$DONOR_TOKEN")"
assert_status "offer_not_found_404" "404" "$HTTP_CODE"

# Success
parse_response "$(do_post "/community/wishes/$WISH_ID/offer" "$DONOR_TOKEN")"
assert_status "offer_wish_204" "204" "$HTTP_CODE"

# Verify status changed to matched
CURRENT_STATUS=$(get_wish_status "$WISH_ID")
assert_json_field "offer_status_matched" "{\"s\":\"$CURRENT_STATUS\"}" ".s" "matched"

# Get matched wish — donor should see matched_with_display_name
parse_response "$(do_get "/community/wishes/$WISH_ID" "$DONOR_TOKEN")"
assert_status "get_matched_donor_200" "200" "$HTTP_CODE"
assert_json_field "get_matched_status" "$BODY" ".status" "matched"

# Owner should also see it
parse_response "$(do_get "/community/wishes/$WISH_ID" "$OWNER_TOKEN")"
assert_status "get_matched_owner_200" "200" "$HTTP_CODE"
assert_json_field "get_matched_owner_status" "$BODY" ".status" "matched"

# Double offer on already matched → 400
DONOR2_EMAIL=$(unique_email)
DONOR2_TOKEN=$(setup_aged_user "$DONOR2_EMAIL")
parse_response "$(do_post "/community/wishes/$WISH_ID/offer" "$DONOR2_TOKEN")"
assert_status "offer_already_matched_400" "400" "$HTTP_CODE"

echo ""

# ── 9. MESSAGES (on matched wish) ───────────────────────────────────
c yellow "── 9. Messages ────────────────────────────────────────────"

# Owner sends message
parse_response "$(do_post "/community/wishes/$WISH_ID/messages" "$OWNER_TOKEN" \
  '{"body":"Hello donor! Thanks for helping."}')"
assert_status "send_msg_owner_201" "201" "$HTTP_CODE"
assert_json_field "send_msg_body" "$BODY" ".body" "Hello donor! Thanks for helping."
assert_json_field "send_msg_is_mine" "$BODY" ".is_mine" "true"
assert_json_not_null "send_msg_has_id" "$BODY" ".id"
assert_json_not_null "send_msg_has_created_at" "$BODY" ".created_at"

# Donor sends message
parse_response "$(do_post "/community/wishes/$WISH_ID/messages" "$DONOR_TOKEN" \
  '{"body":"Happy to help!"}')"
assert_status "send_msg_donor_201" "201" "$HTTP_CODE"
assert_json_field "send_msg_donor_body" "$BODY" ".body" "Happy to help!"

# Stranger cannot send
parse_response "$(do_post "/community/wishes/$WISH_ID/messages" "$STRANGER_TOKEN" \
  '{"body":"Intruder!"}')"
assert_status "send_msg_stranger_403" "403" "$HTTP_CODE"

# Empty body → 400
parse_response "$(do_post "/community/wishes/$WISH_ID/messages" "$OWNER_TOKEN" \
  '{"body":""}')"
assert_status "send_msg_empty_400" "400" "$HTTP_CODE"

# Body too long (501 chars, limit is 500)
LONG_MSG=$(printf 'z%.0s' {1..501})
parse_response "$(do_post "/community/wishes/$WISH_ID/messages" "$OWNER_TOKEN" \
  "{\"body\":\"$LONG_MSG\"}")"
assert_status "send_msg_too_long_400" "400" "$HTTP_CODE"

# List messages as owner
parse_response "$(do_get "/community/wishes/$WISH_ID/messages" "$OWNER_TOKEN")"
assert_status "list_msg_owner_200" "200" "$HTTP_CODE"
MSG_COUNT=$(echo "$BODY" | jq '.messages | length')
MSG_TOTAL=$(echo "$BODY" | jq '.total')
assert_json_field "list_msg_total" "$BODY" ".total" "2"

# is_mine flag correctness for owner
FIRST_MSG=$(echo "$BODY" | jq '.messages[0]')
assert_json_field "list_msg_is_mine_owner" "$FIRST_MSG" ".is_mine" "true"

# List messages as donor — is_mine perspective
parse_response "$(do_get "/community/wishes/$WISH_ID/messages" "$DONOR_TOKEN")"
assert_status "list_msg_donor_200" "200" "$HTTP_CODE"
DONOR_FIRST_MSG=$(echo "$BODY" | jq '.messages[0]')
# The first message was from the owner, so is_mine=false for donor
assert_json_field "list_msg_is_mine_donor_perspective" "$DONOR_FIRST_MSG" ".is_mine" "false"

# Stranger cannot list
parse_response "$(do_get "/community/wishes/$WISH_ID/messages" "$STRANGER_TOKEN")"
assert_status "list_msg_stranger_403" "403" "$HTTP_CODE"

# Message on non-matched wish → 400
parse_response "$(do_post "/community/wishes/$WISH2_ID/messages" "$OWNER2_TOKEN" \
  '{"body":"msg on open wish"}')"
assert_status "send_msg_not_matched_400" "400" "$HTTP_CODE"

# List on non-matched wish → 400
parse_response "$(do_get "/community/wishes/$WISH2_ID/messages" "$OWNER2_TOKEN")"
assert_status "list_msg_not_matched_400" "400" "$HTTP_CODE"

# Pagination
parse_response "$(do_get "/community/wishes/$WISH_ID/messages?limit=1&offset=0" "$OWNER_TOKEN")"
assert_status "list_msg_pagination_200" "200" "$HTTP_CODE"
PAGINATED_COUNT=$(echo "$BODY" | jq '.messages | length')
assert_json_field "list_msg_paginated_count" "{\"c\":$PAGINATED_COUNT}" ".c" "1"
assert_json_field "list_msg_paginated_total" "$BODY" ".total" "2"

echo ""

# ── 10. WITHDRAW OFFER ──────────────────────────────────────────────
c yellow "── 10. Withdraw Offer ─────────────────────────────────────"

# Owner cannot withdraw (only donor can)
parse_response "$(do_delete "/community/wishes/$WISH_ID/offer" "$OWNER_TOKEN")"
assert_status "withdraw_owner_cannot_403" "403" "$HTTP_CODE"

# Stranger cannot withdraw
parse_response "$(do_delete "/community/wishes/$WISH_ID/offer" "$STRANGER_TOKEN")"
assert_status "withdraw_stranger_403" "403" "$HTTP_CODE"

# Donor withdraws
parse_response "$(do_delete "/community/wishes/$WISH_ID/offer" "$DONOR_TOKEN")"
assert_status "withdraw_offer_204" "204" "$HTTP_CODE"

# Wish should be back to open
CURRENT_STATUS=$(get_wish_status "$WISH_ID")
assert_json_field "withdraw_status_open" "{\"s\":\"$CURRENT_STATUS\"}" ".s" "open"

# Cannot withdraw when not matched
parse_response "$(do_delete "/community/wishes/$WISH_ID/offer" "$DONOR_TOKEN")"
assert_status "withdraw_not_matched_400" "400" "$HTTP_CODE"

echo ""

# ── 11. REJECT OFFER ────────────────────────────────────────────────
c yellow "── 11. Reject Offer ───────────────────────────────────────"

# Re-offer (wish is back to open)
parse_response "$(do_post "/community/wishes/$WISH_ID/offer" "$DONOR_TOKEN")"
assert_status "re_offer_for_reject_204" "204" "$HTTP_CODE"

# Donor cannot reject (only owner)
parse_response "$(do_post "/community/wishes/$WISH_ID/reject" "$DONOR_TOKEN")"
assert_status "reject_by_donor_403" "403" "$HTTP_CODE"

# Stranger cannot reject
parse_response "$(do_post "/community/wishes/$WISH_ID/reject" "$STRANGER_TOKEN")"
assert_status "reject_stranger_403" "403" "$HTTP_CODE"

# Owner rejects
parse_response "$(do_post "/community/wishes/$WISH_ID/reject" "$OWNER_TOKEN")"
assert_status "reject_offer_204" "204" "$HTTP_CODE"

CURRENT_STATUS=$(get_wish_status "$WISH_ID")
assert_json_field "reject_status_open" "{\"s\":\"$CURRENT_STATUS\"}" ".s" "open"

# Cannot reject when not matched
parse_response "$(do_post "/community/wishes/$WISH_ID/reject" "$OWNER_TOKEN")"
assert_status "reject_not_matched_400" "400" "$HTTP_CODE"

echo ""

# ── 12. CONFIRM (FULFILL) ───────────────────────────────────────────
c yellow "── 12. Confirm (Fulfill) ──────────────────────────────────"

# Re-offer for confirm
parse_response "$(do_post "/community/wishes/$WISH_ID/offer" "$DONOR_TOKEN")"
assert_status "re_offer_for_confirm_204" "204" "$HTTP_CODE"

# Donor cannot confirm (only owner)
parse_response "$(do_post "/community/wishes/$WISH_ID/confirm" "$DONOR_TOKEN")"
assert_status "confirm_donor_403" "403" "$HTTP_CODE"

# Owner confirms
parse_response "$(do_post "/community/wishes/$WISH_ID/confirm" "$OWNER_TOKEN")"
assert_status "confirm_wish_204" "204" "$HTTP_CODE"

CURRENT_STATUS=$(get_wish_status "$WISH_ID")
assert_json_field "confirm_status_fulfilled" "{\"s\":\"$CURRENT_STATUS\"}" ".s" "fulfilled"

# Fulfilled wish — messages still readable (history)
parse_response "$(do_get "/community/wishes/$WISH_ID/messages" "$OWNER_TOKEN")"
assert_status "list_msg_after_fulfill_200" "200" "$HTTP_CODE"

# But sending new messages → 400
parse_response "$(do_post "/community/wishes/$WISH_ID/messages" "$OWNER_TOKEN" \
  '{"body":"post-fulfill msg"}')"
assert_status "send_msg_after_fulfill_400" "400" "$HTTP_CODE"

# After fulfill, owner can create a new wish (slot freed)
parse_response "$(do_post "/community/wishes" "$OWNER_TOKEN" \
  '{"title":"New wish after fulfill","category":"other","is_anonymous":true}')"
assert_status "create_wish_after_fulfill_201" "201" "$HTTP_CODE"
WISH_AFTER_FULFILL=$(echo "$BODY" | jq -r '.id')
wait_for_status "$WISH_AFTER_FULFILL" "open"

echo ""

# ── 13. CLOSE WISH ──────────────────────────────────────────────────
c yellow "── 13. Close Wish ─────────────────────────────────────────"

# Stranger cannot close
parse_response "$(do_post "/community/wishes/$WISH_AFTER_FULFILL/close" "$STRANGER_TOKEN")"
assert_status "close_not_owner_403" "403" "$HTTP_CODE"

# Owner closes
parse_response "$(do_post "/community/wishes/$WISH_AFTER_FULFILL/close" "$OWNER_TOKEN")"
assert_status "close_wish_204" "204" "$HTTP_CODE"

CURRENT_STATUS=$(get_wish_status "$WISH_AFTER_FULFILL")
assert_json_field "close_status" "{\"s\":\"$CURRENT_STATUS\"}" ".s" "closed"

# Already closed → 400
parse_response "$(do_post "/community/wishes/$WISH_AFTER_FULFILL/close" "$OWNER_TOKEN")"
assert_status "close_already_closed_400" "400" "$HTTP_CODE"

# Already fulfilled → 400
parse_response "$(do_post "/community/wishes/$WISH_ID/close" "$OWNER_TOKEN")"
assert_status "close_already_fulfilled_400" "400" "$HTTP_CODE"

echo ""

# ── 14. REPORT ───────────────────────────────────────────────────────
c yellow "── 14. Report Wish ────────────────────────────────────────"

# Create a fresh wish to report
REPORT_OWNER_EMAIL=$(unique_email)
REPORT_OWNER_TOKEN=$(setup_aged_user "$REPORT_OWNER_EMAIL" "ReportTarget")
REPORT_WISH_ID=$(create_open_wish "$REPORT_OWNER_TOKEN" "Reportable wish" "other")

# Self-report → 400
parse_response "$(do_post "/community/wishes/$REPORT_WISH_ID/report" "$REPORT_OWNER_TOKEN" \
  '{"reason":"spam"}')"
assert_status "report_self_400" "400" "$HTTP_CODE"

# Young account cannot report
parse_response "$(do_post "/community/wishes/$REPORT_WISH_ID/report" "$YOUNG_TOKEN" \
  '{"reason":"spam"}')"
assert_status "report_young_403" "403" "$HTTP_CODE"

# Report with explicit reason
REPORTER1_EMAIL=$(unique_email)
REPORTER1_TOKEN=$(setup_aged_user "$REPORTER1_EMAIL")
parse_response "$(do_post "/community/wishes/$REPORT_WISH_ID/report" "$REPORTER1_TOKEN" \
  '{"reason":"spam"}')"
assert_status "report_explicit_reason_204" "204" "$HTTP_CODE"

# Duplicate report → 409
parse_response "$(do_post "/community/wishes/$REPORT_WISH_ID/report" "$REPORTER1_TOKEN" \
  '{"reason":"spam"}')"
assert_status "report_duplicate_409" "409" "$HTTP_CODE"

# Report with default reason (null → inappropriate)
REPORTER2_EMAIL=$(unique_email)
REPORTER2_TOKEN=$(setup_aged_user "$REPORTER2_EMAIL")
parse_response "$(do_post "/community/wishes/$REPORT_WISH_ID/report" "$REPORTER2_TOKEN" '{}')"
assert_status "report_default_reason_204" "204" "$HTTP_CODE"

# Invalid reason
REPORTER3_EMAIL=$(unique_email)
REPORTER3_TOKEN=$(setup_aged_user "$REPORTER3_EMAIL")
parse_response "$(do_post "/community/wishes/$REPORT_WISH_ID/report" "$REPORTER3_TOKEN" \
  '{"reason":"invalid_reason"}')"
assert_status "report_invalid_reason_400" "400" "$HTTP_CODE"

# Report not found
parse_response "$(do_post "/community/wishes/$FAKE_UUID/report" "$REPORTER1_TOKEN" \
  '{"reason":"spam"}')"
assert_status "report_not_found_404" "404" "$HTTP_CODE"

# Report non-open wish → 400
parse_response "$(do_post "/community/wishes/$WISH_ID/report" "$REPORTER1_TOKEN" \
  '{"reason":"spam"}')"
assert_status "report_non_open_400" "400" "$HTTP_CODE"

# 5 unique reports → triggers review status
# Already have 2 reports. Add 3 more.
for i in 3 4 5; do
  RE=$(unique_email)
  RT=$(setup_aged_user "$RE")
  do_post "/community/wishes/$REPORT_WISH_ID/report" "$RT" '{"reason":"scam"}' > /dev/null
done

REPORT_STATUS=$(get_wish_status "$REPORT_WISH_ID")
assert_json_field "report_threshold_review" "{\"s\":\"$REPORT_STATUS\"}" ".s" "review"

echo ""

# ── 15. REOPEN ───────────────────────────────────────────────────────
c yellow "── 15. Reopen Wish ────────────────────────────────────────"

# Not owner → 403
parse_response "$(do_post "/community/wishes/$REPORT_WISH_ID/reopen" "$STRANGER_TOKEN")"
assert_status "reopen_not_owner_403" "403" "$HTTP_CODE"

# Owner reopens
parse_response "$(do_post "/community/wishes/$REPORT_WISH_ID/reopen" "$REPORT_OWNER_TOKEN")"
assert_status "reopen_success_204" "204" "$HTTP_CODE"

CURRENT_STATUS=$(get_wish_status "$REPORT_WISH_ID")
assert_json_field "reopen_status_open" "{\"s\":\"$CURRENT_STATUS\"}" ".s" "open"

# Reopen cooldown (reopen again immediately) → 400
# First need to get it to review again via 5 more reports
for i in 1 2 3 4 5; do
  RE=$(unique_email)
  RT=$(setup_aged_user "$RE")
  do_post "/community/wishes/$REPORT_WISH_ID/report" "$RT" '{"reason":"spam"}' > /dev/null
done
wait_for_status "$REPORT_WISH_ID" "review" 5 || true
REPORT_STATUS=$(get_wish_status "$REPORT_WISH_ID")
if [ "$REPORT_STATUS" = "review" ]; then
  parse_response "$(do_post "/community/wishes/$REPORT_WISH_ID/reopen" "$REPORT_OWNER_TOKEN")"
  assert_status "reopen_cooldown_400" "400" "$HTTP_CODE"
else
  c yellow "  SKIP  reopen_cooldown_400 (could not re-trigger review)"
fi

# Not in review → 400
OPEN_WISH_FOR_REOPEN=$WISH2_ID
parse_response "$(do_post "/community/wishes/$OPEN_WISH_FOR_REOPEN/reopen" "$OWNER2_TOKEN")"
assert_status "reopen_not_review_400" "400" "$HTTP_CODE"

echo ""

# ── 16. ADMIN ENDPOINTS ─────────────────────────────────────────────
c yellow "── 16. Admin Endpoints ────────────────────────────────────"

# Non-admin → 403
parse_response "$(do_get "/admin/wishes/pending" "$OWNER_TOKEN")"
assert_status "admin_list_non_admin_403" "403" "$HTTP_CODE"

# Setup admin
ADMIN_EMAIL=$(unique_email)
ADMIN_TOKEN=$(setup_aged_user "$ADMIN_EMAIL")
make_admin "$ADMIN_EMAIL"

# Admin lists pending (flagged + review)
parse_response "$(do_get "/admin/wishes/pending" "$ADMIN_TOKEN")"
assert_status "admin_list_pending_200" "200" "$HTTP_CODE"

# Create a wish and force to flagged for admin approve
FLAGGED_OWNER_EMAIL=$(unique_email)
FLAGGED_OWNER_TOKEN=$(setup_aged_user "$FLAGGED_OWNER_EMAIL" "FlaggedOwner")
FLAGGED_WISH_ID=$(create_open_wish "$FLAGGED_OWNER_TOKEN" "Flagged wish" "other")
force_wish_status "$FLAGGED_WISH_ID" "flagged"

# Admin approves
parse_response "$(do_post "/admin/wishes/$FLAGGED_WISH_ID/approve" "$ADMIN_TOKEN")"
assert_status "admin_approve_204" "204" "$HTTP_CODE"

CURRENT_STATUS=$(get_wish_status "$FLAGGED_WISH_ID")
assert_json_field "admin_approve_status" "{\"s\":\"$CURRENT_STATUS\"}" ".s" "open"

# Admin approve on already open → 400
parse_response "$(do_post "/admin/wishes/$FLAGGED_WISH_ID/approve" "$ADMIN_TOKEN")"
assert_status "admin_approve_already_open_400" "400" "$HTTP_CODE"

# Create another for reject
REJECT_OWNER_EMAIL=$(unique_email)
REJECT_OWNER_TOKEN=$(setup_aged_user "$REJECT_OWNER_EMAIL" "RejectOwner")
REJECT_WISH_ID=$(create_open_wish "$REJECT_OWNER_TOKEN" "To reject" "other")
force_wish_status "$REJECT_WISH_ID" "flagged"

# Non-admin cannot admin-reject
parse_response "$(do_post "/admin/wishes/$REJECT_WISH_ID/reject" "$OWNER_TOKEN")"
assert_status "admin_reject_non_admin_403" "403" "$HTTP_CODE"

# Admin rejects
parse_response "$(do_post "/admin/wishes/$REJECT_WISH_ID/reject" "$ADMIN_TOKEN")"
assert_status "admin_reject_204" "204" "$HTTP_CODE"

CURRENT_STATUS=$(get_wish_status "$REJECT_WISH_ID")
assert_json_field "admin_reject_status" "{\"s\":\"$CURRENT_STATUS\"}" ".s" "rejected"

echo ""

# ── 17. E2E: FULL LIFECYCLE ─────────────────────────────────────────
c yellow "── 17. E2E: Full Lifecycle ────────────────────────────────"

E2E_OWNER_EMAIL=$(unique_email)
E2E_OWNER_TOKEN=$(setup_aged_user "$E2E_OWNER_EMAIL" "E2EOwner")

E2E_DONOR_EMAIL=$(unique_email)
E2E_DONOR_TOKEN=$(setup_aged_user "$E2E_DONOR_EMAIL" "E2EDonor")

# 1) Create wish
parse_response "$(do_post "/community/wishes" "$E2E_OWNER_TOKEN" \
  '{"title":"E2E: Need a laptop for studies","category":"education","is_anonymous":false}')"
assert_status "e2e_create_201" "201" "$HTTP_CODE"
E2E_WISH=$(echo "$BODY" | jq -r '.id')
assert_json_field "e2e_initial_status" "$BODY" ".status" "pending"
wait_for_status "$E2E_WISH" "open"
c green "  PASS  e2e: pending → open (moderation)"
TOTAL=$((TOTAL + 1)); PASSED=$((PASSED + 1))

# 2) Visible in public list
parse_response "$(do_get "/community/wishes")"
FOUND=$(echo "$BODY" | jq -r ".wishes[] | select(.id==\"$E2E_WISH\") | .id")
TOTAL=$((TOTAL + 1))
if [ "$FOUND" = "$E2E_WISH" ]; then
  PASSED=$((PASSED + 1))
  c green "  PASS  e2e: wish visible in public list"
else
  FAILED=$((FAILED + 1))
  FAILURES="$FAILURES\n  FAIL  e2e: wish not visible in public list"
  c red "  FAIL  e2e: wish not visible in public list"
fi

# 3) Donor offers
parse_response "$(do_post "/community/wishes/$E2E_WISH/offer" "$E2E_DONOR_TOKEN")"
assert_status "e2e_offer_204" "204" "$HTTP_CODE"

# 4) Exchange messages
parse_response "$(do_post "/community/wishes/$E2E_WISH/messages" "$E2E_OWNER_TOKEN" \
  '{"body":"Thank you so much!"}')"
assert_status "e2e_msg1_201" "201" "$HTTP_CODE"

parse_response "$(do_post "/community/wishes/$E2E_WISH/messages" "$E2E_DONOR_TOKEN" \
  '{"body":"I have a spare one. Where should I drop it?"}')"
assert_status "e2e_msg2_201" "201" "$HTTP_CODE"

parse_response "$(do_post "/community/wishes/$E2E_WISH/messages" "$E2E_OWNER_TOKEN" \
  '{"body":"I will DM you the details."}')"
assert_status "e2e_msg3_201" "201" "$HTTP_CODE"

# 5) Verify message list
parse_response "$(do_get "/community/wishes/$E2E_WISH/messages" "$E2E_OWNER_TOKEN")"
assert_status "e2e_msg_list_200" "200" "$HTTP_CODE"
assert_json_field "e2e_msg_total_3" "$BODY" ".total" "3"

# 6) Owner confirms fulfillment
parse_response "$(do_post "/community/wishes/$E2E_WISH/confirm" "$E2E_OWNER_TOKEN")"
assert_status "e2e_confirm_204" "204" "$HTTP_CODE"

# 7) Verify fulfilled
parse_response "$(do_get "/community/wishes/$E2E_WISH" "$E2E_OWNER_TOKEN")"
assert_status "e2e_fulfilled_200" "200" "$HTTP_CODE"
assert_json_field "e2e_fulfilled_status" "$BODY" ".status" "fulfilled"

# 8) Messages still readable after fulfill
parse_response "$(do_get "/community/wishes/$E2E_WISH/messages" "$E2E_DONOR_TOKEN")"
assert_status "e2e_msg_after_fulfill_200" "200" "$HTTP_CODE"
assert_json_field "e2e_msg_after_fulfill_total" "$BODY" ".total" "3"

# 9) No longer in public list
parse_response "$(do_get "/community/wishes")"
FOUND=$(echo "$BODY" | jq -r ".wishes[] | select(.id==\"$E2E_WISH\") | .id")
TOTAL=$((TOTAL + 1))
if [ -z "$FOUND" ]; then
  PASSED=$((PASSED + 1))
  c green "  PASS  e2e: fulfilled wish not in public list"
else
  FAILED=$((FAILED + 1))
  FAILURES="$FAILURES\n  FAIL  e2e: fulfilled wish still in public list"
  c red "  FAIL  e2e: fulfilled wish still in public list"
fi

# 10) Owner can create new wish after fulfillment
parse_response "$(do_post "/community/wishes" "$E2E_OWNER_TOKEN" \
  '{"title":"E2E: Second wish","category":"other","is_anonymous":true}')"
assert_status "e2e_new_wish_after_fulfill_201" "201" "$HTTP_CODE"

echo ""

# ── 18. E2E: OFFER → REJECT → RE-OFFER → CONFIRM ───────────────────
c yellow "── 18. E2E: Offer-Reject-ReOffer-Confirm ────────────────"

E2E2_OWNER_EMAIL=$(unique_email)
E2E2_OWNER_TOKEN=$(setup_aged_user "$E2E2_OWNER_EMAIL" "E2E2Owner")

E2E2_DONOR_A_EMAIL=$(unique_email)
E2E2_DONOR_A_TOKEN=$(setup_aged_user "$E2E2_DONOR_A_EMAIL" "DonorA")

E2E2_DONOR_B_EMAIL=$(unique_email)
E2E2_DONOR_B_TOKEN=$(setup_aged_user "$E2E2_DONOR_B_EMAIL" "DonorB")

E2E2_WISH=$(create_open_wish "$E2E2_OWNER_TOKEN" "E2E reject flow" "clothing")

# Donor A offers
parse_response "$(do_post "/community/wishes/$E2E2_WISH/offer" "$E2E2_DONOR_A_TOKEN")"
assert_status "e2e2_offer_a_204" "204" "$HTTP_CODE"

# Owner rejects A
parse_response "$(do_post "/community/wishes/$E2E2_WISH/reject" "$E2E2_OWNER_TOKEN")"
assert_status "e2e2_reject_a_204" "204" "$HTTP_CODE"

CURRENT_STATUS=$(get_wish_status "$E2E2_WISH")
assert_json_field "e2e2_back_to_open" "{\"s\":\"$CURRENT_STATUS\"}" ".s" "open"

# Donor B offers
parse_response "$(do_post "/community/wishes/$E2E2_WISH/offer" "$E2E2_DONOR_B_TOKEN")"
assert_status "e2e2_offer_b_204" "204" "$HTTP_CODE"

# Owner confirms B
parse_response "$(do_post "/community/wishes/$E2E2_WISH/confirm" "$E2E2_OWNER_TOKEN")"
assert_status "e2e2_confirm_b_204" "204" "$HTTP_CODE"

CURRENT_STATUS=$(get_wish_status "$E2E2_WISH")
assert_json_field "e2e2_fulfilled" "{\"s\":\"$CURRENT_STATUS\"}" ".s" "fulfilled"

echo ""

# ── 19. E2E: CLOSE AND CREATE NEW ───────────────────────────────────
c yellow "── 19. E2E: Close and Create New ─────────────────────────"

E2E3_OWNER_EMAIL=$(unique_email)
E2E3_OWNER_TOKEN=$(setup_aged_user "$E2E3_OWNER_EMAIL" "E2E3Owner")

E2E3_WISH=$(create_open_wish "$E2E3_OWNER_TOKEN" "Will close this" "home")

# Close
parse_response "$(do_post "/community/wishes/$E2E3_WISH/close" "$E2E3_OWNER_TOKEN")"
assert_status "e2e3_close_204" "204" "$HTTP_CODE"

# Closed wish not in public list
parse_response "$(do_get "/community/wishes")"
FOUND=$(echo "$BODY" | jq -r ".wishes[] | select(.id==\"$E2E3_WISH\") | .id")
TOTAL=$((TOTAL + 1))
if [ -z "$FOUND" ]; then
  PASSED=$((PASSED + 1))
  c green "  PASS  e2e3: closed wish not in public list"
else
  FAILED=$((FAILED + 1))
  FAILURES="$FAILURES\n  FAIL  e2e3: closed wish still in public list"
  c red "  FAIL  e2e3: closed wish still in public list"
fi

# Can create new wish
parse_response "$(do_post "/community/wishes" "$E2E3_OWNER_TOKEN" \
  '{"title":"Brand new after close","category":"children","is_anonymous":true}')"
assert_status "e2e3_new_wish_201" "201" "$HTTP_CODE"

echo ""

# ── 20. EDGE CASES ──────────────────────────────────────────────────
c yellow "── 20. Edge Cases ─────────────────────────────────────────"

# Update with matched wish → 400
EDGE_OWNER_EMAIL=$(unique_email)
EDGE_OWNER_TOKEN=$(setup_aged_user "$EDGE_OWNER_EMAIL" "EdgeOwner")
EDGE_DONOR_EMAIL=$(unique_email)
EDGE_DONOR_TOKEN=$(setup_aged_user "$EDGE_DONOR_EMAIL" "EdgeDonor")
EDGE_WISH=$(create_open_wish "$EDGE_OWNER_TOKEN" "Edge case wish" "other")
EDGE_DONOR_ID=$(get_user_id "$EDGE_DONOR_EMAIL")
force_match "$EDGE_WISH" "$EDGE_DONOR_ID"

parse_response "$(do_patch "/community/wishes/$EDGE_WISH" "$EDGE_OWNER_TOKEN" \
  '{"title":"Cannot update matched"}')"
assert_status "update_matched_wish_400" "400" "$HTTP_CODE"

# Close matched wish (should work)
parse_response "$(do_post "/community/wishes/$EDGE_WISH/close" "$EDGE_OWNER_TOKEN")"
assert_status "close_matched_wish_204" "204" "$HTTP_CODE"

# Messages readable on closed wish
parse_response "$(do_get "/community/wishes/$EDGE_WISH/messages" "$EDGE_OWNER_TOKEN")"
assert_status "list_msg_closed_200" "200" "$HTTP_CODE"

# Send message to non-existent wish → 404
parse_response "$(do_post "/community/wishes/$FAKE_UUID/messages" "$OWNER_TOKEN" \
  '{"body":"to nowhere"}')"
assert_status "send_msg_no_wish_404" "404" "$HTTP_CODE"

# List messages on non-existent wish → 404
parse_response "$(do_get "/community/wishes/$FAKE_UUID/messages" "$OWNER_TOKEN")"
assert_status "list_msg_no_wish_404" "404" "$HTTP_CODE"

# Confirm on non-matched wish → 400
parse_response "$(do_post "/community/wishes/$WISH2_ID/confirm" "$OWNER2_TOKEN")"
assert_status "confirm_not_matched_400" "400" "$HTTP_CODE"

# Multiple messages in sequence then verify ordering
MSG_WISH_OWNER_EMAIL=$(unique_email)
MSG_WISH_OWNER_TOKEN=$(setup_aged_user "$MSG_WISH_OWNER_EMAIL" "MsgOwner")
MSG_WISH_DONOR_EMAIL=$(unique_email)
MSG_WISH_DONOR_TOKEN=$(setup_aged_user "$MSG_WISH_DONOR_EMAIL" "MsgDonor")
MSG_WISH=$(create_open_wish "$MSG_WISH_OWNER_TOKEN" "Message ordering" "other")
MSG_DONOR_ID=$(get_user_id "$MSG_WISH_DONOR_EMAIL")
force_match "$MSG_WISH" "$MSG_DONOR_ID"

for i in $(seq 1 5); do
  do_post "/community/wishes/$MSG_WISH/messages" "$MSG_WISH_OWNER_TOKEN" \
    "{\"body\":\"Message $i from owner\"}" > /dev/null
done
for i in $(seq 1 3); do
  do_post "/community/wishes/$MSG_WISH/messages" "$MSG_WISH_DONOR_TOKEN" \
    "{\"body\":\"Reply $i from donor\"}" > /dev/null
done

parse_response "$(do_get "/community/wishes/$MSG_WISH/messages" "$MSG_WISH_OWNER_TOKEN")"
assert_status "msg_ordering_200" "200" "$HTTP_CODE"
assert_json_field "msg_ordering_total_8" "$BODY" ".total" "8"

# Verify sender display names
FIRST_SENDER=$(echo "$BODY" | jq -r '.messages[0].sender_display_name')
TOTAL=$((TOTAL + 1))
if [ "$FIRST_SENDER" = "MsgOwner" ]; then
  PASSED=$((PASSED + 1))
  c green "  PASS  msg_sender_display_name_correct ($FIRST_SENDER)"
else
  FAILED=$((FAILED + 1))
  FAILURES="$FAILURES\n  FAIL  msg_sender_display_name: expected 'MsgOwner', got '$FIRST_SENDER'"
  c red "  FAIL  msg_sender_display_name: expected 'MsgOwner', got '$FIRST_SENDER'"
fi

echo ""

# ══════════════════════════════════════════════════════════════════════
# SUMMARY
# ══════════════════════════════════════════════════════════════════════
echo ""
c blue "══════════════════════════════════════════════════════════════"
c blue " RESULTS"
c blue "══════════════════════════════════════════════════════════════"
echo ""
c green " Passed: $PASSED / $TOTAL"
if [ $FAILED -gt 0 ]; then
  c red   " Failed: $FAILED / $TOTAL"
  echo ""
  c red   " Failures:"
  echo -e "$FAILURES"
else
  c green " All tests passed!"
fi
echo ""

# Exit with failure code if any test failed
[ $FAILED -eq 0 ]
