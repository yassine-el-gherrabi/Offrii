mod common;

use axum::body::Body;
use axum::http::{StatusCode, header};
use http_body_util::BodyExt;
use tower::ServiceExt;

use common::{TEST_PASSWORD, TestApp};

#[tokio::test]
async fn create_share_link_returns_201() {
    let app = TestApp::new().await;
    let token = app.setup_user_token("alice@test.com", TEST_PASSWORD).await;

    let (status, body) = app.post_with_auth("/share-links", &token).await;

    assert_eq!(status, StatusCode::CREATED);
    assert!(body["token"].is_string());
    assert!(body["url"].is_string());
    assert!(body["id"].is_string());
    let url = body["url"].as_str().unwrap();
    assert!(url.contains("/shared/"));
}

#[tokio::test]
async fn list_share_links_returns_created_links() {
    let app = TestApp::new().await;
    let token = app.setup_user_token("alice@test.com", TEST_PASSWORD).await;

    // Create two links
    app.post_with_auth("/share-links", &token).await;
    app.post_with_auth("/share-links", &token).await;

    let (status, body) = app.get_with_auth("/share-links", &token).await;

    assert_eq!(status, StatusCode::OK);
    let links = body.as_array().unwrap();
    assert_eq!(links.len(), 2);
    assert!(links[0]["token"].is_string());
    assert!(links[0]["id"].is_string());
}

