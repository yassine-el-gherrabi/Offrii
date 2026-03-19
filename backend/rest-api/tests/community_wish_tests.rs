mod common;

use axum::http::StatusCode;
use common::{TEST_PASSWORD, TestApp, assert_error};
use serde_json::json;
use uuid::Uuid;

// ── Helpers ───────────────────────────────────────────────────────────

async fn age_account(app: &TestApp, email: &str) {
    sqlx::query("UPDATE users SET created_at = NOW() - INTERVAL '48 hours' WHERE email = $1")
        .bind(email)
        .execute(&app.db)
        .await
        .unwrap();
}

async fn make_admin(app: &TestApp, email: &str) {
    sqlx::query("UPDATE users SET is_admin = true WHERE email = $1")
        .bind(email)
        .execute(&app.db)
        .await
        .unwrap();
}

async fn setup_aged_user(app: &TestApp, email: &str) -> String {
    let token = app.setup_user_token(email, TEST_PASSWORD).await;
    age_account(app, email).await;
    token
}

async fn setup_aged_user_with_name(app: &TestApp, email: &str, name: &str) -> String {
    let (status, body) = app
        .register_user_with_name(email, TEST_PASSWORD, name)
        .await;
    assert_eq!(status, StatusCode::CREATED, "register {email}: {body}");
    let token = body["tokens"]["access_token"].as_str().unwrap().to_string();
    age_account(app, email).await;
    token
}

async fn get_user_id(app: &TestApp, email: &str) -> Uuid {
    let row: (Uuid,) = sqlx::query_as("SELECT id FROM users WHERE email = $1")
        .bind(email)
        .fetch_one(&app.db)
        .await
        .unwrap();
    row.0
}

/// Create a wish and wait for it to transition to "open" via background moderation.
async fn create_open_wish(app: &TestApp, token: &str) -> Uuid {
    let body = json!({
        "title": "Need winter coat",
        "category": "clothing",
        "is_anonymous": true,
    });
    let (status, resp) = app
        .post_json_with_auth("/community/wishes", &body, token)
        .await;
    assert_eq!(status, StatusCode::CREATED, "create wish: {resp}");
    let wish_id = Uuid::parse_str(resp["id"].as_str().unwrap()).unwrap();
    wait_for_wish_status(app, wish_id, "open").await;
    wish_id
}

async fn wait_for_wish_status(app: &TestApp, wish_id: Uuid, expected: &str) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
    loop {
        let row: (String,) = sqlx::query_as("SELECT status FROM community_wishes WHERE id = $1")
            .bind(wish_id)
            .fetch_one(&app.db)
            .await
            .unwrap();
        if row.0 == expected {
            return;
        }
        if std::time::Instant::now() >= deadline {
            panic!(
                "wish {wish_id} did not reach status '{expected}' within 5s (current: '{}')",
                row.0
            );
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
}

async fn force_wish_status(app: &TestApp, wish_id: Uuid, status: &str) {
    sqlx::query("UPDATE community_wishes SET status = $1 WHERE id = $2")
        .bind(status)
        .bind(wish_id)
        .execute(&app.db)
        .await
        .unwrap();
}

async fn force_match(app: &TestApp, wish_id: Uuid, donor_id: Uuid) {
    sqlx::query(
        "UPDATE community_wishes SET status = 'matched', matched_with = $1, matched_at = NOW() WHERE id = $2",
    )
    .bind(donor_id)
    .bind(wish_id)
    .execute(&app.db)
    .await
    .unwrap();
}

// ═══════════════════════════════════════════════════════════════════════
// Cat. 1: Auth Guards
// ═══════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn create_wish_without_auth_401() {
    let app = TestApp::new().await;
    let body = json!({ "title": "Test", "category": "clothing", "is_anonymous": true });
    let (status, _) = app.post_json("/community/wishes", &body).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn list_my_wishes_without_auth_401() {
    let app = TestApp::new().await;
    let (status, _) = app.get_no_auth("/community/wishes/mine").await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn update_wish_without_auth_401() {
    let app = TestApp::new().await;
    let id = Uuid::new_v4();
    let body = json!({ "title": "Updated" });
    let (status, _) = app
        .patch_json_with_auth(&format!("/community/wishes/{id}"), &body, "bad-token")
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn close_wish_without_auth_401() {
    let app = TestApp::new().await;
    let id = Uuid::new_v4();
    let (status, _) = app
        .post_empty(&format!("/community/wishes/{id}/close"))
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn offer_wish_without_auth_401() {
    let app = TestApp::new().await;
    let id = Uuid::new_v4();
    let (status, _) = app
        .post_empty(&format!("/community/wishes/{id}/offer"))
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn report_wish_without_auth_401() {
    let app = TestApp::new().await;
    let id = Uuid::new_v4();
    let body = json!({});
    let (status, _) = app
        .post_json(&format!("/community/wishes/{id}/report"), &body)
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

// ═══════════════════════════════════════════════════════════════════════
// Cat. 2: Création
// ═══════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn create_wish_anonymous_201() {
    let app = TestApp::new().await;
    let token = setup_aged_user(&app, "owner@test.com").await;

    let body = json!({
        "title": "Need winter coat",
        "category": "clothing",
        "is_anonymous": true,
    });
    let (status, resp) = app
        .post_json_with_auth("/community/wishes", &body, &token)
        .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(resp["status"].as_str(), Some("pending"));
    assert_eq!(resp["title"].as_str(), Some("Need winter coat"));
    assert_eq!(resp["category"].as_str(), Some("clothing"));
    assert_eq!(resp["is_anonymous"].as_bool(), Some(true));

    // Wait for moderation to approve
    let wish_id = Uuid::parse_str(resp["id"].as_str().unwrap()).unwrap();
    wait_for_wish_status(&app, wish_id, "open").await;
}

#[tokio::test]
async fn create_wish_non_anonymous_with_display_name_201() {
    let app = TestApp::new().await;
    let token = setup_aged_user_with_name(&app, "owner@test.com", "Alice").await;

    let body = json!({
        "title": "Need school books",
        "category": "education",
        "is_anonymous": false,
    });
    let (status, resp) = app
        .post_json_with_auth("/community/wishes", &body, &token)
        .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(resp["is_anonymous"].as_bool(), Some(false));
}

#[tokio::test]
async fn create_wish_non_anonymous_without_display_name_400() {
    let app = TestApp::new().await;
    let token = setup_aged_user(&app, "owner@test.com").await;

    let body = json!({
        "title": "Need school books",
        "category": "education",
        "is_anonymous": false,
    });
    let (status, resp) = app
        .post_json_with_auth("/community/wishes", &body, &token)
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_error(&resp, "BAD_REQUEST");
}

#[tokio::test]
async fn create_wish_with_description_201() {
    let app = TestApp::new().await;
    let token = setup_aged_user(&app, "owner@test.com").await;

    let body = json!({
        "title": "Need winter coat",
        "description": "Size M, preferably dark color",
        "category": "clothing",
        "is_anonymous": true,
    });
    let (status, resp) = app
        .post_json_with_auth("/community/wishes", &body, &token)
        .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(
        resp["description"].as_str(),
        Some("Size M, preferably dark color")
    );
}

#[tokio::test]
async fn create_wish_account_too_young_403() {
    let app = TestApp::new().await;
    // Do NOT age the account
    let token = app.setup_user_token("new@test.com", TEST_PASSWORD).await;

    let body = json!({
        "title": "Need something",
        "category": "other",
        "is_anonymous": true,
    });
    let (status, resp) = app
        .post_json_with_auth("/community/wishes", &body, &token)
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_error(&resp, "FORBIDDEN");
}

#[tokio::test]
async fn create_wish_max_active_reached_409() {
    let app = TestApp::new().await;
    let token = setup_aged_user(&app, "owner@test.com").await;
    // Create 3 open wishes (the new limit)
    create_open_wish(&app, &token).await;
    create_open_wish(&app, &token).await;
    create_open_wish(&app, &token).await;

    let body = json!({
        "title": "Fourth wish",
        "category": "other",
        "is_anonymous": true,
    });
    let (status, resp) = app
        .post_json_with_auth("/community/wishes", &body, &token)
        .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_error(&resp, "CONFLICT");
}

#[tokio::test]
async fn create_wish_pending_counts_as_active_409() {
    let app = TestApp::new().await;
    let token = setup_aged_user(&app, "owner@test.com").await;

    // Create 3 wishes rapidly — will be pending immediately
    for i in 0..3 {
        let body = json!({
            "title": format!("Wish {i}"),
            "category": "other",
            "is_anonymous": true,
        });
        let (status, _) = app
            .post_json_with_auth("/community/wishes", &body, &token)
            .await;
        assert_eq!(status, StatusCode::CREATED);
    }

    // 4th should fail — pending counts as active
    let body = json!({
        "title": "Fourth wish",
        "category": "other",
        "is_anonymous": true,
    });
    let (status, resp) = app
        .post_json_with_auth("/community/wishes", &body, &token)
        .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_error(&resp, "CONFLICT");
}

#[tokio::test]
async fn create_wish_matched_counts_as_active_409() {
    let app = TestApp::new().await;
    let token = setup_aged_user(&app, "owner@test.com").await;
    // Create 2 open + 1 matched = 3 active
    create_open_wish(&app, &token).await;
    create_open_wish(&app, &token).await;
    let wish_id = create_open_wish(&app, &token).await;

    // Force match the last one
    let donor_token = setup_aged_user(&app, "donor@test.com").await;
    let donor_id = get_user_id(&app, "donor@test.com").await;
    force_match(&app, wish_id, donor_id).await;
    let _ = donor_token; // keep alive

    let body = json!({
        "title": "Fourth wish",
        "category": "other",
        "is_anonymous": true,
    });
    let (status, resp) = app
        .post_json_with_auth("/community/wishes", &body, &token)
        .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_error(&resp, "CONFLICT");
}

#[tokio::test]
async fn create_wish_after_close_allows_new_201() {
    let app = TestApp::new().await;
    let token = setup_aged_user(&app, "owner@test.com").await;
    // Fill up all 3 slots
    create_open_wish(&app, &token).await;
    create_open_wish(&app, &token).await;
    let wish_id = create_open_wish(&app, &token).await;

    // Close one
    let (status, _) = app
        .post_with_auth(&format!("/community/wishes/{wish_id}/close"), &token)
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Now create another — should succeed since we freed a slot
    let body = json!({
        "title": "New wish after close",
        "category": "other",
        "is_anonymous": true,
    });
    let (status, _) = app
        .post_json_with_auth("/community/wishes", &body, &token)
        .await;
    assert_eq!(status, StatusCode::CREATED);
}

#[tokio::test]
async fn create_wish_invalid_category_400() {
    let app = TestApp::new().await;
    let token = setup_aged_user(&app, "owner@test.com").await;

    let body = json!({
        "title": "Something",
        "category": "invalid_cat",
        "is_anonymous": true,
    });
    let (status, resp) = app
        .post_json_with_auth("/community/wishes", &body, &token)
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_error(&resp, "BAD_REQUEST");
}

#[tokio::test]
async fn create_wish_title_empty_400() {
    let app = TestApp::new().await;
    let token = setup_aged_user(&app, "owner@test.com").await;

    let body = json!({
        "title": "",
        "category": "other",
        "is_anonymous": true,
    });
    let (status, resp) = app
        .post_json_with_auth("/community/wishes", &body, &token)
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_error(&resp, "BAD_REQUEST");
}

#[tokio::test]
async fn create_wish_title_too_long_400() {
    let app = TestApp::new().await;
    let token = setup_aged_user(&app, "owner@test.com").await;

    let long_title = "x".repeat(256);
    let body = json!({
        "title": long_title,
        "category": "other",
        "is_anonymous": true,
    });
    let (status, resp) = app
        .post_json_with_auth("/community/wishes", &body, &token)
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_error(&resp, "BAD_REQUEST");
}

#[tokio::test]
async fn create_wish_description_too_long_400() {
    let app = TestApp::new().await;
    let token = setup_aged_user(&app, "owner@test.com").await;

    let long_desc = "x".repeat(2001);
    let body = json!({
        "title": "Valid title",
        "description": long_desc,
        "category": "other",
        "is_anonymous": true,
    });
    let (status, resp) = app
        .post_json_with_auth("/community/wishes", &body, &token)
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_error(&resp, "BAD_REQUEST");
}

// ═══════════════════════════════════════════════════════════════════════
// Cat. 3: Liste publique
// ═══════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn list_wishes_no_auth_returns_open_200() {
    let app = TestApp::new().await;
    let token = setup_aged_user(&app, "owner@test.com").await;
    create_open_wish(&app, &token).await;

    let (status, body) = app.get_no_auth("/community/wishes").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"].as_array().unwrap().len(), 1);
    assert_eq!(body["pagination"]["total"].as_i64(), Some(1));
}

#[tokio::test]
async fn list_wishes_with_auth_200() {
    let app = TestApp::new().await;
    let owner_token = setup_aged_user(&app, "owner@test.com").await;
    create_open_wish(&app, &owner_token).await;

    let viewer_token = setup_aged_user(&app, "viewer@test.com").await;
    let (status, body) = app.get_with_auth("/community/wishes", &viewer_token).await;
    assert_eq!(status, StatusCode::OK);
    let wish = &body["data"].as_array().unwrap()[0];
    assert_eq!(wish["is_mine"].as_bool(), Some(false));
    assert_eq!(wish["is_matched_by_me"].as_bool(), Some(false));

    // Owner sees is_mine=true
    let (status, body) = app.get_with_auth("/community/wishes", &owner_token).await;
    assert_eq!(status, StatusCode::OK);
    let wish = &body["data"].as_array().unwrap()[0];
    assert_eq!(wish["is_mine"].as_bool(), Some(true));
}

#[tokio::test]
async fn list_wishes_empty_200() {
    let app = TestApp::new().await;
    let (status, body) = app.get_no_auth("/community/wishes").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"].as_array().unwrap().len(), 0);
    assert_eq!(body["pagination"]["total"].as_i64(), Some(0));
}

#[tokio::test]
async fn list_wishes_filter_by_category_200() {
    let app = TestApp::new().await;

    // Create two wishes in different categories from different users
    let t1 = setup_aged_user(&app, "u1@test.com").await;
    let body1 = json!({ "title": "Coat", "category": "clothing", "is_anonymous": true });
    let (s, r) = app
        .post_json_with_auth("/community/wishes", &body1, &t1)
        .await;
    assert_eq!(s, StatusCode::CREATED);
    let w1 = Uuid::parse_str(r["id"].as_str().unwrap()).unwrap();
    wait_for_wish_status(&app, w1, "open").await;

    let t2 = setup_aged_user(&app, "u2@test.com").await;
    let body2 = json!({ "title": "Books", "category": "education", "is_anonymous": true });
    let (s, r) = app
        .post_json_with_auth("/community/wishes", &body2, &t2)
        .await;
    assert_eq!(s, StatusCode::CREATED);
    let w2 = Uuid::parse_str(r["id"].as_str().unwrap()).unwrap();
    wait_for_wish_status(&app, w2, "open").await;

    let (status, body) = app.get_no_auth("/community/wishes?category=clothing").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["pagination"]["total"].as_i64(), Some(1));
    assert_eq!(
        body["data"].as_array().unwrap()[0]["category"].as_str(),
        Some("clothing")
    );
}

#[tokio::test]
async fn list_wishes_pagination_200() {
    let app = TestApp::new().await;

    // Create 3 wishes from 3 different users
    for i in 0..3 {
        let email = format!("u{i}@test.com");
        let token = setup_aged_user(&app, &email).await;
        let body = json!({
            "title": format!("Wish {i}"),
            "category": "other",
            "is_anonymous": true,
        });
        let (s, r) = app
            .post_json_with_auth("/community/wishes", &body, &token)
            .await;
        assert_eq!(s, StatusCode::CREATED);
        let wid = Uuid::parse_str(r["id"].as_str().unwrap()).unwrap();
        wait_for_wish_status(&app, wid, "open").await;
    }

    // Page 1: limit=2, page=1
    let (status, body) = app.get_no_auth("/community/wishes?limit=2&page=1").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"].as_array().unwrap().len(), 2);
    assert_eq!(body["pagination"]["total"].as_i64(), Some(3));
    assert_eq!(body["pagination"]["page"].as_i64(), Some(1));
    assert_eq!(body["pagination"]["limit"].as_i64(), Some(2));
    assert_eq!(body["pagination"]["total_pages"].as_i64(), Some(2));
    assert_eq!(body["pagination"]["has_more"].as_bool(), Some(true));

    // Page 2: limit=2, page=2
    let (status, body) = app.get_no_auth("/community/wishes?limit=2&page=2").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"].as_array().unwrap().len(), 1);
    assert_eq!(body["pagination"]["total"].as_i64(), Some(3));
    assert_eq!(body["pagination"]["page"].as_i64(), Some(2));
    assert_eq!(body["pagination"]["has_more"].as_bool(), Some(false));
}

#[tokio::test]
async fn list_wishes_invalid_limit_400() {
    let app = TestApp::new().await;
    let (status, resp) = app.get_no_auth("/community/wishes?limit=0").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_error(&resp, "BAD_REQUEST");

    let (status, resp) = app.get_no_auth("/community/wishes?limit=101").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_error(&resp, "BAD_REQUEST");
}

#[tokio::test]
async fn list_wishes_anonymous_hides_display_name() {
    let app = TestApp::new().await;
    let token = setup_aged_user_with_name(&app, "owner@test.com", "Alice").await;

    let body = json!({
        "title": "Anonymous wish",
        "category": "other",
        "is_anonymous": true,
    });
    let (s, r) = app
        .post_json_with_auth("/community/wishes", &body, &token)
        .await;
    assert_eq!(s, StatusCode::CREATED);
    let wid = Uuid::parse_str(r["id"].as_str().unwrap()).unwrap();
    wait_for_wish_status(&app, wid, "open").await;

    let (status, body) = app.get_no_auth("/community/wishes").await;
    assert_eq!(status, StatusCode::OK);
    let wish = &body["data"].as_array().unwrap()[0];
    assert!(
        wish["display_name"].is_null(),
        "display_name should be null for anonymous wishes"
    );
}

#[tokio::test]
async fn list_wishes_excludes_non_open() {
    let app = TestApp::new().await;
    let token = setup_aged_user(&app, "owner@test.com").await;

    let body = json!({
        "title": "Wish to be pending",
        "category": "other",
        "is_anonymous": true,
    });
    let (s, r) = app
        .post_json_with_auth("/community/wishes", &body, &token)
        .await;
    assert_eq!(s, StatusCode::CREATED);
    let wish_id = Uuid::parse_str(r["id"].as_str().unwrap()).unwrap();
    wait_for_wish_status(&app, wish_id, "open").await;

    // Force to flagged
    force_wish_status(&app, wish_id, "flagged").await;

    let (status, body) = app.get_no_auth("/community/wishes").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"].as_array().unwrap().len(), 0);
}

// ═══════════════════════════════════════════════════════════════════════
// Cat. 4: Détail wish
// ═══════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn get_wish_open_no_auth_200() {
    let app = TestApp::new().await;
    let token = setup_aged_user(&app, "owner@test.com").await;
    let wish_id = create_open_wish(&app, &token).await;

    let (status, body) = app
        .get_no_auth(&format!("/community/wishes/{wish_id}"))
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["id"].as_str(), Some(wish_id.to_string().as_str()));
    assert_eq!(body["status"].as_str(), Some("open"));
}

#[tokio::test]
async fn get_wish_open_owner_is_mine_200() {
    let app = TestApp::new().await;
    let token = setup_aged_user(&app, "owner@test.com").await;
    let wish_id = create_open_wish(&app, &token).await;

    let (status, body) = app
        .get_with_auth(&format!("/community/wishes/{wish_id}"), &token)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["is_mine"].as_bool(), Some(true));
}

