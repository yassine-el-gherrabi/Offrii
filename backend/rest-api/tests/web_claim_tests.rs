mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use common::{TEST_PASSWORD, TestApp};
use http_body_util::BodyExt;
use tower::ServiceExt;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Create a share link with the given permissions body and return the share token.
async fn create_share_link(app: &TestApp, auth_token: &str, body: &serde_json::Value) -> String {
    let (status, resp) = app
        .post_json_with_auth("/share-links", body, auth_token)
        .await;
    assert_eq!(
        status,
        StatusCode::CREATED,
        "precondition: share link creation failed: {resp}"
    );
    resp["token"]
        .as_str()
        .expect("share link token")
        .to_string()
}

/// Create a share link and return both the share token and the link id.
async fn create_share_link_with_id(
    app: &TestApp,
    auth_token: &str,
    body: &serde_json::Value,
) -> (String, String) {
    let (status, resp) = app
        .post_json_with_auth("/share-links", body, auth_token)
        .await;
    assert_eq!(
        status,
        StatusCode::CREATED,
        "precondition: share link creation failed: {resp}"
    );
    let token = resp["token"]
        .as_str()
        .expect("share link token")
        .to_string();
    let id = resp["id"].as_str().expect("share link id").to_string();
    (token, id)
}

/// Issue a DELETE request with a JSON body (no auth).
async fn delete_json_no_auth(
    app: &TestApp,
    uri: &str,
    body: &serde_json::Value,
) -> (StatusCode, serde_json::Value) {
    let req = Request::builder()
        .method("DELETE")
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_vec(body).unwrap()))
        .unwrap();
    let resp = app.router.clone().oneshot(req).await.unwrap();
    let status = resp.status();
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    if bytes.is_empty() {
        (status, serde_json::Value::Null)
    } else {
        (
            status,
            serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null),
        )
    }
}

// ---------------------------------------------------------------------------
// 1. web_claim_without_auth_201
// ---------------------------------------------------------------------------
#[tokio::test]
async fn web_claim_without_auth_201() {
    let app = TestApp::new().await;
    let token = app.setup_user_token("owner@test.com", TEST_PASSWORD).await;

    let item = app
        .create_item(&token, &serde_json::json!({"name": "PS5"}))
        .await;
    let item_id = item["id"].as_str().unwrap();

    let share_token = create_share_link(
        &app,
        &token,
        &serde_json::json!({"permissions": "view_and_claim"}),
    )
    .await;

    let (status, body) = app
        .post_json(
            &format!("/shared/{share_token}/items/{item_id}/web-claim"),
            &serde_json::json!({"name": "Marie"}),
        )
        .await;

    assert_eq!(status, StatusCode::CREATED);
    assert!(
        body["web_claim_token"].is_string(),
        "expected web_claim_token in response, got: {body}"
    );
}

// ---------------------------------------------------------------------------
// 2. web_claim_name_trimmed
// ---------------------------------------------------------------------------
#[tokio::test]
async fn web_claim_name_trimmed() {
    let app = TestApp::new().await;
    let token = app.setup_user_token("owner@test.com", TEST_PASSWORD).await;

    let item = app
        .create_item(&token, &serde_json::json!({"name": "Switch"}))
        .await;
    let item_id = item["id"].as_str().unwrap();

    let share_token = create_share_link(
        &app,
        &token,
        &serde_json::json!({"permissions": "view_and_claim"}),
    )
    .await;

    let (status, _) = app
        .post_json(
            &format!("/shared/{share_token}/items/{item_id}/web-claim"),
            &serde_json::json!({"name": " Marie "}),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED);

    // Verify the name was trimmed by checking the shared view
    let (_, view) = app
        .get_json_no_auth(&format!("/shared/{share_token}"))
        .await;
    let items = view["items"].as_array().unwrap();
    let claimed_item = items
        .iter()
        .find(|i| i["id"].as_str() == Some(item_id))
        .unwrap();
    assert_eq!(claimed_item["claimed_name"].as_str(), Some("Marie"));
}