#[tokio::test]
async fn delete_share_link_returns_204() {
    let app = TestApp::new().await;
    let token = app.setup_user_token("alice@test.com", TEST_PASSWORD).await;

    let (_, body) = app.post_with_auth("/share-links", &token).await;
    let link_id = body["id"].as_str().unwrap();

    let (status, _) = app
        .delete_with_auth(&format!("/share-links/{link_id}"), &token)
        .await;

    assert_eq!(status, StatusCode::NO_CONTENT);

    // Verify it's gone
    let (_, body) = app.get_with_auth("/share-links", &token).await;
    assert_eq!(body.as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn delete_nonexistent_share_link_returns_404() {
    let app = TestApp::new().await;
    let token = app.setup_user_token("alice@test.com", TEST_PASSWORD).await;

    let fake_id = uuid::Uuid::new_v4();
    let (status, body) = app
        .delete_with_auth(&format!("/share-links/{fake_id}"), &token)
        .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    common::assert_error(&body, "NOT_FOUND");
}

#[tokio::test]
async fn cannot_delete_another_users_share_link() {
    let app = TestApp::new().await;
    let alice_token = app.setup_user_token("alice@test.com", TEST_PASSWORD).await;
    let bob_token = app.setup_user_token("bob@test.com", TEST_PASSWORD).await;

    let (_, body) = app.post_with_auth("/share-links", &alice_token).await;
    let link_id = body["id"].as_str().unwrap();

    let (status, _) = app
        .delete_with_auth(&format!("/share-links/{link_id}"), &bob_token)
        .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn get_shared_view_returns_items() {
    let app = TestApp::new().await;
    let token = app.setup_user_token("alice@test.com", TEST_PASSWORD).await;

    // Create an item
    let item_body = serde_json::json!({ "name": "Nintendo Switch" });
    app.create_item(&token, &item_body).await;

    // Create share link
    let (_, link_body) = app.post_with_auth("/share-links", &token).await;
    let share_token = link_body["token"].as_str().unwrap();

    // Access shared view (public endpoint, no auth needed)
    let (status, body) = app
        .get_json_no_auth(&format!("/shared/{share_token}"))
        .await;

    assert_eq!(status, StatusCode::OK);
    assert!(body["items"].is_array());
    let items = body["items"].as_array().unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["name"], "Nintendo Switch");
}

#[tokio::test]
async fn get_shared_view_invalid_token_returns_404() {
    let app = TestApp::new().await;

    let (status, body) = app.get_json_no_auth("/shared/nonexistent_token").await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    common::assert_error(&body, "NOT_FOUND");
}

#[tokio::test]
async fn claim_via_share_link() {
    let app = TestApp::new().await;
    let alice_token = app.setup_user_token("alice@test.com", TEST_PASSWORD).await;
    let bob_token = app.setup_user_token("bob@test.com", TEST_PASSWORD).await;

    // Alice creates an item
    let item_body = serde_json::json!({ "name": "PS5" });
    let item = app.create_item(&alice_token, &item_body).await;
    let item_id = item["id"].as_str().unwrap();

    // Alice creates a share link
    let (_, link_body) = app.post_with_auth("/share-links", &alice_token).await;
    let share_token = link_body["token"].as_str().unwrap();

    // Bob claims the item via share link
    let (status, _) = app
        .post_with_auth(
            &format!("/shared/{share_token}/items/{item_id}/claim"),
            &bob_token,
        )
        .await;

    assert_eq!(status, StatusCode::NO_CONTENT);

    // Verify item is claimed in the shared view
    let (_, body) = app
        .get_json_no_auth(&format!("/shared/{share_token}"))
        .await;
    let items = body["items"].as_array().unwrap();
    assert!(items[0]["is_claimed"].as_bool().unwrap());
}

#[tokio::test]
async fn unclaim_via_share_link() {
    let app = TestApp::new().await;
    let alice_token = app.setup_user_token("alice@test.com", TEST_PASSWORD).await;
    let bob_token = app.setup_user_token("bob@test.com", TEST_PASSWORD).await;

    // Alice creates an item
    let item_body = serde_json::json!({ "name": "PS5" });
    let item = app.create_item(&alice_token, &item_body).await;
    let item_id = item["id"].as_str().unwrap();

    // Alice creates a share link
    let (_, link_body) = app.post_with_auth("/share-links", &alice_token).await;
    let share_token = link_body["token"].as_str().unwrap();

    // Bob claims then unclaims
    app.post_with_auth(
        &format!("/shared/{share_token}/items/{item_id}/claim"),
        &bob_token,
    )
    .await;

    let (status, _) = app
        .delete_with_auth(
            &format!("/shared/{share_token}/items/{item_id}/claim"),
            &bob_token,
        )
        .await;

    assert_eq!(status, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn cannot_claim_own_item_via_share() {
    let app = TestApp::new().await;
    let alice_token = app.setup_user_token("alice@test.com", TEST_PASSWORD).await;

    let item_body = serde_json::json!({ "name": "PS5" });
    let item = app.create_item(&alice_token, &item_body).await;
    let item_id = item["id"].as_str().unwrap();

    let (_, link_body) = app.post_with_auth("/share-links", &alice_token).await;
    let share_token = link_body["token"].as_str().unwrap();

    let (status, body) = app
        .post_with_auth(
            &format!("/shared/{share_token}/items/{item_id}/claim"),
            &alice_token,
        )
        .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    common::assert_error(&body, "BAD_REQUEST");
}

#[tokio::test]
async fn share_link_requires_auth_for_protected_endpoints() {
    let app = TestApp::new().await;

    // POST /share-links without auth
    let (status, _) = app.post_empty("/share-links").await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    // GET /share-links without auth
    let (status, _) = app.get_no_auth("/share-links").await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn shared_view_returns_user_username() {
    let app = TestApp::new().await;
    let token = app.setup_user_token("alice@test.com", TEST_PASSWORD).await;

    // Create share link
    let (_, link_body) = app.post_with_auth("/share-links", &token).await;
    let share_token = link_body["token"].as_str().unwrap();

    let (status, body) = app
        .get_json_no_auth(&format!("/shared/{share_token}"))
        .await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        body["user_username"].is_string(),
        "expected user_username string, got {body}"
    );
}

#[tokio::test]
async fn expired_link_returns_410() {
    let app = TestApp::new().await;
    let token = app.setup_user_token("alice@test.com", TEST_PASSWORD).await;

    // Create a share link via API
    let (_, link_body) = app.post_with_auth("/share-links", &token).await;
    let share_token = link_body["token"].as_str().unwrap();

    // Manually set expires_at to the past via direct SQL
    sqlx::query("UPDATE share_links SET expires_at = NOW() - INTERVAL '1 hour' WHERE token = $1")
        .bind(share_token)
        .execute(&app.db)
        .await
        .unwrap();

    let (status, body) = app
        .get_json_no_auth(&format!("/shared/{share_token}"))
        .await;
    assert_eq!(status, StatusCode::GONE);
    common::assert_error(&body, "GONE");
}

#[tokio::test]
async fn claim_expired_link_returns_410() {
    let app = TestApp::new().await;
    let alice_token = app.setup_user_token("alice@test.com", TEST_PASSWORD).await;
    let bob_token = app.setup_user_token("bob@test.com", TEST_PASSWORD).await;

    // Alice creates item + share link
    let item = app
        .create_item(&alice_token, &serde_json::json!({ "name": "PS5" }))
        .await;
    let item_id = item["id"].as_str().unwrap();
    let (_, link_body) = app.post_with_auth("/share-links", &alice_token).await;
    let share_token = link_body["token"].as_str().unwrap();

    // Expire the link
    sqlx::query("UPDATE share_links SET expires_at = NOW() - INTERVAL '1 hour' WHERE token = $1")
        .bind(share_token)
        .execute(&app.db)
        .await
        .unwrap();

    // Bob tries to claim
    let (status, body) = app
        .post_with_auth(
            &format!("/shared/{share_token}/items/{item_id}/claim"),
            &bob_token,
        )
        .await;

    assert_eq!(status, StatusCode::GONE);
    common::assert_error(&body, "GONE");
}

#[tokio::test]
async fn double_claim_returns_409() {
    let app = TestApp::new().await;
    let alice_token = app.setup_user_token("alice@test.com", TEST_PASSWORD).await;
    let bob_token = app.setup_user_token("bob@test.com", TEST_PASSWORD).await;
    let carol_token = app.setup_user_token("carol@test.com", TEST_PASSWORD).await;

    // Alice creates item + share link
    let item = app
        .create_item(&alice_token, &serde_json::json!({ "name": "PS5" }))
        .await;
    let item_id = item["id"].as_str().unwrap();
    let (_, link_body) = app.post_with_auth("/share-links", &alice_token).await;
    let share_token = link_body["token"].as_str().unwrap();

    // Bob claims first
    let (status, _) = app
        .post_with_auth(
            &format!("/shared/{share_token}/items/{item_id}/claim"),
            &bob_token,
        )
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Carol tries to claim the same item
    let (status, body) = app
        .post_with_auth(
            &format!("/shared/{share_token}/items/{item_id}/claim"),
            &carol_token,
        )
        .await;
    assert_eq!(status, StatusCode::CONFLICT);
    common::assert_error(&body, "CONFLICT");
}

#[tokio::test]
async fn shared_view_does_not_show_deleted_items() {
    let app = TestApp::new().await;
    let token = app.setup_user_token("alice@test.com", TEST_PASSWORD).await;

    // Create two items
    let item1 = app
        .create_item(&token, &serde_json::json!({ "name": "Item 1" }))
        .await;
    app.create_item(&token, &serde_json::json!({ "name": "Item 2" }))
        .await;

    // Delete first item
    let item1_id = item1["id"].as_str().unwrap();
    app.delete_with_auth(&format!("/items/{item1_id}"), &token)
        .await;

    // Create share link and view
    let (_, link_body) = app.post_with_auth("/share-links", &token).await;
    let share_token = link_body["token"].as_str().unwrap();

    let (status, body) = app
        .get_json_no_auth(&format!("/shared/{share_token}"))
        .await;

    assert_eq!(status, StatusCode::OK);
    let items = body["items"].as_array().unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["name"], "Item 2");
}

// ── Scope tests ──────────────────────────────────────────────────────

#[tokio::test]
async fn create_share_link_with_scope_category() {
    let app = TestApp::new().await;
    let token = app.setup_user_token("alice@test.com", TEST_PASSWORD).await;

    // Use a global category (Loisirs)
    let (_, cats) = app.get_with_auth("/categories", &token).await;
    let cat_id = cats
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"].as_str().unwrap() == "Loisirs")
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();

    app.create_item(
        &token,
        &serde_json::json!({ "name": "PS5", "category_id": cat_id }),
    )
    .await;
    app.create_item(
        &token,
        &serde_json::json!({ "name": "Book" }), // no category
    )
    .await;

    // Create share link with scope=category
    let body = serde_json::json!({
        "scope": "category",
        "scope_data": { "category_id": cat_id }
    });
    let (status, link_body) = app.post_json_with_auth("/share-links", &body, &token).await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(link_body["scope"], "category");

    let share_token = link_body["token"].as_str().unwrap();

    // Access shared view — should only show PS5
    let (status, view) = app
        .get_json_no_auth(&format!("/shared/{share_token}"))
        .await;
    assert_eq!(status, StatusCode::OK);
    let items = view["items"].as_array().unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["name"], "PS5");
}

#[tokio::test]
async fn create_share_link_with_scope_selection() {
    let app = TestApp::new().await;
    let token = app.setup_user_token("alice@test.com", TEST_PASSWORD).await;

    let item1 = app
        .create_item(&token, &serde_json::json!({ "name": "PS5" }))
        .await;
    let item2 = app
        .create_item(&token, &serde_json::json!({ "name": "Xbox" }))
        .await;
    app.create_item(&token, &serde_json::json!({ "name": "Switch" }))
        .await;

    let item1_id = item1["id"].as_str().unwrap();
    let item2_id = item2["id"].as_str().unwrap();

    let body = serde_json::json!({
        "scope": "selection",
        "scope_data": { "item_ids": [item1_id, item2_id] }
    });
    let (status, link_body) = app.post_json_with_auth("/share-links", &body, &token).await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(link_body["scope"], "selection");

    let share_token = link_body["token"].as_str().unwrap();

    let (status, view) = app
        .get_json_no_auth(&format!("/shared/{share_token}"))
        .await;
    assert_eq!(status, StatusCode::OK);
    let items = view["items"].as_array().unwrap();
    assert_eq!(items.len(), 2);
    let names: Vec<&str> = items.iter().map(|i| i["name"].as_str().unwrap()).collect();
    assert!(names.contains(&"PS5"));
    assert!(names.contains(&"Xbox"));
}

#[tokio::test]
async fn create_share_link_scope_all_with_scope_data_returns_400() {
    let app = TestApp::new().await;
    let token = app.setup_user_token("alice@test.com", TEST_PASSWORD).await;

    let body = serde_json::json!({
        "scope": "all",
        "scope_data": { "some": "data" }
    });
    let (status, body) = app.post_json_with_auth("/share-links", &body, &token).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    common::assert_error(&body, "BAD_REQUEST");
}

#[tokio::test]
async fn create_share_link_scope_selection_invalid_items_returns_400() {
    let app = TestApp::new().await;
    let alice_token = app.setup_user_token("alice@test.com", TEST_PASSWORD).await;
    let bob_token = app.setup_user_token("bob@test.com", TEST_PASSWORD).await;

    // Bob creates an item
    let bob_item = app
        .create_item(&bob_token, &serde_json::json!({ "name": "Bob's item" }))
        .await;
    let bob_item_id = bob_item["id"].as_str().unwrap();

    // Alice tries to create a link with Bob's item
    let body = serde_json::json!({
        "scope": "selection",
        "scope_data": { "item_ids": [bob_item_id] }
    });
    let (status, resp) = app
        .post_json_with_auth("/share-links", &body, &alice_token)
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    common::assert_error(&resp, "BAD_REQUEST");
}

// ── Permissions tests ────────────────────────────────────────────────

#[tokio::test]
async fn claim_view_only_link_returns_403() {
    let app = TestApp::new().await;
    let alice_token = app.setup_user_token("alice@test.com", TEST_PASSWORD).await;
    let bob_token = app.setup_user_token("bob@test.com", TEST_PASSWORD).await;

    let item = app
        .create_item(&alice_token, &serde_json::json!({ "name": "PS5" }))
        .await;
    let item_id = item["id"].as_str().unwrap();

    let body = serde_json::json!({ "permissions": "view_only" });
    let (_, link_body) = app
        .post_json_with_auth("/share-links", &body, &alice_token)
        .await;
    let share_token = link_body["token"].as_str().unwrap();

    let (status, resp) = app
        .post_with_auth(
            &format!("/shared/{share_token}/items/{item_id}/claim"),
            &bob_token,
        )
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    common::assert_error(&resp, "FORBIDDEN");
}

#[tokio::test]
async fn view_only_link_shows_items() {
    let app = TestApp::new().await;
    let token = app.setup_user_token("alice@test.com", TEST_PASSWORD).await;

    app.create_item(&token, &serde_json::json!({ "name": "PS5" }))
        .await;

    let body = serde_json::json!({ "permissions": "view_only" });
    let (_, link_body) = app.post_json_with_auth("/share-links", &body, &token).await;
    let share_token = link_body["token"].as_str().unwrap();

    let (status, view) = app
        .get_json_no_auth(&format!("/shared/{share_token}"))
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(view["permissions"], "view_only");
    let items = view["items"].as_array().unwrap();
    assert_eq!(items.len(), 1);
}

// ── is_active tests ──────────────────────────────────────────────────

#[tokio::test]
async fn deactivated_link_returns_410() {
    let app = TestApp::new().await;
    let token = app.setup_user_token("alice@test.com", TEST_PASSWORD).await;

    let (_, link_body) = app.post_with_auth("/share-links", &token).await;
    let link_id = link_body["id"].as_str().unwrap();
    let share_token = link_body["token"].as_str().unwrap();

    // Deactivate
    let patch_body = serde_json::json!({ "is_active": false });
    let (status, _) = app
        .patch_json_with_auth(&format!("/share-links/{link_id}"), &patch_body, &token)
        .await;
    assert_eq!(status, StatusCode::OK);

    // Try to access
    let (status, body) = app
        .get_json_no_auth(&format!("/shared/{share_token}"))
        .await;
    assert_eq!(status, StatusCode::GONE);
    common::assert_error(&body, "GONE");
}

#[tokio::test]
async fn reactivate_link_works() {
    let app = TestApp::new().await;
    let token = app.setup_user_token("alice@test.com", TEST_PASSWORD).await;

    app.create_item(&token, &serde_json::json!({ "name": "PS5" }))
        .await;

    let (_, link_body) = app.post_with_auth("/share-links", &token).await;
    let link_id = link_body["id"].as_str().unwrap();
    let share_token = link_body["token"].as_str().unwrap();

    // Deactivate then reactivate
    app.patch_json_with_auth(
        &format!("/share-links/{link_id}"),
        &serde_json::json!({ "is_active": false }),
        &token,
    )
    .await;
    app.patch_json_with_auth(
        &format!("/share-links/{link_id}"),
        &serde_json::json!({ "is_active": true }),
        &token,
    )
    .await;

    let (status, _) = app
        .get_json_no_auth(&format!("/shared/{share_token}"))
        .await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn claim_deactivated_link_returns_410() {
    let app = TestApp::new().await;
    let alice_token = app.setup_user_token("alice@test.com", TEST_PASSWORD).await;
    let bob_token = app.setup_user_token("bob@test.com", TEST_PASSWORD).await;

    let item = app
        .create_item(&alice_token, &serde_json::json!({ "name": "PS5" }))
        .await;
    let item_id = item["id"].as_str().unwrap();

    let (_, link_body) = app.post_with_auth("/share-links", &alice_token).await;
    let link_id = link_body["id"].as_str().unwrap();
    let share_token = link_body["token"].as_str().unwrap();

    // Deactivate
    app.patch_json_with_auth(
        &format!("/share-links/{link_id}"),
        &serde_json::json!({ "is_active": false }),
        &alice_token,
    )
    .await;

    let (status, body) = app
        .post_with_auth(
            &format!("/shared/{share_token}/items/{item_id}/claim"),
            &bob_token,
        )
        .await;
    assert_eq!(status, StatusCode::GONE);
    common::assert_error(&body, "GONE");
}

// ── PATCH tests ──────────────────────────────────────────────────────

#[tokio::test]
async fn update_share_link_label() {
    let app = TestApp::new().await;
    let token = app.setup_user_token("alice@test.com", TEST_PASSWORD).await;

    let (_, link_body) = app.post_with_auth("/share-links", &token).await;
    let link_id = link_body["id"].as_str().unwrap();

    let patch = serde_json::json!({ "label": "Noël 2026" });
    let (status, resp) = app
        .patch_json_with_auth(&format!("/share-links/{link_id}"), &patch, &token)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["label"], "Noël 2026");
}

#[tokio::test]
async fn update_share_link_toggle_active() {
    let app = TestApp::new().await;
    let token = app.setup_user_token("alice@test.com", TEST_PASSWORD).await;

    let (_, link_body) = app.post_with_auth("/share-links", &token).await;
    let link_id = link_body["id"].as_str().unwrap();

    let patch = serde_json::json!({ "is_active": false });
    let (status, resp) = app
        .patch_json_with_auth(&format!("/share-links/{link_id}"), &patch, &token)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["is_active"], false);
}

#[tokio::test]
async fn update_share_link_permissions() {
    let app = TestApp::new().await;
    let token = app.setup_user_token("alice@test.com", TEST_PASSWORD).await;

    let (_, link_body) = app.post_with_auth("/share-links", &token).await;
    let link_id = link_body["id"].as_str().unwrap();

    let patch = serde_json::json!({ "permissions": "view_only" });
    let (status, resp) = app
        .patch_json_with_auth(&format!("/share-links/{link_id}"), &patch, &token)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["permissions"], "view_only");
}