#[tokio::test]
async fn get_wish_matched_visible_to_donor_200() {
    let app = TestApp::new().await;
    let owner_token = setup_aged_user_with_name(&app, "owner@test.com", "Alice").await;
    let donor_token = setup_aged_user_with_name(&app, "donor@test.com", "Bob").await;
    let donor_id = get_user_id(&app, "donor@test.com").await;
    let wish_id = create_open_wish(&app, &owner_token).await;
    force_match(&app, wish_id, donor_id).await;

    let (status, body) = app
        .get_with_auth(&format!("/community/wishes/{wish_id}"), &donor_token)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["is_matched_by_me"].as_bool(), Some(true));
    assert_eq!(body["matched_with_display_name"].as_str(), Some("Bob"));
}

#[tokio::test]
async fn get_wish_matched_visible_to_owner_200() {
    let app = TestApp::new().await;
    let owner_token = setup_aged_user_with_name(&app, "owner@test.com", "Alice").await;
    let donor_token = setup_aged_user_with_name(&app, "donor@test.com", "Bob").await;
    let donor_id = get_user_id(&app, "donor@test.com").await;
    let wish_id = create_open_wish(&app, &owner_token).await;
    force_match(&app, wish_id, donor_id).await;
    let _ = donor_token;

    let (status, body) = app
        .get_with_auth(&format!("/community/wishes/{wish_id}"), &owner_token)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["is_mine"].as_bool(), Some(true));
    assert_eq!(body["matched_with_display_name"].as_str(), Some("Bob"));
}

#[tokio::test]
async fn get_wish_pending_not_visible_to_stranger_404() {
    let app = TestApp::new().await;
    let owner_token = setup_aged_user(&app, "owner@test.com").await;

    let body = json!({
        "title": "Pending wish",
        "category": "other",
        "is_anonymous": true,
    });
    let (s, r) = app
        .post_json_with_auth("/community/wishes", &body, &owner_token)
        .await;
    assert_eq!(s, StatusCode::CREATED);
    let wish_id = Uuid::parse_str(r["id"].as_str().unwrap()).unwrap();
    // Wait for background moderation to finish, then force back to pending
    wait_for_wish_status(&app, wish_id, "open").await;
    force_wish_status(&app, wish_id, "pending").await;

    let stranger_token = setup_aged_user(&app, "stranger@test.com").await;
    let (status, resp) = app
        .get_with_auth(&format!("/community/wishes/{wish_id}"), &stranger_token)
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_error(&resp, "NOT_FOUND");
}

#[tokio::test]
async fn get_wish_pending_visible_to_owner_200() {
    let app = TestApp::new().await;
    let owner_token = setup_aged_user(&app, "owner@test.com").await;

    let body = json!({
        "title": "Pending wish",
        "category": "other",
        "is_anonymous": true,
    });
    let (s, r) = app
        .post_json_with_auth("/community/wishes", &body, &owner_token)
        .await;
    assert_eq!(s, StatusCode::CREATED);
    let wish_id = Uuid::parse_str(r["id"].as_str().unwrap()).unwrap();
    force_wish_status(&app, wish_id, "pending").await;

    let (status, body) = app
        .get_with_auth(&format!("/community/wishes/{wish_id}"), &owner_token)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["is_mine"].as_bool(), Some(true));
}

#[tokio::test]
async fn get_wish_anonymous_hides_name_for_stranger() {
    let app = TestApp::new().await;
    let token = setup_aged_user_with_name(&app, "owner@test.com", "Alice").await;
    let wish_id = create_open_wish(&app, &token).await;

    let (status, body) = app
        .get_no_auth(&format!("/community/wishes/{wish_id}"))
        .await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        body["display_name"].is_null(),
        "anonymous wish should hide display_name from strangers"
    );
}

#[tokio::test]
async fn get_wish_anonymous_shows_name_to_owner() {
    let app = TestApp::new().await;
    let token = setup_aged_user_with_name(&app, "owner@test.com", "Alice").await;
    let wish_id = create_open_wish(&app, &token).await;

    let (status, body) = app
        .get_with_auth(&format!("/community/wishes/{wish_id}"), &token)
        .await;
    assert_eq!(status, StatusCode::OK);
    // Owner always sees their own display_name
    assert_eq!(body["display_name"].as_str(), Some("Alice"));
}