// ---------------------------------------------------------------------------
// 3. web_claim_empty_name_400
// ---------------------------------------------------------------------------
#[tokio::test]
async fn web_claim_empty_name_400() {
    let app = TestApp::new().await;
    let token = app.setup_user_token("owner@test.com", TEST_PASSWORD).await;

    let item = app
        .create_item(&token, &serde_json::json!({"name": "Item"}))
        .await;
    let item_id = item["id"].as_str().unwrap();

    let share_token = create_share_link(
        &app,
        &token,
        &serde_json::json!({"permissions": "view_and_claim"}),
    )
    .await;

    let (status, body) = app
        .post_json(
            &format!("/shared/{share_token}/items/{item_id}/web-claim"),
            &serde_json::json!({"name": ""}),
        )
        .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    common::assert_error(&body, "BAD_REQUEST");
}

// ---------------------------------------------------------------------------
// 4. web_claim_name_too_long_400
// ---------------------------------------------------------------------------
#[tokio::test]
async fn web_claim_name_too_long_400() {
    let app = TestApp::new().await;
    let token = app.setup_user_token("owner@test.com", TEST_PASSWORD).await;

    let item = app
        .create_item(&token, &serde_json::json!({"name": "Item"}))
        .await;
    let item_id = item["id"].as_str().unwrap();

    let share_token = create_share_link(
        &app,
        &token,
        &serde_json::json!({"permissions": "view_and_claim"}),
    )
    .await;

    let long_name = "A".repeat(101);
    let (status, body) = app
        .post_json(
            &format!("/shared/{share_token}/items/{item_id}/web-claim"),
            &serde_json::json!({"name": long_name}),
        )
        .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    common::assert_error(&body, "BAD_REQUEST");
}

// ---------------------------------------------------------------------------
// 5. web_claim_item_already_claimed_via_app_409
// ---------------------------------------------------------------------------
#[tokio::test]
async fn web_claim_item_already_claimed_via_app_409() {
    let app = TestApp::new().await;
    let owner_token = app.setup_user_token("owner@test.com", TEST_PASSWORD).await;
    let token2 = app.setup_user_token("other@test.com", TEST_PASSWORD).await;

    let item = app
        .create_item(&owner_token, &serde_json::json!({"name": "PS5"}))
        .await;
    let item_id = item["id"].as_str().unwrap();

    let share_token = create_share_link(
        &app,
        &owner_token,
        &serde_json::json!({"permissions": "view_and_claim"}),
    )
    .await;

    // Claim via app (authenticated user)
    let (status, _) = app
        .post_with_auth(
            &format!("/shared/{share_token}/items/{item_id}/claim"),
            &token2,
        )
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Try web claim on the already-claimed item
    let (status, body) = app
        .post_json(
            &format!("/shared/{share_token}/items/{item_id}/web-claim"),
            &serde_json::json!({"name": "Marie"}),
        )
        .await;

    assert_eq!(status, StatusCode::CONFLICT);
    common::assert_error(&body, "CONFLICT");
}

// ---------------------------------------------------------------------------
// 6. web_claim_item_already_web_claimed_409
// ---------------------------------------------------------------------------
#[tokio::test]
async fn web_claim_item_already_web_claimed_409() {
    let app = TestApp::new().await;
    let token = app.setup_user_token("owner@test.com", TEST_PASSWORD).await;

    let item = app
        .create_item(&token, &serde_json::json!({"name": "Switch"}))
        .await;
    let item_id = item["id"].as_str().unwrap();

    let share_token = create_share_link(
        &app,
        &token,
        &serde_json::json!({"permissions": "view_and_claim"}),
    )
    .await;

    // First web claim succeeds
    let (status, _) = app
        .post_json(
            &format!("/shared/{share_token}/items/{item_id}/web-claim"),
            &serde_json::json!({"name": "Marie"}),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED);

    // Second web claim should fail with 409
    let (status, body) = app
        .post_json(
            &format!("/shared/{share_token}/items/{item_id}/web-claim"),
            &serde_json::json!({"name": "Paul"}),
        )
        .await;

    assert_eq!(status, StatusCode::CONFLICT);
    common::assert_error(&body, "CONFLICT");
}