#[tokio::test]
async fn update_another_users_link_returns_404() {
    let app = TestApp::new().await;
    let alice_token = app.setup_user_token("alice@test.com", TEST_PASSWORD).await;
    let bob_token = app.setup_user_token("bob@test.com", TEST_PASSWORD).await;

    let (_, link_body) = app.post_with_auth("/share-links", &alice_token).await;
    let link_id = link_body["id"].as_str().unwrap();

    let patch = serde_json::json!({ "label": "Hacked" });
    let (status, _) = app
        .patch_json_with_auth(&format!("/share-links/{link_id}"), &patch, &bob_token)
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

// ── Content negotiation tests ────────────────────────────────────────

#[tokio::test]
async fn shared_view_returns_html_by_default() {
    let app = TestApp::new().await;
    let token = app.setup_user_token("alice@test.com", TEST_PASSWORD).await;

    let (_, link_body) = app.post_with_auth("/share-links", &token).await;
    let share_token = link_body["token"].as_str().unwrap();

    let (status, bytes) = app
        .get_with_accept(&format!("/shared/{share_token}"), "text/html")
        .await;
    assert_eq!(status, StatusCode::OK);
    let html = String::from_utf8(bytes).unwrap();
    assert!(html.contains("<!DOCTYPE html>"));
    assert!(html.contains("Offrii"));
}

#[tokio::test]
async fn shared_view_returns_json_when_requested() {
    let app = TestApp::new().await;
    let token = app.setup_user_token("alice@test.com", TEST_PASSWORD).await;

    app.create_item(&token, &serde_json::json!({ "name": "PS5" }))
        .await;

    let (_, link_body) = app.post_with_auth("/share-links", &token).await;
    let share_token = link_body["token"].as_str().unwrap();

    let (status, bytes) = app
        .get_with_accept(&format!("/shared/{share_token}"), "application/json")
        .await;
    assert_eq!(status, StatusCode::OK);
    let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert!(body["items"].is_array());
    assert_eq!(body["items"][0]["name"], "PS5");
}

#[tokio::test]
async fn html_page_contains_items() {
    let app = TestApp::new().await;
    let token = app.setup_user_token("alice@test.com", TEST_PASSWORD).await;

    app.create_item(&token, &serde_json::json!({ "name": "Nintendo Switch" }))
        .await;

    let (_, link_body) = app.post_with_auth("/share-links", &token).await;
    let share_token = link_body["token"].as_str().unwrap();

    let (status, bytes) = app
        .get_with_accept(&format!("/shared/{share_token}"), "text/html")
        .await;
    assert_eq!(status, StatusCode::OK);
    let html = String::from_utf8(bytes).unwrap();
    assert!(html.contains("Nintendo Switch"));
}

#[tokio::test]
async fn html_page_respects_scope() {
    let app = TestApp::new().await;
    let token = app.setup_user_token("alice@test.com", TEST_PASSWORD).await;

    let item1 = app
        .create_item(&token, &serde_json::json!({ "name": "PS5" }))
        .await;
    app.create_item(&token, &serde_json::json!({ "name": "Xbox" }))
        .await;
    let item1_id = item1["id"].as_str().unwrap();

    let body = serde_json::json!({
        "scope": "selection",
        "scope_data": { "item_ids": [item1_id] }
    });
    let (_, link_body) = app.post_json_with_auth("/share-links", &body, &token).await;
    let share_token = link_body["token"].as_str().unwrap();

    let (status, bytes) = app
        .get_with_accept(&format!("/shared/{share_token}"), "text/html")
        .await;
    assert_eq!(status, StatusCode::OK);
    let html = String::from_utf8(bytes).unwrap();
    assert!(html.contains("PS5"));
    assert!(!html.contains("Xbox"));
}

// ── Empty PATCH body test ────────────────────────────────────────────

#[tokio::test]
async fn empty_patch_body_returns_400() {
    let app = TestApp::new().await;
    let token = app.setup_user_token("alice@test.com", TEST_PASSWORD).await;

    let (_, link_body) = app.post_with_auth("/share-links", &token).await;
    let link_id = link_body["id"].as_str().unwrap();

    let patch = serde_json::json!({});
    let (status, body) = app
        .patch_json_with_auth(&format!("/share-links/{link_id}"), &patch, &token)
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    common::assert_error(&body, "BAD_REQUEST");
}

// ── Unclaim permissions test ─────────────────────────────────────────

#[tokio::test]
async fn unclaim_view_only_link_returns_403() {
    let app = TestApp::new().await;
    let alice_token = app.setup_user_token("alice@test.com", TEST_PASSWORD).await;
    let bob_token = app.setup_user_token("bob@test.com", TEST_PASSWORD).await;

    let item = app
        .create_item(&alice_token, &serde_json::json!({ "name": "PS5" }))
        .await;
    let item_id = item["id"].as_str().unwrap();

    // Create a view_and_claim link first so Bob can claim
    let (_, claim_link) = app.post_with_auth("/share-links", &alice_token).await;
    let claim_token = claim_link["token"].as_str().unwrap();

    // Bob claims via the view_and_claim link
    let (status, _) = app
        .post_with_auth(
            &format!("/shared/{claim_token}/items/{item_id}/claim"),
            &bob_token,
        )
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Create a view_only link
    let body = serde_json::json!({ "permissions": "view_only" });
    let (_, view_link) = app
        .post_json_with_auth("/share-links", &body, &alice_token)
        .await;
    let view_token = view_link["token"].as_str().unwrap();

    // Bob tries to unclaim via the view_only link
    let (status, resp) = app
        .delete_with_auth(
            &format!("/shared/{view_token}/items/{item_id}/claim"),
            &bob_token,
        )
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    common::assert_error(&resp, "FORBIDDEN");
}

// ── Retrocompatibility: default shared view returns JSON via get_no_auth ──

#[tokio::test]
async fn create_share_link_returns_new_fields() {
    let app = TestApp::new().await;
    let token = app.setup_user_token("alice@test.com", TEST_PASSWORD).await;

    let (status, body) = app.post_with_auth("/share-links", &token).await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["permissions"], "view_and_claim");
    assert_eq!(body["scope"], "all");
    assert_eq!(body["is_active"], true);
    assert!(body["scope_data"].is_null());
    assert!(body["label"].is_null());
}