#[tokio::test]
async fn get_wish_not_found_404() {
    let app = TestApp::new().await;
    let fake_id = Uuid::new_v4();
    let (status, resp) = app
        .get_no_auth(&format!("/community/wishes/{fake_id}"))
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_error(&resp, "NOT_FOUND");
}

// ═══════════════════════════════════════════════════════════════════════
// Cat. 5: Mes souhaits
// ═══════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn list_my_wishes_returns_all_statuses_200() {
    let app = TestApp::new().await;
    let token = setup_aged_user(&app, "owner@test.com").await;
    let wish_id = create_open_wish(&app, &token).await;

    // Close the wish
    app.post_with_auth(&format!("/community/wishes/{wish_id}/close"), &token)
        .await;

    let (status, body) = app.get_with_auth("/community/wishes/mine", &token).await;
    assert_eq!(status, StatusCode::OK);
    let wishes = body.as_array().unwrap();
    assert_eq!(wishes.len(), 1);
    assert_eq!(wishes[0]["status"].as_str(), Some("closed"));
}

#[tokio::test]
async fn list_my_wishes_empty_200() {
    let app = TestApp::new().await;
    let token = setup_aged_user(&app, "owner@test.com").await;

    let (status, body) = app.get_with_auth("/community/wishes/mine", &token).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body.as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn list_my_wishes_shows_private_fields() {
    let app = TestApp::new().await;
    let token = setup_aged_user(&app, "owner@test.com").await;
    create_open_wish(&app, &token).await;

    let (status, body) = app.get_with_auth("/community/wishes/mine", &token).await;
    assert_eq!(status, StatusCode::OK);
    let wish = &body.as_array().unwrap()[0];
    // Private fields should exist (even if 0/null)
    assert!(wish.get("report_count").is_some());
    assert!(wish.get("reopen_count").is_some());
    assert!(wish.get("moderation_note").is_some());
    assert!(wish.get("closed_at").is_some());
}

// ═══════════════════════════════════════════════════════════════════════
// Cat. 6: Update wish
// ═══════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn update_wish_title_200() {
    let app = TestApp::new().await;
    let token = setup_aged_user(&app, "owner@test.com").await;
    let wish_id = create_open_wish(&app, &token).await;

    let body = json!({ "title": "Updated title" });
    let (status, resp) = app
        .patch_json_with_auth(&format!("/community/wishes/{wish_id}"), &body, &token)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["title"].as_str(), Some("Updated title"));
}

#[tokio::test]
async fn update_wish_description_200() {
    let app = TestApp::new().await;
    let token = setup_aged_user(&app, "owner@test.com").await;
    let wish_id = create_open_wish(&app, &token).await;

    let body = json!({ "description": "New description" });
    let (status, resp) = app
        .patch_json_with_auth(&format!("/community/wishes/{wish_id}"), &body, &token)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["description"].as_str(), Some("New description"));
}

#[tokio::test]
async fn update_wish_category_200() {
    let app = TestApp::new().await;
    let token = setup_aged_user(&app, "owner@test.com").await;
    let wish_id = create_open_wish(&app, &token).await;

    let body = json!({ "category": "health" });
    let (status, resp) = app
        .patch_json_with_auth(&format!("/community/wishes/{wish_id}"), &body, &token)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["category"].as_str(), Some("health"));
}

#[tokio::test]
async fn update_wish_not_owner_403() {
    let app = TestApp::new().await;
    let owner_token = setup_aged_user(&app, "owner@test.com").await;
    let wish_id = create_open_wish(&app, &owner_token).await;

    let other_token = setup_aged_user(&app, "other@test.com").await;
    let body = json!({ "title": "Hijacked" });
    let (status, resp) = app
        .patch_json_with_auth(&format!("/community/wishes/{wish_id}"), &body, &other_token)
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_error(&resp, "FORBIDDEN");
}

#[tokio::test]
async fn update_wish_not_open_400() {
    let app = TestApp::new().await;
    let owner_token = setup_aged_user(&app, "owner@test.com").await;
    let wish_id = create_open_wish(&app, &owner_token).await;

    // Force to matched
    let donor_id = get_user_id(&app, "owner@test.com").await; // doesn't matter for force
    force_match(&app, wish_id, donor_id).await;

    let body = json!({ "title": "Update matched" });
    let (status, resp) = app
        .patch_json_with_auth(&format!("/community/wishes/{wish_id}"), &body, &owner_token)
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_error(&resp, "BAD_REQUEST");
}

#[tokio::test]
async fn update_wish_review_status_allowed_200() {
    let app = TestApp::new().await;
    let token = setup_aged_user(&app, "owner@test.com").await;
    let wish_id = create_open_wish(&app, &token).await;
    force_wish_status(&app, wish_id, "review").await;

    let body = json!({ "title": "Updated in review" });
    let (status, resp) = app
        .patch_json_with_auth(&format!("/community/wishes/{wish_id}"), &body, &token)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["title"].as_str(), Some("Updated in review"));
}

#[tokio::test]
async fn update_wish_not_found_404() {
    let app = TestApp::new().await;
    let token = setup_aged_user(&app, "owner@test.com").await;
    let fake_id = Uuid::new_v4();

    let body = json!({ "title": "Nope" });
    let (status, resp) = app
        .patch_json_with_auth(&format!("/community/wishes/{fake_id}"), &body, &token)
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_error(&resp, "NOT_FOUND");
}

#[tokio::test]
async fn update_wish_invalid_category_400() {
    let app = TestApp::new().await;
    let token = setup_aged_user(&app, "owner@test.com").await;
    let wish_id = create_open_wish(&app, &token).await;

    let body = json!({ "category": "bogus" });
    let (status, resp) = app
        .patch_json_with_auth(&format!("/community/wishes/{wish_id}"), &body, &token)
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_error(&resp, "BAD_REQUEST");
}

// ═══════════════════════════════════════════════════════════════════════
// Cat. 7: Close wish
// ═══════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn close_wish_from_open_204() {
    let app = TestApp::new().await;
    let token = setup_aged_user(&app, "owner@test.com").await;
    let wish_id = create_open_wish(&app, &token).await;

    let (status, _) = app
        .post_with_auth(&format!("/community/wishes/{wish_id}/close"), &token)
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Verify in DB
    let row: (String, Option<chrono::DateTime<chrono::Utc>>) =
        sqlx::query_as("SELECT status, closed_at FROM community_wishes WHERE id = $1")
            .bind(wish_id)
            .fetch_one(&app.db)
            .await
            .unwrap();
    assert_eq!(row.0, "closed");
    assert!(row.1.is_some(), "closed_at should be set");
}

#[tokio::test]
async fn close_wish_from_matched_204() {
    let app = TestApp::new().await;
    let owner_token = setup_aged_user(&app, "owner@test.com").await;
    let donor_token = setup_aged_user(&app, "donor@test.com").await;
    let donor_id = get_user_id(&app, "donor@test.com").await;
    let wish_id = create_open_wish(&app, &owner_token).await;
    force_match(&app, wish_id, donor_id).await;
    let _ = donor_token;

    let (status, _) = app
        .post_with_auth(&format!("/community/wishes/{wish_id}/close"), &owner_token)
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn close_wish_from_pending_204() {
    let app = TestApp::new().await;
    let token = setup_aged_user(&app, "owner@test.com").await;

    let body = json!({ "title": "Wish", "category": "other", "is_anonymous": true });
    let (s, r) = app
        .post_json_with_auth("/community/wishes", &body, &token)
        .await;
    assert_eq!(s, StatusCode::CREATED);
    let wish_id = Uuid::parse_str(r["id"].as_str().unwrap()).unwrap();
    force_wish_status(&app, wish_id, "pending").await;

    let (status, _) = app
        .post_with_auth(&format!("/community/wishes/{wish_id}/close"), &token)
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn close_wish_not_owner_403() {
    let app = TestApp::new().await;
    let owner_token = setup_aged_user(&app, "owner@test.com").await;
    let wish_id = create_open_wish(&app, &owner_token).await;

    let other_token = setup_aged_user(&app, "other@test.com").await;
    let (status, resp) = app
        .post_with_auth(&format!("/community/wishes/{wish_id}/close"), &other_token)
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_error(&resp, "FORBIDDEN");
}

#[tokio::test]
async fn close_wish_already_closed_400() {
    let app = TestApp::new().await;
    let token = setup_aged_user(&app, "owner@test.com").await;
    let wish_id = create_open_wish(&app, &token).await;

    app.post_with_auth(&format!("/community/wishes/{wish_id}/close"), &token)
        .await;
    let (status, resp) = app
        .post_with_auth(&format!("/community/wishes/{wish_id}/close"), &token)
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_error(&resp, "BAD_REQUEST");
}

#[tokio::test]
async fn close_wish_already_fulfilled_400() {
    let app = TestApp::new().await;
    let token = setup_aged_user(&app, "owner@test.com").await;
    let wish_id = create_open_wish(&app, &token).await;
    sqlx::query(
        "UPDATE community_wishes SET status = 'fulfilled', fulfilled_at = NOW() WHERE id = $1",
    )
    .bind(wish_id)
    .execute(&app.db)
    .await
    .unwrap();

    let (status, resp) = app
        .post_with_auth(&format!("/community/wishes/{wish_id}/close"), &token)
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_error(&resp, "BAD_REQUEST");
}

// ═══════════════════════════════════════════════════════════════════════
// Cat. 8: Offer
// ═══════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn offer_wish_success_204() {
    let app = TestApp::new().await;
    let owner_token = setup_aged_user(&app, "owner@test.com").await;
    let donor_token = setup_aged_user(&app, "donor@test.com").await;
    let donor_id = get_user_id(&app, "donor@test.com").await;
    let wish_id = create_open_wish(&app, &owner_token).await;

    let (status, _) = app
        .post_with_auth(&format!("/community/wishes/{wish_id}/offer"), &donor_token)
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Verify in DB
    let row: (String, Option<Uuid>) =
        sqlx::query_as("SELECT status, matched_with FROM community_wishes WHERE id = $1")
            .bind(wish_id)
            .fetch_one(&app.db)
            .await
            .unwrap();
    assert_eq!(row.0, "matched");
    assert_eq!(row.1, Some(donor_id));
}

#[tokio::test]
async fn offer_wish_self_offer_400() {
    let app = TestApp::new().await;
    let token = setup_aged_user(&app, "owner@test.com").await;
    let wish_id = create_open_wish(&app, &token).await;

    let (status, resp) = app
        .post_with_auth(&format!("/community/wishes/{wish_id}/offer"), &token)
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_error(&resp, "BAD_REQUEST");
}

#[tokio::test]
async fn offer_wish_not_open_400() {
    let app = TestApp::new().await;
    let owner_token = setup_aged_user(&app, "owner@test.com").await;
    let donor_token = setup_aged_user(&app, "donor@test.com").await;
    let wish_id = create_open_wish(&app, &owner_token).await;
    force_wish_status(&app, wish_id, "review").await;

    let (status, resp) = app
        .post_with_auth(&format!("/community/wishes/{wish_id}/offer"), &donor_token)
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_error(&resp, "BAD_REQUEST");
}

#[tokio::test]
async fn offer_wish_account_too_young_403() {
    let app = TestApp::new().await;
    let owner_token = setup_aged_user(&app, "owner@test.com").await;
    let wish_id = create_open_wish(&app, &owner_token).await;

    // Donor is NOT aged
    let donor_token = app.setup_user_token("donor@test.com", TEST_PASSWORD).await;
    let (status, resp) = app
        .post_with_auth(&format!("/community/wishes/{wish_id}/offer"), &donor_token)
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_error(&resp, "FORBIDDEN");
}

#[tokio::test]
async fn offer_wish_not_found_404() {
    let app = TestApp::new().await;
    let donor_token = setup_aged_user(&app, "donor@test.com").await;
    let fake_id = Uuid::new_v4();

    let (status, resp) = app
        .post_with_auth(&format!("/community/wishes/{fake_id}/offer"), &donor_token)
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_error(&resp, "NOT_FOUND");
}

#[tokio::test]
async fn offer_wish_already_matched_400() {
    let app = TestApp::new().await;
    let owner_token = setup_aged_user(&app, "owner@test.com").await;
    let donor1_token = setup_aged_user(&app, "donor1@test.com").await;
    let donor2_token = setup_aged_user(&app, "donor2@test.com").await;
    let wish_id = create_open_wish(&app, &owner_token).await;

    // First offer
    let (s, _) = app
        .post_with_auth(&format!("/community/wishes/{wish_id}/offer"), &donor1_token)
        .await;
    assert_eq!(s, StatusCode::NO_CONTENT);

    // Second offer should fail
    let (status, resp) = app
        .post_with_auth(&format!("/community/wishes/{wish_id}/offer"), &donor2_token)
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_error(&resp, "BAD_REQUEST");
}

#[tokio::test]
async fn offer_wish_on_review_wish_400() {
    let app = TestApp::new().await;
    let owner_token = setup_aged_user(&app, "owner@test.com").await;
    let donor_token = setup_aged_user(&app, "donor@test.com").await;
    let wish_id = create_open_wish(&app, &owner_token).await;
    force_wish_status(&app, wish_id, "review").await;

    let (status, resp) = app
        .post_with_auth(&format!("/community/wishes/{wish_id}/offer"), &donor_token)
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_error(&resp, "BAD_REQUEST");
}

// ═══════════════════════════════════════════════════════════════════════
// Cat. 9: Withdraw offer
// ═══════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn withdraw_offer_success_204() {
    let app = TestApp::new().await;
    let owner_token = setup_aged_user(&app, "owner@test.com").await;
    let donor_token = setup_aged_user(&app, "donor@test.com").await;
    let donor_id = get_user_id(&app, "donor@test.com").await;
    let wish_id = create_open_wish(&app, &owner_token).await;
    force_match(&app, wish_id, donor_id).await;

    let (status, _) = app
        .delete_with_auth(&format!("/community/wishes/{wish_id}/offer"), &donor_token)
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Verify wish is back to open
    let row: (String,) = sqlx::query_as("SELECT status FROM community_wishes WHERE id = $1")
        .bind(wish_id)
        .fetch_one(&app.db)
        .await
        .unwrap();
    assert_eq!(row.0, "open");
}