// ---------------------------------------------------------------------------
// 7. web_claim_view_only_403
// ---------------------------------------------------------------------------
#[tokio::test]
async fn web_claim_view_only_403() {
    let app = TestApp::new().await;
    let token = app.setup_user_token("owner@test.com", TEST_PASSWORD).await;

    let item = app
        .create_item(&token, &serde_json::json!({"name": "Item"}))
        .await;
    let item_id = item["id"].as_str().unwrap();

    let share_token = create_share_link(
        &app,
        &token,
        &serde_json::json!({"permissions": "view_only"}),
    )
    .await;

    let (status, body) = app
        .post_json(
            &format!("/shared/{share_token}/items/{item_id}/web-claim"),
            &serde_json::json!({"name": "Marie"}),
        )
        .await;

    assert_eq!(status, StatusCode::FORBIDDEN);
    common::assert_error(&body, "FORBIDDEN");
}

// ---------------------------------------------------------------------------
// 8. web_claim_expired_link_410
// ---------------------------------------------------------------------------
#[tokio::test]
async fn web_claim_expired_link_410() {
    let app = TestApp::new().await;
    let token = app.setup_user_token("owner@test.com", TEST_PASSWORD).await;

    let item = app
        .create_item(&token, &serde_json::json!({"name": "Item"}))
        .await;
    let item_id = item["id"].as_str().unwrap();

    let share_token = create_share_link(
        &app,
        &token,
        &serde_json::json!({
            "permissions": "view_and_claim",
            "expires_at": "2020-01-01T00:00:00Z"
        }),
    )
    .await;

    let (status, body) = app
        .post_json(
            &format!("/shared/{share_token}/items/{item_id}/web-claim"),
            &serde_json::json!({"name": "Marie"}),
        )
        .await;

    assert_eq!(status, StatusCode::GONE);
    common::assert_error(&body, "GONE");
}

// ---------------------------------------------------------------------------
// 9. web_claim_disabled_link_410
// ---------------------------------------------------------------------------
#[tokio::test]
async fn web_claim_disabled_link_410() {
    let app = TestApp::new().await;
    let token = app.setup_user_token("owner@test.com", TEST_PASSWORD).await;

    let item = app
        .create_item(&token, &serde_json::json!({"name": "Item"}))
        .await;
    let item_id = item["id"].as_str().unwrap();

    let (share_token, link_id) = create_share_link_with_id(
        &app,
        &token,
        &serde_json::json!({"permissions": "view_and_claim"}),
    )
    .await;

    // Disable the link
    app.patch_json_with_auth(
        &format!("/share-links/{link_id}"),
        &serde_json::json!({"is_active": false}),
        &token,
    )
    .await;

    let (status, body) = app
        .post_json(
            &format!("/shared/{share_token}/items/{item_id}/web-claim"),
            &serde_json::json!({"name": "Marie"}),
        )
        .await;

    assert_eq!(status, StatusCode::GONE);
    common::assert_error(&body, "GONE");
}

// ---------------------------------------------------------------------------
// 10. web_claim_nonexistent_item_404
// ---------------------------------------------------------------------------
#[tokio::test]
async fn web_claim_nonexistent_item_404() {
    let app = TestApp::new().await;
    let token = app.setup_user_token("owner@test.com", TEST_PASSWORD).await;

    let share_token = create_share_link(
        &app,
        &token,
        &serde_json::json!({"permissions": "view_and_claim"}),
    )
    .await;

    let fake_item_id = uuid::Uuid::new_v4();
    let (status, body) = app
        .post_json(
            &format!("/shared/{share_token}/items/{fake_item_id}/web-claim"),
            &serde_json::json!({"name": "Marie"}),
        )
        .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    common::assert_error(&body, "NOT_FOUND");
}