#[tokio::test]
async fn list_share_links_returns_new_fields() {
    let app = TestApp::new().await;
    let token = app.setup_user_token("alice@test.com", TEST_PASSWORD).await;

    app.post_with_auth("/share-links", &token).await;

    let (status, body) = app.get_with_auth("/share-links", &token).await;
    assert_eq!(status, StatusCode::OK);
    let links = body.as_array().unwrap();
    assert_eq!(links[0]["permissions"], "view_and_claim");
    assert_eq!(links[0]["scope"], "all");
    assert_eq!(links[0]["is_active"], true);
}

// ── HTML shared page tests ──────────────────────────────────────────

#[tokio::test]
async fn shared_view_html_contains_items() {
    let app = TestApp::new().await;
    let token = app.setup_user_token("alice@test.com", TEST_PASSWORD).await;

    app.create_item(&token, &serde_json::json!({ "name": "Nintendo Switch" }))
        .await;
    app.create_item(&token, &serde_json::json!({ "name": "AirPods Pro" }))
        .await;

    let (_, link_body) = app.post_with_auth("/share-links", &token).await;
    let share_token = link_body["token"].as_str().unwrap();

    let (status, bytes) = app
        .get_with_accept(&format!("/shared/{share_token}"), "text/html")
        .await;
    assert_eq!(status, StatusCode::OK);
    let html = String::from_utf8(bytes).unwrap();
    assert!(
        html.contains("Nintendo Switch"),
        "HTML should contain item name 'Nintendo Switch'"
    );
    assert!(
        html.contains("AirPods Pro"),
        "HTML should contain item name 'AirPods Pro'"
    );
}