#[tokio::test]
async fn withdraw_offer_not_donor_403() {
    let app = TestApp::new().await;
    let owner_token = setup_aged_user(&app, "owner@test.com").await;
    let donor_token = setup_aged_user(&app, "donor@test.com").await;
    let donor_id = get_user_id(&app, "donor@test.com").await;
    let wish_id = create_open_wish(&app, &owner_token).await;
    force_match(&app, wish_id, donor_id).await;
    let _ = donor_token;

    let random_token = setup_aged_user(&app, "random@test.com").await;
    let (status, resp) = app
        .delete_with_auth(&format!("/community/wishes/{wish_id}/offer"), &random_token)
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_error(&resp, "FORBIDDEN");
}

#[tokio::test]
async fn withdraw_offer_not_matched_400() {
    let app = TestApp::new().await;
    let owner_token = setup_aged_user(&app, "owner@test.com").await;
    let donor_token = setup_aged_user(&app, "donor@test.com").await;
    let wish_id = create_open_wish(&app, &owner_token).await;

    let (status, resp) = app
        .delete_with_auth(&format!("/community/wishes/{wish_id}/offer"), &donor_token)
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_error(&resp, "BAD_REQUEST");
}

#[tokio::test]
async fn withdraw_offer_not_found_404() {
    let app = TestApp::new().await;
    let token = setup_aged_user(&app, "donor@test.com").await;
    let fake_id = Uuid::new_v4();

    let (status, resp) = app
        .delete_with_auth(&format!("/community/wishes/{fake_id}/offer"), &token)
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_error(&resp, "NOT_FOUND");
}

#[tokio::test]
async fn withdraw_offer_owner_cannot_withdraw_403() {
    let app = TestApp::new().await;
    let owner_token = setup_aged_user(&app, "owner@test.com").await;
    let donor_token = setup_aged_user(&app, "donor@test.com").await;
    let donor_id = get_user_id(&app, "donor@test.com").await;
    let wish_id = create_open_wish(&app, &owner_token).await;
    force_match(&app, wish_id, donor_id).await;
    let _ = donor_token;

    let (status, resp) = app
        .delete_with_auth(&format!("/community/wishes/{wish_id}/offer"), &owner_token)
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_error(&resp, "FORBIDDEN");
}

// ═══════════════════════════════════════════════════════════════════════
// Cat. 10: Reject offer
// ═══════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn reject_offer_success_204() {
    let app = TestApp::new().await;
    let owner_token = setup_aged_user(&app, "owner@test.com").await;
    let donor_token = setup_aged_user(&app, "donor@test.com").await;
    let donor_id = get_user_id(&app, "donor@test.com").await;
    let wish_id = create_open_wish(&app, &owner_token).await;
    force_match(&app, wish_id, donor_id).await;
    let _ = donor_token;

    let (status, _) = app
        .post_with_auth(&format!("/community/wishes/{wish_id}/reject"), &owner_token)
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Verify wish is back to open
    let row: (String,) = sqlx::query_as("SELECT status FROM community_wishes WHERE id = $1")
        .bind(wish_id)
        .fetch_one(&app.db)
        .await
        .unwrap();
    assert_eq!(row.0, "open");
}

#[tokio::test]
async fn reject_offer_not_owner_403() {
    let app = TestApp::new().await;
    let owner_token = setup_aged_user(&app, "owner@test.com").await;
    let donor_token = setup_aged_user(&app, "donor@test.com").await;
    let donor_id = get_user_id(&app, "donor@test.com").await;
    let wish_id = create_open_wish(&app, &owner_token).await;
    force_match(&app, wish_id, donor_id).await;

    let random_token = setup_aged_user(&app, "random@test.com").await;
    let (status, resp) = app
        .post_with_auth(
            &format!("/community/wishes/{wish_id}/reject"),
            &random_token,
        )
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_error(&resp, "FORBIDDEN");
    let _ = donor_token;
}

#[tokio::test]
async fn reject_offer_not_matched_400() {
    let app = TestApp::new().await;
    let owner_token = setup_aged_user(&app, "owner@test.com").await;
    let wish_id = create_open_wish(&app, &owner_token).await;

    let (status, resp) = app
        .post_with_auth(&format!("/community/wishes/{wish_id}/reject"), &owner_token)
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_error(&resp, "BAD_REQUEST");
}

#[tokio::test]
async fn reject_offer_by_donor_403() {
    let app = TestApp::new().await;
    let owner_token = setup_aged_user(&app, "owner@test.com").await;
    let donor_token = setup_aged_user(&app, "donor@test.com").await;
    let donor_id = get_user_id(&app, "donor@test.com").await;
    let wish_id = create_open_wish(&app, &owner_token).await;
    force_match(&app, wish_id, donor_id).await;
    let _ = owner_token;

    let (status, resp) = app
        .post_with_auth(&format!("/community/wishes/{wish_id}/reject"), &donor_token)
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_error(&resp, "FORBIDDEN");
}

#[tokio::test]
async fn reject_offer_not_found_404() {
    let app = TestApp::new().await;
    let token = setup_aged_user(&app, "owner@test.com").await;
    let fake_id = Uuid::new_v4();

    let (status, resp) = app
        .post_with_auth(&format!("/community/wishes/{fake_id}/reject"), &token)
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_error(&resp, "NOT_FOUND");
}

// ═══════════════════════════════════════════════════════════════════════
// Cat. 11: Confirm (fulfill)
// ═══════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn confirm_wish_success_204() {
    let app = TestApp::new().await;
    let owner_token = setup_aged_user(&app, "owner@test.com").await;
    let donor_token = setup_aged_user(&app, "donor@test.com").await;
    let donor_id = get_user_id(&app, "donor@test.com").await;
    let wish_id = create_open_wish(&app, &owner_token).await;
    force_match(&app, wish_id, donor_id).await;
    let _ = donor_token;

    let (status, _) = app
        .post_with_auth(
            &format!("/community/wishes/{wish_id}/confirm"),
            &owner_token,
        )
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Verify in DB
    let row: (String, Option<chrono::DateTime<chrono::Utc>>) =
        sqlx::query_as("SELECT status, fulfilled_at FROM community_wishes WHERE id = $1")
            .bind(wish_id)
            .fetch_one(&app.db)
            .await
            .unwrap();
    assert_eq!(row.0, "fulfilled");
    assert!(row.1.is_some(), "fulfilled_at should be set");
}

#[tokio::test]
async fn confirm_wish_not_owner_403() {
    let app = TestApp::new().await;
    let owner_token = setup_aged_user(&app, "owner@test.com").await;
    let donor_token = setup_aged_user(&app, "donor@test.com").await;
    let donor_id = get_user_id(&app, "donor@test.com").await;
    let wish_id = create_open_wish(&app, &owner_token).await;
    force_match(&app, wish_id, donor_id).await;

    let random_token = setup_aged_user(&app, "random@test.com").await;
    let (status, resp) = app
        .post_with_auth(
            &format!("/community/wishes/{wish_id}/confirm"),
            &random_token,
        )
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_error(&resp, "FORBIDDEN");
    let _ = donor_token;
}

#[tokio::test]
async fn confirm_wish_not_matched_400() {
    let app = TestApp::new().await;
    let token = setup_aged_user(&app, "owner@test.com").await;
    let wish_id = create_open_wish(&app, &token).await;

    let (status, resp) = app
        .post_with_auth(&format!("/community/wishes/{wish_id}/confirm"), &token)
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_error(&resp, "BAD_REQUEST");
}

#[tokio::test]
async fn confirm_wish_by_donor_403() {
    let app = TestApp::new().await;
    let owner_token = setup_aged_user(&app, "owner@test.com").await;
    let donor_token = setup_aged_user(&app, "donor@test.com").await;
    let donor_id = get_user_id(&app, "donor@test.com").await;
    let wish_id = create_open_wish(&app, &owner_token).await;
    force_match(&app, wish_id, donor_id).await;
    let _ = owner_token;

    let (status, resp) = app
        .post_with_auth(
            &format!("/community/wishes/{wish_id}/confirm"),
            &donor_token,
        )
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_error(&resp, "FORBIDDEN");
}

#[tokio::test]
async fn confirm_wish_not_found_404() {
    let app = TestApp::new().await;
    let token = setup_aged_user(&app, "owner@test.com").await;
    let fake_id = Uuid::new_v4();

    let (status, resp) = app
        .post_with_auth(&format!("/community/wishes/{fake_id}/confirm"), &token)
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_error(&resp, "NOT_FOUND");
}

// ═══════════════════════════════════════════════════════════════════════
// Cat. 12: Report
// ═══════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn report_wish_success_204() {
    let app = TestApp::new().await;
    let owner_token = setup_aged_user(&app, "owner@test.com").await;
    let wish_id = create_open_wish(&app, &owner_token).await;

    let reporter_token = setup_aged_user(&app, "reporter@test.com").await;
    let body = json!({ "reason": "spam" });
    let (status, _) = app
        .post_json_with_auth(
            &format!("/community/wishes/{wish_id}/report"),
            &body,
            &reporter_token,
        )
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn report_wish_default_reason_204() {
    let app = TestApp::new().await;
    let owner_token = setup_aged_user(&app, "owner@test.com").await;
    let wish_id = create_open_wish(&app, &owner_token).await;

    let reporter_token = setup_aged_user(&app, "reporter@test.com").await;
    let body = json!({});
    let (status, _) = app
        .post_json_with_auth(
            &format!("/community/wishes/{wish_id}/report"),
            &body,
            &reporter_token,
        )
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Verify in DB the report was created with "inappropriate"
    let reporter_id = get_user_id(&app, "reporter@test.com").await;
    let row: (String,) =
        sqlx::query_as("SELECT reason FROM wish_reports WHERE wish_id = $1 AND reporter_id = $2")
            .bind(wish_id)
            .bind(reporter_id)
            .fetch_one(&app.db)
            .await
            .unwrap();
    assert_eq!(row.0, "inappropriate");
}