// ---------------------------------------------------------------------------
// 11. web_claim_nonexistent_link_404
// ---------------------------------------------------------------------------
#[tokio::test]
async fn web_claim_nonexistent_link_404() {
    let app = TestApp::new().await;

    let fake_item_id = uuid::Uuid::new_v4();
    let (status, body) = app
        .post_json(
            &format!("/shared/nonexistent_token_xyz/items/{fake_item_id}/web-claim"),
            &serde_json::json!({"name": "Marie"}),
        )
        .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    common::assert_error(&body, "NOT_FOUND");
}

// ---------------------------------------------------------------------------
// 12. web_unclaim_with_valid_token_204
// ---------------------------------------------------------------------------
#[tokio::test]
async fn web_unclaim_with_valid_token_204() {
    let app = TestApp::new().await;
    let token = app.setup_user_token("owner@test.com", TEST_PASSWORD).await;

    let item = app
        .create_item(&token, &serde_json::json!({"name": "PS5"}))
        .await;
    let item_id = item["id"].as_str().unwrap();

    let share_token = create_share_link(
        &app,
        &token,
        &serde_json::json!({"permissions": "view_and_claim"}),
    )
    .await;

    // Web claim
    let (status, claim_resp) = app
        .post_json(
            &format!("/shared/{share_token}/items/{item_id}/web-claim"),
            &serde_json::json!({"name": "Marie"}),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED);
    let web_claim_token = claim_resp["web_claim_token"].as_str().unwrap();

    // Web unclaim
    let (status, _) = delete_json_no_auth(
        &app,
        &format!("/shared/{share_token}/items/{item_id}/web-claim"),
        &serde_json::json!({"web_claim_token": web_claim_token}),
    )
    .await;

    assert_eq!(status, StatusCode::NO_CONTENT);

    // Verify item is no longer claimed
    let (_, view) = app
        .get_json_no_auth(&format!("/shared/{share_token}"))
        .await;
    let items = view["items"].as_array().unwrap();
    let item_view = items
        .iter()
        .find(|i| i["id"].as_str() == Some(item_id))
        .unwrap();
    assert_eq!(item_view["is_claimed"].as_bool(), Some(false));
}

// ---------------------------------------------------------------------------
// 13. web_unclaim_wrong_token_404
// ---------------------------------------------------------------------------
#[tokio::test]
async fn web_unclaim_wrong_token_404() {
    let app = TestApp::new().await;
    let token = app.setup_user_token("owner@test.com", TEST_PASSWORD).await;

    let item = app
        .create_item(&token, &serde_json::json!({"name": "PS5"}))
        .await;
    let item_id = item["id"].as_str().unwrap();

    let share_token = create_share_link(
        &app,
        &token,
        &serde_json::json!({"permissions": "view_and_claim"}),
    )
    .await;

    // Web claim
    let (status, _) = app
        .post_json(
            &format!("/shared/{share_token}/items/{item_id}/web-claim"),
            &serde_json::json!({"name": "Marie"}),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED);

    // Try unclaim with wrong token
    let wrong_token = uuid::Uuid::new_v4();
    let (status, body) = delete_json_no_auth(
        &app,
        &format!("/shared/{share_token}/items/{item_id}/web-claim"),
        &serde_json::json!({"web_claim_token": wrong_token}),
    )
    .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    common::assert_error(&body, "NOT_FOUND");
}