#[tokio::test]
async fn shared_view_html_claim_buttons() {
    let app = TestApp::new().await;
    let token = app.setup_user_token("alice@test.com", TEST_PASSWORD).await;

    app.create_item(&token, &serde_json::json!({ "name": "PS5" }))
        .await;

    let body = serde_json::json!({ "permissions": "view_and_claim" });
    let (_, link_body) = app.post_json_with_auth("/share-links", &body, &token).await;
    let share_token = link_body["token"].as_str().unwrap();

    let (status, bytes) = app
        .get_with_accept(&format!("/shared/{share_token}"), "text/html")
        .await;
    assert_eq!(status, StatusCode::OK);
    let html = String::from_utf8(bytes).unwrap();
    assert!(
        html.contains("Je m\u{2019}en occupe") || html.contains("Je m'en occupe"),
        "HTML should contain claim button text for view_and_claim links"
    );
}

#[tokio::test]
async fn shared_view_html_no_claim_for_view_only() {
    let app = TestApp::new().await;
    let token = app.setup_user_token("alice@test.com", TEST_PASSWORD).await;

    app.create_item(&token, &serde_json::json!({ "name": "PS5" }))
        .await;

    let body = serde_json::json!({ "permissions": "view_only" });
    let (_, link_body) = app.post_json_with_auth("/share-links", &body, &token).await;
    let share_token = link_body["token"].as_str().unwrap();

    let (status, bytes) = app
        .get_with_accept(&format!("/shared/{share_token}"), "text/html")
        .await;
    assert_eq!(status, StatusCode::OK);
    let html = String::from_utf8(bytes).unwrap();
    assert!(
        !html.contains("Je m\u{2019}en occupe") && !html.contains("Je m'en occupe"),
        "HTML should NOT contain claim button text for view_only links"
    );
}