#[tokio::test]
async fn report_wish_self_report_400() {
    let app = TestApp::new().await;
    let owner_token = setup_aged_user(&app, "owner@test.com").await;
    let wish_id = create_open_wish(&app, &owner_token).await;

    let body = json!({ "reason": "spam" });
    let (status, resp) = app
        .post_json_with_auth(
            &format!("/community/wishes/{wish_id}/report"),
            &body,
            &owner_token,
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_error(&resp, "BAD_REQUEST");
}

#[tokio::test]
async fn report_wish_account_too_young_403() {
    let app = TestApp::new().await;
    let owner_token = setup_aged_user(&app, "owner@test.com").await;
    let wish_id = create_open_wish(&app, &owner_token).await;

    // Reporter NOT aged
    let reporter_token = app
        .setup_user_token("reporter@test.com", TEST_PASSWORD)
        .await;
    let body = json!({ "reason": "spam" });
    let (status, resp) = app
        .post_json_with_auth(
            &format!("/community/wishes/{wish_id}/report"),
            &body,
            &reporter_token,
        )
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_error(&resp, "FORBIDDEN");
}

#[tokio::test]
async fn report_wish_not_open_400() {
    let app = TestApp::new().await;
    let owner_token = setup_aged_user(&app, "owner@test.com").await;
    let wish_id = create_open_wish(&app, &owner_token).await;
    force_wish_status(&app, wish_id, "matched").await;

    let reporter_token = setup_aged_user(&app, "reporter@test.com").await;
    let body = json!({ "reason": "spam" });
    let (status, resp) = app
        .post_json_with_auth(
            &format!("/community/wishes/{wish_id}/report"),
            &body,
            &reporter_token,
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_error(&resp, "BAD_REQUEST");
}

#[tokio::test]
async fn report_wish_duplicate_409() {
    let app = TestApp::new().await;
    let owner_token = setup_aged_user(&app, "owner@test.com").await;
    let wish_id = create_open_wish(&app, &owner_token).await;

    let reporter_token = setup_aged_user(&app, "reporter@test.com").await;
    let body = json!({ "reason": "spam" });

    // First report
    let (s, _) = app
        .post_json_with_auth(
            &format!("/community/wishes/{wish_id}/report"),
            &body,
            &reporter_token,
        )
        .await;
    assert_eq!(s, StatusCode::NO_CONTENT);

    // Second report
    let (status, resp) = app
        .post_json_with_auth(
            &format!("/community/wishes/{wish_id}/report"),
            &body,
            &reporter_token,
        )
        .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_error(&resp, "CONFLICT");
}

#[tokio::test]
async fn report_wish_daily_limit_reached_400() {
    let app = TestApp::new().await;

    // The reporter who will hit the limit
    let reporter_token = setup_aged_user(&app, "reporter@test.com").await;

    // Create 11 wishes from 11 different owners, report 10 of them
    for i in 0..11 {
        let email = format!("owner{i}@test.com");
        let owner_token = setup_aged_user(&app, &email).await;
        let wish_id = create_open_wish(&app, &owner_token).await;

        let body = json!({ "reason": "spam" });
        let (status, resp) = app
            .post_json_with_auth(
                &format!("/community/wishes/{wish_id}/report"),
                &body,
                &reporter_token,
            )
            .await;

        if i < 10 {
            assert_eq!(
                status,
                StatusCode::NO_CONTENT,
                "report {i} should succeed: {resp}"
            );
        } else {
            // 11th report should be rejected
            assert_eq!(
                status,
                StatusCode::BAD_REQUEST,
                "report {i} should be rejected: {resp}"
            );
            assert_error(&resp, "BAD_REQUEST");
        }
    }
}

#[tokio::test]
async fn report_wish_invalid_reason_400() {
    let app = TestApp::new().await;
    let owner_token = setup_aged_user(&app, "owner@test.com").await;
    let wish_id = create_open_wish(&app, &owner_token).await;

    let reporter_token = setup_aged_user(&app, "reporter@test.com").await;
    let body = json!({ "reason": "invalid_reason" });
    let (status, resp) = app
        .post_json_with_auth(
            &format!("/community/wishes/{wish_id}/report"),
            &body,
            &reporter_token,
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_error(&resp, "BAD_REQUEST");
}

#[tokio::test]
async fn report_wish_threshold_triggers_review() {
    let app = TestApp::new().await;
    let owner_token = setup_aged_user(&app, "owner@test.com").await;
    let wish_id = create_open_wish(&app, &owner_token).await;

    // 5 unique reporters
    for i in 0..5 {
        let email = format!("reporter{i}@test.com");
        let reporter_token = setup_aged_user(&app, &email).await;
        let body = json!({ "reason": "spam" });
        let (s, _) = app
            .post_json_with_auth(
                &format!("/community/wishes/{wish_id}/report"),
                &body,
                &reporter_token,
            )
            .await;
        assert_eq!(s, StatusCode::NO_CONTENT);
    }

    // Verify the wish moved to review
    let row: (String,) = sqlx::query_as("SELECT status FROM community_wishes WHERE id = $1")
        .bind(wish_id)
        .fetch_one(&app.db)
        .await
        .unwrap();
    assert_eq!(row.0, "review");
}

#[tokio::test]
async fn report_wish_not_found_404() {
    let app = TestApp::new().await;
    let reporter_token = setup_aged_user(&app, "reporter@test.com").await;
    let fake_id = Uuid::new_v4();

    let body = json!({ "reason": "spam" });
    let (status, resp) = app
        .post_json_with_auth(
            &format!("/community/wishes/{fake_id}/report"),
            &body,
            &reporter_token,
        )
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_error(&resp, "NOT_FOUND");
}

#[tokio::test]
async fn report_wish_4_reports_stays_open() {
    let app = TestApp::new().await;
    let owner_token = setup_aged_user(&app, "owner@test.com").await;
    let wish_id = create_open_wish(&app, &owner_token).await;

    // Only 4 reporters (threshold is 5)
    for i in 0..4 {
        let email = format!("reporter{i}@test.com");
        let reporter_token = setup_aged_user(&app, &email).await;
        let body = json!({ "reason": "spam" });
        let (s, _) = app
            .post_json_with_auth(
                &format!("/community/wishes/{wish_id}/report"),
                &body,
                &reporter_token,
            )
            .await;
        assert_eq!(s, StatusCode::NO_CONTENT);
    }

    let row: (String,) = sqlx::query_as("SELECT status FROM community_wishes WHERE id = $1")
        .bind(wish_id)
        .fetch_one(&app.db)
        .await
        .unwrap();
    assert_eq!(row.0, "open", "4 reports should NOT trigger review");
}

// ═══════════════════════════════════════════════════════════════════════
// Cat. 13: Reopen
// ═══════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn reopen_wish_from_review_204() {
    let app = TestApp::new().await;
    let token = setup_aged_user(&app, "owner@test.com").await;
    let wish_id = create_open_wish(&app, &token).await;
    force_wish_status(&app, wish_id, "review").await;

    let (status, _) = app
        .post_with_auth(&format!("/community/wishes/{wish_id}/reopen"), &token)
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Verify reopen_count and status in DB
    let row: (String, i32) =
        sqlx::query_as("SELECT status, reopen_count FROM community_wishes WHERE id = $1")
            .bind(wish_id)
            .fetch_one(&app.db)
            .await
            .unwrap();
    assert_eq!(row.0, "open");
    assert_eq!(row.1, 1);
}

#[tokio::test]
async fn reopen_wish_not_review_400() {
    let app = TestApp::new().await;
    let token = setup_aged_user(&app, "owner@test.com").await;
    let wish_id = create_open_wish(&app, &token).await;

    let (status, resp) = app
        .post_with_auth(&format!("/community/wishes/{wish_id}/reopen"), &token)
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_error(&resp, "BAD_REQUEST");
}

#[tokio::test]
async fn reopen_wish_not_owner_403() {
    let app = TestApp::new().await;
    let owner_token = setup_aged_user(&app, "owner@test.com").await;
    let wish_id = create_open_wish(&app, &owner_token).await;
    force_wish_status(&app, wish_id, "review").await;

    let other_token = setup_aged_user(&app, "other@test.com").await;
    let (status, resp) = app
        .post_with_auth(&format!("/community/wishes/{wish_id}/reopen"), &other_token)
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_error(&resp, "FORBIDDEN");
}

#[tokio::test]
async fn reopen_wish_max_reopens_reached_403() {
    let app = TestApp::new().await;
    let token = setup_aged_user(&app, "owner@test.com").await;
    let wish_id = create_open_wish(&app, &token).await;

    // Set reopen_count = 2 (max) and status = review
    sqlx::query("UPDATE community_wishes SET reopen_count = 2, status = 'review' WHERE id = $1")
        .bind(wish_id)
        .execute(&app.db)
        .await
        .unwrap();

    let (status, resp) = app
        .post_with_auth(&format!("/community/wishes/{wish_id}/reopen"), &token)
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_error(&resp, "FORBIDDEN");
}

#[tokio::test]
async fn reopen_wish_cooldown_not_elapsed_400() {
    let app = TestApp::new().await;
    let token = setup_aged_user(&app, "owner@test.com").await;
    let wish_id = create_open_wish(&app, &token).await;

    // Set status=review, reopen_count=1, last_reopen_at=NOW() (within cooldown)
    sqlx::query(
        "UPDATE community_wishes SET reopen_count = 1, status = 'review', last_reopen_at = NOW() WHERE id = $1",
    )
    .bind(wish_id)
    .execute(&app.db)
    .await
    .unwrap();

    let (status, resp) = app
        .post_with_auth(&format!("/community/wishes/{wish_id}/reopen"), &token)
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_error(&resp, "BAD_REQUEST");
}

#[tokio::test]
async fn reopen_wish_clears_reports() {
    let app = TestApp::new().await;
    let owner_token = setup_aged_user(&app, "owner@test.com").await;
    let wish_id = create_open_wish(&app, &owner_token).await;

    // Create some reports to reach threshold
    for i in 0..5 {
        let email = format!("reporter{i}@test.com");
        let reporter_token = setup_aged_user(&app, &email).await;
        let body = json!({ "reason": "spam" });
        app.post_json_with_auth(
            &format!("/community/wishes/{wish_id}/report"),
            &body,
            &reporter_token,
        )
        .await;
    }

    // Wish should now be in review
    let row: (String,) = sqlx::query_as("SELECT status FROM community_wishes WHERE id = $1")
        .bind(wish_id)
        .fetch_one(&app.db)
        .await
        .unwrap();
    assert_eq!(row.0, "review");

    // Reopen
    let (status, _) = app
        .post_with_auth(&format!("/community/wishes/{wish_id}/reopen"), &owner_token)
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Verify reports are cleared
    let row: (i32,) = sqlx::query_as("SELECT report_count FROM community_wishes WHERE id = $1")
        .bind(wish_id)
        .fetch_one(&app.db)
        .await
        .unwrap();
    assert_eq!(row.0, 0, "report_count should be 0 after reopen");

    let report_count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM wish_reports WHERE wish_id = $1")
            .bind(wish_id)
            .fetch_one(&app.db)
            .await
            .unwrap();
    assert_eq!(report_count.0, 0, "wish_reports should be deleted");
}

#[tokio::test]
async fn reopen_wish_not_found_404() {
    let app = TestApp::new().await;
    let token = setup_aged_user(&app, "owner@test.com").await;
    let fake_id = Uuid::new_v4();

    let (status, resp) = app
        .post_with_auth(&format!("/community/wishes/{fake_id}/reopen"), &token)
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_error(&resp, "NOT_FOUND");
}

// ═══════════════════════════════════════════════════════════════════════
// Cat. 14: Admin
// ═══════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn admin_list_pending_without_auth_401() {
    let app = TestApp::new().await;
    let (status, _) = app.get_no_auth("/admin/wishes/pending").await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn admin_list_pending_non_admin_403() {
    let app = TestApp::new().await;
    let token = setup_aged_user(&app, "user@test.com").await;

    let (status, resp) = app.get_with_auth("/admin/wishes/pending", &token).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_error(&resp, "FORBIDDEN");
}

#[tokio::test]
async fn admin_list_pending_returns_flagged_and_review_200() {
    let app = TestApp::new().await;
    let admin_token = setup_aged_user(&app, "admin@test.com").await;
    make_admin(&app, "admin@test.com").await;

    // Create wishes and set various statuses
    let owner_token = setup_aged_user(&app, "owner@test.com").await;
    let wish_id = create_open_wish(&app, &owner_token).await;
    force_wish_status(&app, wish_id, "flagged").await;

    let owner2_token = setup_aged_user(&app, "owner2@test.com").await;
    let wish_id2 = create_open_wish(&app, &owner2_token).await;
    force_wish_status(&app, wish_id2, "review").await;

    // Open wish should NOT appear
    let owner3_token = setup_aged_user(&app, "owner3@test.com").await;
    create_open_wish(&app, &owner3_token).await;

    let (status, body) = app
        .get_with_auth("/admin/wishes/pending", &admin_token)
        .await;
    assert_eq!(status, StatusCode::OK);
    let wishes = body["data"].as_array().unwrap();
    assert_eq!(wishes.len(), 2);
    let statuses: Vec<&str> = wishes
        .iter()
        .map(|w| w["status"].as_str().unwrap())
        .collect();
    assert!(statuses.contains(&"flagged"));
    assert!(statuses.contains(&"review"));
    assert!(body["pagination"]["total"].as_i64().unwrap() >= 2);
}

#[tokio::test]
async fn admin_list_pending_empty_200() {
    let app = TestApp::new().await;
    let admin_token = setup_aged_user(&app, "admin@test.com").await;
    make_admin(&app, "admin@test.com").await;

    let (status, body) = app
        .get_with_auth("/admin/wishes/pending", &admin_token)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"].as_array().unwrap().len(), 0);
    assert_eq!(body["pagination"]["total"].as_i64().unwrap(), 0);
}

#[tokio::test]
async fn admin_approve_wish_204() {
    let app = TestApp::new().await;
    let admin_token = setup_aged_user(&app, "admin@test.com").await;
    make_admin(&app, "admin@test.com").await;

    let owner_token = setup_aged_user(&app, "owner@test.com").await;
    let wish_id = create_open_wish(&app, &owner_token).await;
    force_wish_status(&app, wish_id, "flagged").await;

    let (status, _) = app
        .post_with_auth(&format!("/admin/wishes/{wish_id}/approve"), &admin_token)
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let row: (String,) = sqlx::query_as("SELECT status FROM community_wishes WHERE id = $1")
        .bind(wish_id)
        .fetch_one(&app.db)
        .await
        .unwrap();
    assert_eq!(row.0, "open");
}

#[tokio::test]
async fn admin_approve_review_wish_204() {
    let app = TestApp::new().await;
    let admin_token = setup_aged_user(&app, "admin@test.com").await;
    make_admin(&app, "admin@test.com").await;

    let owner_token = setup_aged_user(&app, "owner@test.com").await;
    let wish_id = create_open_wish(&app, &owner_token).await;
    force_wish_status(&app, wish_id, "review").await;

    let (status, _) = app
        .post_with_auth(&format!("/admin/wishes/{wish_id}/approve"), &admin_token)
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let row: (String,) = sqlx::query_as("SELECT status FROM community_wishes WHERE id = $1")
        .bind(wish_id)
        .fetch_one(&app.db)
        .await
        .unwrap();
    assert_eq!(row.0, "open");
}

#[tokio::test]
async fn admin_approve_open_wish_400() {
    let app = TestApp::new().await;
    let admin_token = setup_aged_user(&app, "admin@test.com").await;
    make_admin(&app, "admin@test.com").await;

    let owner_token = setup_aged_user(&app, "owner@test.com").await;
    let wish_id = create_open_wish(&app, &owner_token).await;

    let (status, resp) = app
        .post_with_auth(&format!("/admin/wishes/{wish_id}/approve"), &admin_token)
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_error(&resp, "BAD_REQUEST");
}

#[tokio::test]
async fn admin_reject_wish_204() {
    let app = TestApp::new().await;
    let admin_token = setup_aged_user(&app, "admin@test.com").await;
    make_admin(&app, "admin@test.com").await;

    let owner_token = setup_aged_user(&app, "owner@test.com").await;
    let wish_id = create_open_wish(&app, &owner_token).await;
    force_wish_status(&app, wish_id, "flagged").await;

    let (status, _) = app
        .post_with_auth(&format!("/admin/wishes/{wish_id}/reject"), &admin_token)
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let row: (String,) = sqlx::query_as("SELECT status FROM community_wishes WHERE id = $1")
        .bind(wish_id)
        .fetch_one(&app.db)
        .await
        .unwrap();
    assert_eq!(row.0, "rejected");
}

#[tokio::test]
async fn admin_reject_non_admin_403() {
    let app = TestApp::new().await;
    let user_token = setup_aged_user(&app, "user@test.com").await;

    let owner_token = setup_aged_user(&app, "owner@test.com").await;
    let wish_id = create_open_wish(&app, &owner_token).await;
    force_wish_status(&app, wish_id, "flagged").await;

    let (status, resp) = app
        .post_with_auth(&format!("/admin/wishes/{wish_id}/reject"), &user_token)
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_error(&resp, "FORBIDDEN");
}

// ═══════════════════════════════════════════════════════════════════════
// Cat. 15: E2E Lifecycle
// ═══════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn e2e_full_lifecycle_create_offer_confirm() {
    let app = TestApp::new().await;
    let owner_token = setup_aged_user_with_name(&app, "owner@test.com", "Alice").await;
    let donor_token = setup_aged_user_with_name(&app, "donor@test.com", "Bob").await;

    // 1. Create
    let body = json!({
        "title": "Need school books",
        "category": "education",
        "is_anonymous": false,
    });
    let (status, resp) = app
        .post_json_with_auth("/community/wishes", &body, &owner_token)
        .await;
    assert_eq!(status, StatusCode::CREATED);
    let wish_id = Uuid::parse_str(resp["id"].as_str().unwrap()).unwrap();
    wait_for_wish_status(&app, wish_id, "open").await;

    // 2. Offer
    let (status, _) = app
        .post_with_auth(&format!("/community/wishes/{wish_id}/offer"), &donor_token)
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // 3. Verify matched via get_wish
    let (status, body) = app
        .get_with_auth(&format!("/community/wishes/{wish_id}"), &owner_token)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"].as_str(), Some("matched"));
    assert_eq!(body["matched_with_display_name"].as_str(), Some("Bob"));

    // 4. Confirm
    let (status, _) = app
        .post_with_auth(
            &format!("/community/wishes/{wish_id}/confirm"),
            &owner_token,
        )
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // 5. Verify fulfilled via list_my_wishes
    let (status, body) = app
        .get_with_auth("/community/wishes/mine", &owner_token)
        .await;
    assert_eq!(status, StatusCode::OK);
    let wishes = body.as_array().unwrap();
    assert_eq!(wishes[0]["status"].as_str(), Some("fulfilled"));
    assert!(wishes[0]["fulfilled_at"].as_str().is_some());
}

#[tokio::test]
async fn e2e_offer_reject_re_offer_confirm() {
    let app = TestApp::new().await;
    let owner_token = setup_aged_user(&app, "owner@test.com").await;
    let donor_a_token = setup_aged_user(&app, "donora@test.com").await;
    let donor_b_token = setup_aged_user(&app, "donorb@test.com").await;

    let wish_id = create_open_wish(&app, &owner_token).await;

    // Offer A
    let (s, _) = app
        .post_with_auth(
            &format!("/community/wishes/{wish_id}/offer"),
            &donor_a_token,
        )
        .await;
    assert_eq!(s, StatusCode::NO_CONTENT);

    // Reject A
    let (s, _) = app
        .post_with_auth(&format!("/community/wishes/{wish_id}/reject"), &owner_token)
        .await;
    assert_eq!(s, StatusCode::NO_CONTENT);

    // Offer B
    let (s, _) = app
        .post_with_auth(
            &format!("/community/wishes/{wish_id}/offer"),
            &donor_b_token,
        )
        .await;
    assert_eq!(s, StatusCode::NO_CONTENT);

    // Confirm B
    let (s, _) = app
        .post_with_auth(
            &format!("/community/wishes/{wish_id}/confirm"),
            &owner_token,
        )
        .await;
    assert_eq!(s, StatusCode::NO_CONTENT);

    let row: (String,) = sqlx::query_as("SELECT status FROM community_wishes WHERE id = $1")
        .bind(wish_id)
        .fetch_one(&app.db)
        .await
        .unwrap();
    assert_eq!(row.0, "fulfilled");
}

#[tokio::test]
async fn e2e_report_review_reopen_cycle() {
    let app = TestApp::new().await;
    let owner_token = setup_aged_user(&app, "owner@test.com").await;
    let wish_id = create_open_wish(&app, &owner_token).await;

    // 5 reports → review
    for i in 0..5 {
        let email = format!("reporter{i}@test.com");
        let reporter_token = setup_aged_user(&app, &email).await;
        let body = json!({ "reason": "spam" });
        app.post_json_with_auth(
            &format!("/community/wishes/{wish_id}/report"),
            &body,
            &reporter_token,
        )
        .await;
    }

    let row: (String,) = sqlx::query_as("SELECT status FROM community_wishes WHERE id = $1")
        .bind(wish_id)
        .fetch_one(&app.db)
        .await
        .unwrap();
    assert_eq!(row.0, "review");

    // Reopen
    let (status, _) = app
        .post_with_auth(&format!("/community/wishes/{wish_id}/reopen"), &owner_token)
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let row: (String, i32) =
        sqlx::query_as("SELECT status, reopen_count FROM community_wishes WHERE id = $1")
            .bind(wish_id)
            .fetch_one(&app.db)
            .await
            .unwrap();
    assert_eq!(row.0, "open");
    assert_eq!(row.1, 1);
}

#[tokio::test]
async fn e2e_create_close_create_new() {
    let app = TestApp::new().await;
    let token = setup_aged_user(&app, "owner@test.com").await;

    // First wish
    let wish_id = create_open_wish(&app, &token).await;

    // Close it
    let (s, _) = app
        .post_with_auth(&format!("/community/wishes/{wish_id}/close"), &token)
        .await;
    assert_eq!(s, StatusCode::NO_CONTENT);

    // Second wish should succeed
    let body = json!({ "title": "New wish", "category": "health", "is_anonymous": true });
    let (status, resp) = app
        .post_json_with_auth("/community/wishes", &body, &token)
        .await;
    assert_eq!(status, StatusCode::CREATED);

    let wish_id2 = Uuid::parse_str(resp["id"].as_str().unwrap()).unwrap();
    wait_for_wish_status(&app, wish_id2, "open").await;

    // list_my_wishes should show both
    let (_, body) = app.get_with_auth("/community/wishes/mine", &token).await;
    assert_eq!(body.as_array().unwrap().len(), 2);
}

// ═══════════════════════════════════════════════════════════════════════
// Cat. 16: Cache
// ═══════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn list_wishes_cache_invalidated_after_create() {
    let app = TestApp::new().await;

    // List → 0 wishes (possibly cached)
    let (_, body) = app.get_no_auth("/community/wishes").await;
    assert_eq!(body["data"].as_array().unwrap().len(), 0);

    // Create a wish
    let token = setup_aged_user(&app, "owner@test.com").await;
    let wish_id = create_open_wish(&app, &token).await;
    let _ = wish_id;

    // List again → should see the wish (cache was invalidated)
    let (_, body) = app.get_no_auth("/community/wishes").await;
    assert_eq!(body["data"].as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn list_wishes_cache_invalidated_after_close() {
    let app = TestApp::new().await;
    let token = setup_aged_user(&app, "owner@test.com").await;
    let wish_id = create_open_wish(&app, &token).await;

    // List → 1 wish
    let (_, body) = app.get_no_auth("/community/wishes").await;
    assert_eq!(body["data"].as_array().unwrap().len(), 1);

    // Close it
    app.post_with_auth(&format!("/community/wishes/{wish_id}/close"), &token)
        .await;

    // List again → 0 wishes
    let (_, body) = app.get_no_auth("/community/wishes").await;
    assert_eq!(body["data"].as_array().unwrap().len(), 0);
}

// ═══════════════════════════════════════════════════════════════════════
// Cat. 12: image_url & links
// ═══════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn create_wish_with_image_url_201() {
    let app = TestApp::new().await;
    let token = setup_aged_user(&app, "owner@test.com").await;

    let body = json!({
        "title": "Need winter coat",
        "category": "clothing",
        "is_anonymous": true,
        "image_url": "https://example.com/coat.jpg",
    });
    let (status, resp) = app
        .post_json_with_auth("/community/wishes", &body, &token)
        .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(
        resp["image_url"].as_str(),
        Some("https://example.com/coat.jpg")
    );
}

#[tokio::test]
async fn create_wish_with_links_201() {
    let app = TestApp::new().await;
    let token = setup_aged_user(&app, "owner@test.com").await;

    let body = json!({
        "title": "Need winter coat",
        "category": "clothing",
        "is_anonymous": true,
        "links": ["https://shop.com/coat1", "https://shop.com/coat2"],
    });
    let (status, resp) = app
        .post_json_with_auth("/community/wishes", &body, &token)
        .await;
    assert_eq!(status, StatusCode::CREATED);
    let links = resp["links"].as_array().unwrap();
    assert_eq!(links.len(), 2);
    assert_eq!(links[0].as_str(), Some("https://shop.com/coat1"));
    assert_eq!(links[1].as_str(), Some("https://shop.com/coat2"));
}

#[tokio::test]
async fn create_wish_links_too_many_400() {
    let app = TestApp::new().await;
    let token = setup_aged_user(&app, "owner@test.com").await;

    let links: Vec<String> = (0..11)
        .map(|i| format!("https://shop.com/item{i}"))
        .collect();
    let body = json!({
        "title": "Need winter coat",
        "category": "clothing",
        "is_anonymous": true,
        "links": links,
    });
    let (status, _) = app
        .post_json_with_auth("/community/wishes", &body, &token)
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn create_wish_image_url_too_long_400() {
    let app = TestApp::new().await;
    let token = setup_aged_user(&app, "owner@test.com").await;

    let long_url = format!("https://example.com/{}", "a".repeat(2049));
    let body = json!({
        "title": "Need winter coat",
        "category": "clothing",
        "is_anonymous": true,
        "image_url": long_url,
    });
    let (status, _) = app
        .post_json_with_auth("/community/wishes", &body, &token)
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn update_wish_image_url_200() {
    let app = TestApp::new().await;
    let token = setup_aged_user(&app, "owner@test.com").await;
    let wish_id = create_open_wish(&app, &token).await;

    let body = json!({
        "image_url": "https://example.com/updated.jpg",
    });
    let (status, resp) = app
        .patch_json_with_auth(&format!("/community/wishes/{wish_id}"), &body, &token)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        resp["image_url"].as_str(),
        Some("https://example.com/updated.jpg")
    );
}

#[tokio::test]
async fn list_wishes_includes_image_and_links() {
    let app = TestApp::new().await;
    let token = setup_aged_user(&app, "owner@test.com").await;

    let body = json!({
        "title": "Need winter coat",
        "category": "clothing",
        "is_anonymous": true,
        "image_url": "https://example.com/coat.jpg",
        "links": ["https://shop.com/coat1"],
    });
    let (status, resp) = app
        .post_json_with_auth("/community/wishes", &body, &token)
        .await;
    assert_eq!(status, StatusCode::CREATED);

    let wish_id = Uuid::parse_str(resp["id"].as_str().unwrap()).unwrap();
    wait_for_wish_status(&app, wish_id, "open").await;

    // List endpoint should include the fields
    let (status, body) = app.get_no_auth("/community/wishes").await;
    assert_eq!(status, StatusCode::OK);
    let wishes = body["data"].as_array().unwrap();
    assert_eq!(wishes.len(), 1);
    assert_eq!(
        wishes[0]["image_url"].as_str(),
        Some("https://example.com/coat.jpg")
    );
    let links = wishes[0]["links"].as_array().unwrap();
    assert_eq!(links.len(), 1);
    assert_eq!(links[0].as_str(), Some("https://shop.com/coat1"));

    // Detail endpoint too
    let (status, detail) = app
        .get_no_auth(&format!("/community/wishes/{wish_id}"))
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        detail["image_url"].as_str(),
        Some("https://example.com/coat.jpg")
    );
    let detail_links = detail["links"].as_array().unwrap();
    assert_eq!(detail_links.len(), 1);
    assert_eq!(detail_links[0].as_str(), Some("https://shop.com/coat1"));
}

#[tokio::test]
async fn create_wish_with_empty_links_array_201() {
    let app = TestApp::new().await;
    let token = setup_aged_user(&app, "owner@test.com").await;

    let body = json!({
        "title": "Need winter coat",
        "category": "clothing",
        "is_anonymous": true,
        "links": [],
    });
    let (status, resp) = app
        .post_json_with_auth("/community/wishes", &body, &token)
        .await;
    assert_eq!(status, StatusCode::CREATED);
    assert!(resp["links"].is_null() || resp["links"].as_array().is_none_or(|a| a.is_empty()));
}

#[tokio::test]
async fn create_wish_link_individual_too_long_400() {
    let app = TestApp::new().await;
    let token = setup_aged_user(&app, "owner@test.com").await;

    let long_link = format!("https://example.com/{}", "a".repeat(2049));
    let body = json!({
        "title": "Need winter coat",
        "category": "clothing",
        "is_anonymous": true,
        "links": [long_link],
    });
    let (status, _) = app
        .post_json_with_auth("/community/wishes", &body, &token)
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn create_wish_with_exactly_10_links_201() {
    let app = TestApp::new().await;
    let token = setup_aged_user(&app, "owner@test.com").await;

    let links: Vec<String> = (0..10)
        .map(|i| format!("https://shop.com/item{i}"))
        .collect();
    let body = json!({
        "title": "Need winter coat",
        "category": "clothing",
        "is_anonymous": true,
        "links": links,
    });
    let (status, resp) = app
        .post_json_with_auth("/community/wishes", &body, &token)
        .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(resp["links"].as_array().unwrap().len(), 10);
}

#[tokio::test]
async fn update_wish_links_200() {
    let app = TestApp::new().await;
    let token = setup_aged_user(&app, "owner@test.com").await;
    let wish_id = create_open_wish(&app, &token).await;

    let body = json!({
        "links": ["https://shop.com/updated1", "https://shop.com/updated2"],
    });
    let (status, resp) = app
        .patch_json_with_auth(&format!("/community/wishes/{wish_id}"), &body, &token)
        .await;
    assert_eq!(status, StatusCode::OK);
    let links = resp["links"].as_array().unwrap();
    assert_eq!(links.len(), 2);
    assert_eq!(links[0].as_str(), Some("https://shop.com/updated1"));
}

#[tokio::test]
async fn list_my_wishes_includes_image_url_and_links() {
    let app = TestApp::new().await;
    let token = setup_aged_user(&app, "owner@test.com").await;

    let body = json!({
        "title": "Need winter coat",
        "category": "clothing",
        "is_anonymous": true,
        "image_url": "https://example.com/coat.jpg",
        "links": ["https://shop.com/coat1"],
    });
    let (status, _) = app
        .post_json_with_auth("/community/wishes", &body, &token)
        .await;
    assert_eq!(status, StatusCode::CREATED);

    let (status, resp) = app.get_with_auth("/community/wishes/mine", &token).await;
    assert_eq!(status, StatusCode::OK);
    let wishes = resp.as_array().unwrap();
    assert_eq!(wishes.len(), 1);
    assert_eq!(
        wishes[0]["image_url"].as_str(),
        Some("https://example.com/coat.jpg")
    );
    let links = wishes[0]["links"].as_array().unwrap();
    assert_eq!(links.len(), 1);
}

#[tokio::test]
async fn admin_list_pending_includes_image_url_and_links() {
    let app = TestApp::new().await;
    let token = setup_aged_user(&app, "owner@test.com").await;
    make_admin(&app, "owner@test.com").await;

    let body = json!({
        "title": "Need winter coat",
        "category": "clothing",
        "is_anonymous": true,
        "image_url": "https://example.com/coat.jpg",
        "links": ["https://shop.com/coat1"],
    });
    let (status, resp) = app
        .post_json_with_auth("/community/wishes", &body, &token)
        .await;
    assert_eq!(status, StatusCode::CREATED);
    let wish_id = Uuid::parse_str(resp["id"].as_str().unwrap()).unwrap();
    wait_for_wish_status(&app, wish_id, "open").await;
    force_wish_status(&app, wish_id, "flagged").await;

    let (status, resp) = app.get_with_auth("/admin/wishes/pending", &token).await;
    assert_eq!(status, StatusCode::OK);
    let wishes = resp["data"].as_array().unwrap();
    assert_eq!(wishes.len(), 1);
    assert_eq!(
        wishes[0]["image_url"].as_str(),
        Some("https://example.com/coat.jpg")
    );
    let links = wishes[0]["links"].as_array().unwrap();
    assert_eq!(links.len(), 1);
}

// ═══════════════════════════════════════════════════════════════════════
// Cat. 13: Edge cases
// ═══════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn create_wish_at_two_active_allows_third_201() {
    let app = TestApp::new().await;
    let token = setup_aged_user(&app, "owner@test.com").await;

    // Create 2 open wishes
    create_open_wish(&app, &token).await;
    create_open_wish(&app, &token).await;

    // Third should succeed (limit is 3)
    let body = json!({
        "title": "Third wish",
        "category": "clothing",
        "is_anonymous": true,
    });
    let (status, _) = app
        .post_json_with_auth("/community/wishes", &body, &token)
        .await;
    assert_eq!(status, StatusCode::CREATED);
}

#[tokio::test]
async fn close_wish_from_flagged_204() {
    let app = TestApp::new().await;
    let token = setup_aged_user(&app, "owner@test.com").await;
    let wish_id = create_open_wish(&app, &token).await;
    force_wish_status(&app, wish_id, "flagged").await;

    let (status, _) = app
        .post_with_auth(&format!("/community/wishes/{wish_id}/close"), &token)
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn admin_approve_not_found_404() {
    let app = TestApp::new().await;
    let token = setup_aged_user(&app, "admin@test.com").await;
    make_admin(&app, "admin@test.com").await;

    let fake_id = Uuid::new_v4();
    let (status, resp) = app
        .post_with_auth(&format!("/admin/wishes/{fake_id}/approve"), &token)
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_error(&resp, "NOT_FOUND");
}

#[tokio::test]
async fn admin_reject_not_found_404() {
    let app = TestApp::new().await;
    let token = setup_aged_user(&app, "admin@test.com").await;
    make_admin(&app, "admin@test.com").await;

    let fake_id = Uuid::new_v4();
    let (status, resp) = app
        .post_with_auth(&format!("/admin/wishes/{fake_id}/reject"), &token)
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_error(&resp, "NOT_FOUND");
}

#[tokio::test]
async fn admin_reject_review_wish_204() {
    let app = TestApp::new().await;
    let admin_token = setup_aged_user(&app, "admin@test.com").await;
    make_admin(&app, "admin@test.com").await;

    let owner_token = setup_aged_user(&app, "owner@test.com").await;
    let wish_id = create_open_wish(&app, &owner_token).await;
    force_wish_status(&app, wish_id, "review").await;

    let (status, _) = app
        .post_with_auth(&format!("/admin/wishes/{wish_id}/reject"), &admin_token)
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let row: (String,) = sqlx::query_as("SELECT status FROM community_wishes WHERE id = $1")
        .bind(wish_id)
        .fetch_one(&app.db)
        .await
        .unwrap();
    assert_eq!(row.0, "rejected");
}

#[tokio::test]
async fn get_wish_flagged_visible_to_owner_200() {
    let app = TestApp::new().await;
    let token = setup_aged_user(&app, "owner@test.com").await;
    let wish_id = create_open_wish(&app, &token).await;
    force_wish_status(&app, wish_id, "flagged").await;

    let (status, resp) = app
        .get_with_auth(&format!("/community/wishes/{wish_id}"), &token)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["status"].as_str(), Some("flagged"));
}

#[tokio::test]
async fn get_wish_flagged_not_visible_to_stranger_404() {
    let app = TestApp::new().await;
    let owner_token = setup_aged_user(&app, "owner@test.com").await;
    let wish_id = create_open_wish(&app, &owner_token).await;
    force_wish_status(&app, wish_id, "flagged").await;

    let stranger_token = setup_aged_user(&app, "stranger@test.com").await;
    let (status, _) = app
        .get_with_auth(&format!("/community/wishes/{wish_id}"), &stranger_token)
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn get_wish_rejected_visible_to_owner_200() {
    let app = TestApp::new().await;
    let token = setup_aged_user(&app, "owner@test.com").await;
    let wish_id = create_open_wish(&app, &token).await;
    force_wish_status(&app, wish_id, "rejected").await;

    let (status, resp) = app
        .get_with_auth(&format!("/community/wishes/{wish_id}"), &token)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["status"].as_str(), Some("rejected"));
}

#[tokio::test]
async fn get_wish_rejected_not_visible_to_stranger_404() {
    let app = TestApp::new().await;
    let owner_token = setup_aged_user(&app, "owner@test.com").await;
    let wish_id = create_open_wish(&app, &owner_token).await;
    force_wish_status(&app, wish_id, "rejected").await;

    let stranger_token = setup_aged_user(&app, "stranger@test.com").await;
    let (status, _) = app
        .get_with_auth(&format!("/community/wishes/{wish_id}"), &stranger_token)
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

// ── Notification center integration ──────────────────────────────────

#[tokio::test]
async fn wish_offer_creates_notification_for_owner() {
    let app = TestApp::new().await;
    let owner = setup_aged_user_with_name(&app, "notif_own@test.com", "Owner").await;
    let donor = setup_aged_user(&app, "notif_don@test.com").await;

    let wish_id = create_open_wish(&app, &owner).await;

    app.post_with_auth(&format!("/community/wishes/{wish_id}/offer"), &donor)
        .await;

    tokio::time::sleep(std::time::Duration::from_millis(300)).await;

    let (_, body) = app.get_with_auth("/me/notifications", &owner).await;
    let notifs = body["data"].as_array().unwrap();
    assert!(
        notifs
            .iter()
            .any(|n| n["type"].as_str().unwrap() == "wish_offer"),
        "owner should have a wish_offer notification"
    );
}

#[tokio::test]
async fn wish_confirm_creates_notification_for_donor() {
    let app = TestApp::new().await;
    let owner = setup_aged_user_with_name(&app, "nc_own@test.com", "Owner").await;
    let donor = setup_aged_user(&app, "nc_don@test.com").await;

    let wish_id = create_open_wish(&app, &owner).await;
    app.post_with_auth(&format!("/community/wishes/{wish_id}/offer"), &donor)
        .await;

    app.post_with_auth(&format!("/community/wishes/{wish_id}/confirm"), &owner)
        .await;

    tokio::time::sleep(std::time::Duration::from_millis(300)).await;

    let (_, body) = app.get_with_auth("/me/notifications", &donor).await;
    let notifs = body["data"].as_array().unwrap();
    assert!(
        notifs
            .iter()
            .any(|n| n["type"].as_str().unwrap() == "wish_confirmed"),
        "donor should have a wish_confirmed notification"
    );
}

#[tokio::test]
async fn wish_reject_creates_notification_for_donor() {
    let app = TestApp::new().await;
    let owner = setup_aged_user_with_name(&app, "nr_own@test.com", "Owner").await;
    let donor = setup_aged_user(&app, "nr_don@test.com").await;

    let wish_id = create_open_wish(&app, &owner).await;
    app.post_with_auth(&format!("/community/wishes/{wish_id}/offer"), &donor)
        .await;

    app.post_with_auth(&format!("/community/wishes/{wish_id}/reject"), &owner)
        .await;

    tokio::time::sleep(std::time::Duration::from_millis(300)).await;

    let (_, body) = app.get_with_auth("/me/notifications", &donor).await;
    let notifs = body["data"].as_array().unwrap();
    assert!(
        notifs
            .iter()
            .any(|n| n["type"].as_str().unwrap() == "wish_offer_rejected"),
        "donor should have a wish_offer_rejected notification"
    );
}

#[tokio::test]
async fn wish_close_when_matched_creates_notification_for_donor() {
    let app = TestApp::new().await;
    let owner = setup_aged_user_with_name(&app, "ncl_own@test.com", "Owner").await;
    let donor = setup_aged_user(&app, "ncl_don@test.com").await;

    let wish_id = create_open_wish(&app, &owner).await;
    app.post_with_auth(&format!("/community/wishes/{wish_id}/offer"), &donor)
        .await;

    app.post_with_auth(&format!("/community/wishes/{wish_id}/close"), &owner)
        .await;

    tokio::time::sleep(std::time::Duration::from_millis(300)).await;

    let (_, body) = app.get_with_auth("/me/notifications", &donor).await;
    let notifs = body["data"].as_array().unwrap();
    assert!(
        notifs
            .iter()
            .any(|n| n["type"].as_str().unwrap() == "wish_closed"),
        "donor should have a wish_closed notification"
    );
}

#[tokio::test]
async fn wish_moderation_creates_notification_for_owner() {
    let app = TestApp::new().await;
    let owner = setup_aged_user_with_name(&app, "nmod@test.com", "Owner").await;

    let body = json!({
        "title": "Need help",
        "category": "education",
        "is_anonymous": true,
    });
    let (status, resp) = app
        .post_json_with_auth("/community/wishes", &body, &owner)
        .await;
    assert_eq!(status, StatusCode::CREATED);
    let wish_id = Uuid::parse_str(resp["id"].as_str().unwrap()).unwrap();
    wait_for_wish_status(&app, wish_id, "open").await;

    tokio::time::sleep(std::time::Duration::from_millis(300)).await;

    let (_, notifs_body) = app.get_with_auth("/me/notifications", &owner).await;
    let notifs = notifs_body["data"].as_array().unwrap();
    assert!(
        notifs
            .iter()
            .any(|n| n["type"].as_str().unwrap().starts_with("wish_moderation")),
        "owner should have a moderation notification, got: {:?}",
        notifs
            .iter()
            .map(|n| n["type"].as_str())
            .collect::<Vec<_>>()
    );
}

// ── Re-moderation on update ──────────────────────────────────────────

#[tokio::test]
async fn update_wish_triggers_remoderation() {
    let app = TestApp::new().await;
    let token = setup_aged_user_with_name(&app, "remod@test.com", "Remod").await;

    let wish_id = create_open_wish(&app, &token).await;

    // Update content — should go to pending
    let (status, body) = app
        .patch_json_with_auth(
            &format!("/community/wishes/{wish_id}"),
            &json!({ "title": "Updated title" }),
            &token,
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        body["status"], "pending",
        "update should set status to pending"
    );

    // Wait for moderation to complete (NoopModerationService → approved → open)
    wait_for_wish_status(&app, wish_id, "open").await;
}

#[tokio::test]
async fn update_wish_from_review_triggers_remoderation() {
    let app = TestApp::new().await;
    let token = setup_aged_user_with_name(&app, "remod_rev@test.com", "Remod2").await;

    let wish_id = create_open_wish(&app, &token).await;
    force_wish_status(&app, wish_id, "review").await;

    let (status, body) = app
        .patch_json_with_auth(
            &format!("/community/wishes/{wish_id}"),
            &json!({ "title": "Fixed content" }),
            &token,
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "pending");

    wait_for_wish_status(&app, wish_id, "open").await;
}

// ── Re-moderation on reopen ──────────────────────────────────────────

#[tokio::test]
async fn reopen_wish_triggers_remoderation() {
    let app = TestApp::new().await;
    let token = setup_aged_user_with_name(&app, "reopen_mod@test.com", "Reopen").await;

    let wish_id = create_open_wish(&app, &token).await;
    force_wish_status(&app, wish_id, "review").await;

    let (status, _) = app
        .post_with_auth(&format!("/community/wishes/{wish_id}/reopen"), &token)
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Should go through pending → open via moderation (NoopModerationService approves instantly)
    wait_for_wish_status(&app, wish_id, "open").await;

    // Verify reports were cleared
    let count: (i32,) = sqlx::query_as("SELECT report_count FROM community_wishes WHERE id = $1")
        .bind(wish_id)
        .fetch_one(&app.db)
        .await
        .unwrap();
    assert_eq!(count.0, 0, "reports should be cleared after reopen");
}

#[tokio::test]
async fn reopen_still_respects_max_count() {
    let app = TestApp::new().await;
    let token = setup_aged_user_with_name(&app, "reopen_max@test.com", "Max").await;

    let wish_id = create_open_wish(&app, &token).await;

    // Simulate 2 previous reopens
    sqlx::query("UPDATE community_wishes SET status = 'review', reopen_count = 2 WHERE id = $1")
        .bind(wish_id)
        .execute(&app.db)
        .await
        .unwrap();

    let (status, body) = app
        .post_with_auth(&format!("/community/wishes/{wish_id}/reopen"), &token)
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert!(
        body["error"]["message"]
            .as_str()
            .unwrap()
            .contains("maximum")
    );
}

// ── Donor account deletion cleans up matched wishes ──────────────────

#[tokio::test]
async fn delete_donor_account_resets_matched_wish_to_open() {
    let app = TestApp::new().await;
    let owner = setup_aged_user_with_name(&app, "deldnr_own@test.com", "Owner").await;
    let donor = setup_aged_user(&app, "deldnr_don@test.com").await;

    let wish_id = create_open_wish(&app, &owner).await;

    // Donor offers
    let (status, _) = app
        .post_with_auth(&format!("/community/wishes/{wish_id}/offer"), &donor)
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Verify matched
    let row: (String,) = sqlx::query_as("SELECT status FROM community_wishes WHERE id = $1")
        .bind(wish_id)
        .fetch_one(&app.db)
        .await
        .unwrap();
    assert_eq!(row.0, "matched");

    // Delete donor account
    let (status, _) = app.delete_with_auth("/users/me", &donor).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Wish should be back to open
    let row: (String,) = sqlx::query_as("SELECT status FROM community_wishes WHERE id = $1")
        .bind(wish_id)
        .fetch_one(&app.db)
        .await
        .unwrap();
    assert_eq!(
        row.0, "open",
        "wish should reset to open when donor is deleted"
    );

    // matched_with should be NULL
    let matched: (Option<Uuid>,) =
        sqlx::query_as("SELECT matched_with FROM community_wishes WHERE id = $1")
            .bind(wish_id)
            .fetch_one(&app.db)
            .await
            .unwrap();
    assert!(matched.0.is_none(), "matched_with should be NULL");
}

#[tokio::test]
async fn delete_donor_does_not_affect_fulfilled_wishes() {
    let app = TestApp::new().await;
    let owner = setup_aged_user_with_name(&app, "delf_own@test.com", "Owner").await;
    let donor = setup_aged_user(&app, "delf_don@test.com").await;

    let wish_id = create_open_wish(&app, &owner).await;

    app.post_with_auth(&format!("/community/wishes/{wish_id}/offer"), &donor)
        .await;
    app.post_with_auth(&format!("/community/wishes/{wish_id}/confirm"), &owner)
        .await;

    // Verify fulfilled
    let row: (String,) = sqlx::query_as("SELECT status FROM community_wishes WHERE id = $1")
        .bind(wish_id)
        .fetch_one(&app.db)
        .await
        .unwrap();
    assert_eq!(row.0, "fulfilled");

    // Delete donor
    app.delete_with_auth("/users/me", &donor).await;

    // Wish should stay fulfilled (trigger only affects matched status)
    let row: (String,) = sqlx::query_as("SELECT status FROM community_wishes WHERE id = $1")
        .bind(wish_id)
        .fetch_one(&app.db)
        .await
        .unwrap();
    assert_eq!(
        row.0, "fulfilled",
        "fulfilled wishes should not be affected"
    );
}