// ---------------------------------------------------------------------------
// 14. web_unclaim_already_unclaimed_404
// ---------------------------------------------------------------------------
#[tokio::test]
async fn web_unclaim_already_unclaimed_404() {
    let app = TestApp::new().await;
    let token = app.setup_user_token("owner@test.com", TEST_PASSWORD).await;

    let item = app
        .create_item(&token, &serde_json::json!({"name": "PS5"}))
        .await;
    let item_id = item["id"].as_str().unwrap();

    let share_token = create_share_link(
        &app,
        &token,
        &serde_json::json!({"permissions": "view_and_claim"}),
    )
    .await;

    // Web claim
    let (_, claim_resp) = app
        .post_json(
            &format!("/shared/{share_token}/items/{item_id}/web-claim"),
            &serde_json::json!({"name": "Marie"}),
        )
        .await;
    let web_claim_token = claim_resp["web_claim_token"].as_str().unwrap();

    // First unclaim succeeds
    let (status, _) = delete_json_no_auth(
        &app,
        &format!("/shared/{share_token}/items/{item_id}/web-claim"),
        &serde_json::json!({"web_claim_token": web_claim_token}),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Second unclaim should fail with 404
    let (status, body) = delete_json_no_auth(
        &app,
        &format!("/shared/{share_token}/items/{item_id}/web-claim"),
        &serde_json::json!({"web_claim_token": web_claim_token}),
    )
    .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    common::assert_error(&body, "NOT_FOUND");
}

// ---------------------------------------------------------------------------
// 15. owner_unclaim_web_claim_204
// ---------------------------------------------------------------------------
#[tokio::test]
async fn owner_unclaim_web_claim_204() {
    let app = TestApp::new().await;
    let owner_token = app.setup_user_token("owner@test.com", TEST_PASSWORD).await;

    let item = app
        .create_item(&owner_token, &serde_json::json!({"name": "PS5"}))
        .await;
    let item_id = item["id"].as_str().unwrap();

    let share_token = create_share_link(
        &app,
        &owner_token,
        &serde_json::json!({"permissions": "view_and_claim"}),
    )
    .await;

    // Web claim the item
    let (status, _) = app
        .post_json(
            &format!("/shared/{share_token}/items/{item_id}/web-claim"),
            &serde_json::json!({"name": "Marie"}),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED);

    // Owner unclaims via authenticated endpoint
    let (status, _) = app
        .delete_with_auth(&format!("/items/{item_id}/web-claim"), &owner_token)
        .await;

    assert_eq!(status, StatusCode::NO_CONTENT);

    // Verify item is no longer claimed
    let (_, view) = app
        .get_json_no_auth(&format!("/shared/{share_token}"))
        .await;
    let items = view["items"].as_array().unwrap();
    let item_view = items
        .iter()
        .find(|i| i["id"].as_str() == Some(item_id))
        .unwrap();
    assert_eq!(item_view["is_claimed"].as_bool(), Some(false));
}

// ---------------------------------------------------------------------------
// 16. owner_cannot_unclaim_app_claim_403
// ---------------------------------------------------------------------------
#[tokio::test]
async fn owner_cannot_unclaim_app_claim_403() {
    let app = TestApp::new().await;
    let owner_token = app.setup_user_token("owner@test.com", TEST_PASSWORD).await;
    let token2 = app.setup_user_token("other@test.com", TEST_PASSWORD).await;

    let item = app
        .create_item(&owner_token, &serde_json::json!({"name": "PS5"}))
        .await;
    let item_id = item["id"].as_str().unwrap();

    let share_token = create_share_link(
        &app,
        &owner_token,
        &serde_json::json!({"permissions": "view_and_claim"}),
    )
    .await;

    // Other user claims via app
    let (status, _) = app
        .post_with_auth(
            &format!("/shared/{share_token}/items/{item_id}/claim"),
            &token2,
        )
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Owner tries to unclaim via the web-claim owner endpoint
    let (status, body) = app
        .delete_with_auth(&format!("/items/{item_id}/web-claim"), &owner_token)
        .await;

    assert_eq!(status, StatusCode::FORBIDDEN);
    common::assert_error(&body, "FORBIDDEN");
}

// ---------------------------------------------------------------------------
// 17. owner_unclaim_not_claimed_409
// ---------------------------------------------------------------------------
#[tokio::test]
async fn owner_unclaim_not_claimed_409() {
    let app = TestApp::new().await;
    let owner_token = app.setup_user_token("owner@test.com", TEST_PASSWORD).await;

    let item = app
        .create_item(&owner_token, &serde_json::json!({"name": "PS5"}))
        .await;
    let item_id = item["id"].as_str().unwrap();

    // Owner tries to unclaim an item that is not claimed at all
    let (status, body) = app
        .delete_with_auth(&format!("/items/{item_id}/web-claim"), &owner_token)
        .await;

    assert_eq!(status, StatusCode::CONFLICT);
    common::assert_error(&body, "CONFLICT");
}

// ---------------------------------------------------------------------------
// 18. owner_unclaim_without_auth_401
// ---------------------------------------------------------------------------
#[tokio::test]
async fn owner_unclaim_without_auth_401() {
    let app = TestApp::new().await;
    let owner_token = app.setup_user_token("owner@test.com", TEST_PASSWORD).await;

    let item = app
        .create_item(&owner_token, &serde_json::json!({"name": "PS5"}))
        .await;
    let item_id = item["id"].as_str().unwrap();

    // Try to hit the authenticated endpoint without a token
    let req = Request::builder()
        .method("DELETE")
        .uri(format!("/items/{item_id}/web-claim"))
        .body(Body::empty())
        .unwrap();
    let resp = app.router.clone().oneshot(req).await.unwrap();

    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

// ---------------------------------------------------------------------------
// 19. app_claim_sets_claimed_via_app
// ---------------------------------------------------------------------------
#[tokio::test]
async fn app_claim_sets_claimed_via_app() {
    let app = TestApp::new().await;
    let owner_token = app.setup_user_token("owner@test.com", TEST_PASSWORD).await;
    let token2 = app.setup_user_token("other@test.com", TEST_PASSWORD).await;

    let item = app
        .create_item(&owner_token, &serde_json::json!({"name": "PS5"}))
        .await;
    let item_id = item["id"].as_str().unwrap();

    let share_token = create_share_link(
        &app,
        &owner_token,
        &serde_json::json!({"permissions": "view_and_claim"}),
    )
    .await;

    // Claim via app
    let (status, _) = app
        .post_with_auth(
            &format!("/shared/{share_token}/items/{item_id}/claim"),
            &token2,
        )
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Verify claimed_via is "app" by querying the DB directly
    let row: (String,) = sqlx::query_as("SELECT claimed_via FROM items WHERE id = $1::uuid")
        .bind(item_id)
        .fetch_one(&app.db)
        .await
        .expect("item should exist in DB");

    assert_eq!(row.0, "app");
}

// ---------------------------------------------------------------------------
// 20. app_claim_blocked_by_web_claim_409
// ---------------------------------------------------------------------------
#[tokio::test]
async fn app_claim_blocked_by_web_claim_409() {
    let app = TestApp::new().await;
    let owner_token = app.setup_user_token("owner@test.com", TEST_PASSWORD).await;
    let token2 = app.setup_user_token("other@test.com", TEST_PASSWORD).await;

    let item = app
        .create_item(&owner_token, &serde_json::json!({"name": "PS5"}))
        .await;
    let item_id = item["id"].as_str().unwrap();

    let share_token = create_share_link(
        &app,
        &owner_token,
        &serde_json::json!({"permissions": "view_and_claim"}),
    )
    .await;

    // Web claim first
    let (status, _) = app
        .post_json(
            &format!("/shared/{share_token}/items/{item_id}/web-claim"),
            &serde_json::json!({"name": "Marie"}),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED);

    // App claim should fail with 409
    let (status, body) = app
        .post_with_auth(
            &format!("/shared/{share_token}/items/{item_id}/claim"),
            &token2,
        )
        .await;

    assert_eq!(status, StatusCode::CONFLICT);
    common::assert_error(&body, "CONFLICT");
}