#[tokio::test]
async fn shared_view_html_reserved_overlay() {
    let app = TestApp::new().await;
    let alice_token = app.setup_user_token("alice@test.com", TEST_PASSWORD).await;
    let bob_token = app.setup_user_token("bob@test.com", TEST_PASSWORD).await;

    let item = app
        .create_item(&alice_token, &serde_json::json!({ "name": "PS5" }))
        .await;
    let item_id = item["id"].as_str().unwrap();

    let (_, link_body) = app.post_with_auth("/share-links", &alice_token).await;
    let share_token = link_body["token"].as_str().unwrap();

    // Bob claims the item
    let (status, _) = app
        .post_with_auth(
            &format!("/shared/{share_token}/items/{item_id}/claim"),
            &bob_token,
        )
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Get HTML and check for reserved overlay
    let (status, bytes) = app
        .get_with_accept(&format!("/shared/{share_token}"), "text/html")
        .await;
    assert_eq!(status, StatusCode::OK);
    let html = String::from_utf8(bytes).unwrap();
    assert!(
        html.contains("claimed-overlay"),
        "HTML should contain claimed overlay for claimed items"
    );
}

#[tokio::test]
async fn shared_view_html_hides_private_items() {
    let app = TestApp::new().await;
    let token = app.setup_user_token("alice@test.com", TEST_PASSWORD).await;

    app.create_item(&token, &serde_json::json!({ "name": "Public Item" }))
        .await;
    app.create_item(
        &token,
        &serde_json::json!({ "name": "Secret Item", "is_private": true }),
    )
    .await;

    let (_, link_body) = app.post_with_auth("/share-links", &token).await;
    let share_token = link_body["token"].as_str().unwrap();

    let (status, bytes) = app
        .get_with_accept(&format!("/shared/{share_token}"), "text/html")
        .await;
    assert_eq!(status, StatusCode::OK);
    let html = String::from_utf8(bytes).unwrap();
    assert!(
        html.contains("Public Item"),
        "HTML should contain the public item"
    );
    assert!(
        !html.contains("Secret Item"),
        "HTML should NOT contain the private item"
    );
}

#[tokio::test]
async fn shared_view_html_og_meta_tags() {
    let app = TestApp::new().await;
    let token = app.setup_user_token("alice@test.com", TEST_PASSWORD).await;

    let (_, link_body) = app.post_with_auth("/share-links", &token).await;
    let share_token = link_body["token"].as_str().unwrap();

    let (status, bytes) = app
        .get_with_accept(&format!("/shared/{share_token}"), "text/html")
        .await;
    assert_eq!(status, StatusCode::OK);
    let html = String::from_utf8(bytes).unwrap();
    assert!(
        html.contains("og:title"),
        "HTML should contain og:title meta tag"
    );
    assert!(
        html.contains("og:description"),
        "HTML should contain og:description meta tag"
    );
    assert!(
        html.contains("og:type"),
        "HTML should contain og:type meta tag"
    );
}

// ── i18n tests ──────────────────────────────────────────────────────

#[tokio::test]
async fn shared_view_html_default_french() {
    let app = TestApp::new().await;
    let token = app.setup_user_token("alice@test.com", TEST_PASSWORD).await;

    let (_, link_body) = app.post_with_auth("/share-links", &token).await;
    let share_token = link_body["token"].as_str().unwrap();

    // No Accept-Language header — should default to French
    let (status, bytes) = app
        .get_with_accept(&format!("/shared/{share_token}"), "text/html")
        .await;
    assert_eq!(status, StatusCode::OK);
    let html = String::from_utf8(bytes).unwrap();
    assert!(
        html.contains("Les envies de"),
        "HTML should default to French and contain 'Les envies de'"
    );
}

#[tokio::test]
async fn shared_view_html_english() {
    let app = TestApp::new().await;
    let token = app.setup_user_token("alice@test.com", TEST_PASSWORD).await;

    let (_, link_body) = app.post_with_auth("/share-links", &token).await;
    let share_token = link_body["token"].as_str().unwrap();

    let req = axum::http::Request::builder()
        .method("GET")
        .uri(format!("/shared/{share_token}"))
        .header(header::ACCEPT, "text/html")
        .header(header::ACCEPT_LANGUAGE, "en")
        .body(Body::empty())
        .unwrap();
    let resp = app.router.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    let html = String::from_utf8(bytes.to_vec()).unwrap();
    assert!(
        html.contains("Wishes of"),
        "HTML should contain English text 'Wishes of' when Accept-Language is 'en'"
    );
}

// ── HTML error page tests ───────────────────────────────────────────

#[tokio::test]
async fn shared_view_expired_html_error() {
    let app = TestApp::new().await;
    let token = app.setup_user_token("alice@test.com", TEST_PASSWORD).await;

    let (_, link_body) = app.post_with_auth("/share-links", &token).await;
    let share_token = link_body["token"].as_str().unwrap();

    // Expire the link
    sqlx::query("UPDATE share_links SET expires_at = NOW() - INTERVAL '1 hour' WHERE token = $1")
        .bind(share_token)
        .execute(&app.db)
        .await
        .unwrap();

    let (status, bytes) = app
        .get_with_accept(&format!("/shared/{share_token}"), "text/html")
        .await;
    assert_eq!(status, StatusCode::OK);
    let html = String::from_utf8(bytes).unwrap();
    assert!(
        html.contains("expir"),
        "Expired link HTML error page should contain 'expir' substring, got: {html}"
    );
}

#[tokio::test]
async fn shared_view_disabled_html_error() {
    let app = TestApp::new().await;
    let token = app.setup_user_token("alice@test.com", TEST_PASSWORD).await;

    let (_, link_body) = app.post_with_auth("/share-links", &token).await;
    let link_id = link_body["id"].as_str().unwrap();
    let share_token = link_body["token"].as_str().unwrap();

    // Deactivate
    let patch_body = serde_json::json!({ "is_active": false });
    let (status, _) = app
        .patch_json_with_auth(&format!("/share-links/{link_id}"), &patch_body, &token)
        .await;
    assert_eq!(status, StatusCode::OK);

    let (status, bytes) = app
        .get_with_accept(&format!("/shared/{share_token}"), "text/html")
        .await;
    assert_eq!(status, StatusCode::OK);
    let html = String::from_utf8(bytes).unwrap();
    assert!(
        html.contains("actif"),
        "Disabled link HTML error page should contain 'actif' substring, got: {html}"
    );
}

#[tokio::test]
async fn shared_view_not_found_html_error() {
    let app = TestApp::new().await;

    let (status, bytes) = app
        .get_with_accept("/shared/nonexistent_token_xyz", "text/html")
        .await;
    assert_eq!(status, StatusCode::OK);
    let html = String::from_utf8(bytes).unwrap();
    assert!(
        html.contains("introuvable"),
        "Not-found link HTML error page should contain 'introuvable' substring, got: {html}"
    );
}

#[tokio::test]
async fn shared_view_expired_json_still_410() {
    let app = TestApp::new().await;
    let token = app.setup_user_token("alice@test.com", TEST_PASSWORD).await;

    let (_, link_body) = app.post_with_auth("/share-links", &token).await;
    let share_token = link_body["token"].as_str().unwrap();

    // Expire the link
    sqlx::query("UPDATE share_links SET expires_at = NOW() - INTERVAL '1 hour' WHERE token = $1")
        .bind(share_token)
        .execute(&app.db)
        .await
        .unwrap();

    let (status, bytes) = app
        .get_with_accept(&format!("/shared/{share_token}"), "application/json")
        .await;
    assert_eq!(status, StatusCode::GONE);
    let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    common::assert_error(&body, "GONE");
}
