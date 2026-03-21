mod common;

use axum::http::StatusCode;
use common::{TEST_PASSWORD, TestApp, assert_error};
use tower::ServiceExt;

const TEST_EMAIL: &str = "items@example.com";

// ── Create ──────────────────────────────────────────────────────────

#[tokio::test]
async fn create_item_quick_capture_201() {
    let app = TestApp::new().await;
    let token = app.setup_user_token(TEST_EMAIL, TEST_PASSWORD).await;

    let body = serde_json::json!({ "name": "écran" });
    let item = app.create_item(&token, &body).await;

    assert_eq!(item["name"], "écran");
    assert_eq!(item["priority"], 2);
    assert_eq!(item["status"], "active");
    assert!(item["id"].is_string());
    assert!(item["description"].is_null());
    assert!(item["url"].is_null());
    assert!(item["estimated_price"].is_null());
    assert!(item["category_id"].is_null());
    assert!(item["purchased_at"].is_null());
    assert!(item["created_at"].is_string());
    assert!(item["updated_at"].is_string());
    assert_eq!(item["is_claimed"], false);
}

#[tokio::test]
async fn create_item_full_201() {
    let app = TestApp::new().await;
    let token = app.setup_user_token(TEST_EMAIL, TEST_PASSWORD).await;

    // Get a global category for full test
    let cat_id: uuid::Uuid = sqlx::query_scalar("SELECT id FROM categories LIMIT 1")
        .fetch_one(&app.db)
        .await
        .unwrap();

    let body = serde_json::json!({
        "name": "MacBook Pro",
        "description": "Laptop for dev",
        "url": "https://apple.com/macbook",
        "estimated_price": "2499.99",
        "priority": 1,
        "category_id": cat_id.to_string(),
    });
    let (status, item) = app.post_json_with_auth("/items", &body, &token).await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(item["name"], "MacBook Pro");
    assert_eq!(item["description"], "Laptop for dev");
    assert_eq!(item["url"], "https://apple.com/macbook");
    assert_eq!(item["estimated_price"], "2499.99");
    assert_eq!(item["priority"], 1);
    assert_eq!(item["status"], "active");
    assert_eq!(item["category_id"], cat_id.to_string());
}

#[tokio::test]
async fn create_item_missing_name_422() {
    let app = TestApp::new().await;
    let token = app.setup_user_token(TEST_EMAIL, TEST_PASSWORD).await;

    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/items")
        .header(axum::http::header::CONTENT_TYPE, "application/json")
        .header(axum::http::header::AUTHORIZATION, format!("Bearer {token}"))
        .body(axum::body::Body::from(
            serde_json::to_vec(&serde_json::json!({})).unwrap(),
        ))
        .unwrap();

    let resp = app.router.clone().oneshot(req).await.unwrap();
    // 422 comes from Axum's Json<T> deserialization rejection, not our AppError,
    // so the body shape differs from assert_error expectations.
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn create_item_empty_name_400() {
    let app = TestApp::new().await;
    let token = app.setup_user_token(TEST_EMAIL, TEST_PASSWORD).await;

    let body = serde_json::json!({ "name": "" });
    let (status, resp) = app.post_json_with_auth("/items", &body, &token).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_error(&resp, "BAD_REQUEST");
}

#[tokio::test]
async fn create_item_invalid_priority_400() {
    let app = TestApp::new().await;
    let token = app.setup_user_token(TEST_EMAIL, TEST_PASSWORD).await;

    let body = serde_json::json!({ "name": "test", "priority": 5 });
    let (status, resp) = app.post_json_with_auth("/items", &body, &token).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_error(&resp, "BAD_REQUEST");
}

#[tokio::test]
async fn create_item_priority_0_400() {
    let app = TestApp::new().await;
    let token = app.setup_user_token(TEST_EMAIL, TEST_PASSWORD).await;

    let body = serde_json::json!({ "name": "test", "priority": 0 });
    let (status, resp) = app.post_json_with_auth("/items", &body, &token).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_error(&resp, "BAD_REQUEST");
}

#[tokio::test]
async fn create_item_priority_boundaries_201() {
    let app = TestApp::new().await;
    let token = app.setup_user_token(TEST_EMAIL, TEST_PASSWORD).await;

    for p in [1, 2, 3] {
        let body = serde_json::json!({ "name": format!("p{p}"), "priority": p });
        let (status, item) = app.post_json_with_auth("/items", &body, &token).await;
        assert_eq!(status, StatusCode::CREATED, "priority {p} should be valid");
        assert_eq!(item["priority"], p);
    }
}

#[tokio::test]
async fn create_item_negative_price_400() {
    let app = TestApp::new().await;
    let token = app.setup_user_token(TEST_EMAIL, TEST_PASSWORD).await;

    let body = serde_json::json!({ "name": "test", "estimated_price": "-1.00" });
    let (status, resp) = app.post_json_with_auth("/items", &body, &token).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_error(&resp, "BAD_REQUEST");
}

#[tokio::test]
async fn create_item_zero_price_201() {
    let app = TestApp::new().await;
    let token = app.setup_user_token(TEST_EMAIL, TEST_PASSWORD).await;

    let body = serde_json::json!({ "name": "free stuff", "estimated_price": "0.00" });
    let (status, item) = app.post_json_with_auth("/items", &body, &token).await;

    assert_eq!(status, StatusCode::CREATED);
    // rust_decimal with serde-with-str serializes zero as "0", not "0.00"
    let price = item["estimated_price"].as_str().unwrap();
    assert!(
        price == "0" || price == "0.00",
        "expected zero price, got {price}"
    );
}

#[tokio::test]
async fn create_item_nonexistent_category_400() {
    let app = TestApp::new().await;
    let token = app.setup_user_token(TEST_EMAIL, TEST_PASSWORD).await;

    let body = serde_json::json!({
        "name": "test",
        "category_id": uuid::Uuid::new_v4().to_string(),
    });
    let (status, resp) = app.post_json_with_auth("/items", &body, &token).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_error(&resp, "BAD_REQUEST");
}

#[tokio::test]
async fn create_item_invalid_category_400() {
    let app = TestApp::new().await;
    let token = app
        .setup_user_token("catthief@example.com", TEST_PASSWORD)
        .await;

    // Try to use a non-existent category UUID
    let fake_cat = uuid::Uuid::new_v4();
    let body = serde_json::json!({
        "name": "test",
        "category_id": fake_cat.to_string(),
    });
    let (status, resp) = app.post_json_with_auth("/items", &body, &token).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_error(&resp, "BAD_REQUEST");
}

#[tokio::test]
async fn create_item_without_auth_401() {
    let app = TestApp::new().await;

    let body = serde_json::json!({ "name": "test" });
    let (status, _) = app.post_json("/items", &body).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

// ── Get ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn get_item_200() {
    let app = TestApp::new().await;
    let token = app.setup_user_token(TEST_EMAIL, TEST_PASSWORD).await;

    let item = app
        .create_item(&token, &serde_json::json!({ "name": "clavier" }))
        .await;
    let id = item["id"].as_str().unwrap();

    let (status, fetched) = app.get_with_auth(&format!("/items/{id}"), &token).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(fetched["id"], id);
    assert_eq!(fetched["name"], "clavier");
}

#[tokio::test]
async fn get_item_other_user_404() {
    let app = TestApp::new().await;
    let token1 = app
        .setup_user_token("user1@example.com", TEST_PASSWORD)
        .await;
    let token2 = app
        .setup_user_token("user2@example.com", TEST_PASSWORD)
        .await;

    let item = app
        .create_item(&token1, &serde_json::json!({ "name": "private" }))
        .await;
    let id = item["id"].as_str().unwrap();

    let (status, resp) = app.get_with_auth(&format!("/items/{id}"), &token2).await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_error(&resp, "NOT_FOUND");
}

#[tokio::test]
async fn get_item_not_found_404() {
    let app = TestApp::new().await;
    let token = app.setup_user_token(TEST_EMAIL, TEST_PASSWORD).await;

    let fake_id = uuid::Uuid::new_v4();
    let (status, resp) = app
        .get_with_auth(&format!("/items/{fake_id}"), &token)
        .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_error(&resp, "NOT_FOUND");
}

#[tokio::test]
async fn get_item_without_auth_401() {
    let app = TestApp::new().await;
    let token = app.setup_user_token(TEST_EMAIL, TEST_PASSWORD).await;

    let item = app
        .create_item(&token, &serde_json::json!({ "name": "x" }))
        .await;
    let id = item["id"].as_str().unwrap();

    let (status, _) = app.get_no_auth(&format!("/items/{id}")).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

// ── List ────────────────────────────────────────────────────────────

#[tokio::test]
async fn list_items_default_200() {
    let app = TestApp::new().await;
    let token = app.setup_user_token(TEST_EMAIL, TEST_PASSWORD).await;

    app.create_item(&token, &serde_json::json!({ "name": "item1" }))
        .await;
    app.create_item(&token, &serde_json::json!({ "name": "item2" }))
        .await;

    let (status, body) = app.get_with_auth("/items", &token).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"].as_array().unwrap().len(), 2);
    assert_eq!(body["pagination"]["total"], 2);
    assert_eq!(body["pagination"]["page"], 1);
    assert_eq!(body["pagination"]["limit"], 20);
}

#[tokio::test]
async fn list_items_pagination() {
    let app = TestApp::new().await;
    let token = app.setup_user_token(TEST_EMAIL, TEST_PASSWORD).await;

    for i in 0..5 {
        app.create_item(&token, &serde_json::json!({ "name": format!("item{i}") }))
            .await;
    }

    let (status, body) = app.get_with_auth("/items?page=2&limit=2", &token).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"].as_array().unwrap().len(), 2);
    assert_eq!(body["pagination"]["total"], 5);
    assert_eq!(body["pagination"]["page"], 2);
    assert_eq!(body["pagination"]["limit"], 2);
}

#[tokio::test]
async fn list_items_page_beyond_total_returns_empty() {
    let app = TestApp::new().await;
    let token = app.setup_user_token(TEST_EMAIL, TEST_PASSWORD).await;

    app.create_item(&token, &serde_json::json!({ "name": "x" }))
        .await;

    let (status, body) = app.get_with_auth("/items?page=100&limit=50", &token).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"].as_array().unwrap().len(), 0);
    assert_eq!(body["pagination"]["total"], 1);
}

#[tokio::test]
async fn list_items_limit_clamped() {
    let app = TestApp::new().await;
    let token = app.setup_user_token(TEST_EMAIL, TEST_PASSWORD).await;

    // limit=0 should clamp to 1
    let (status, body) = app.get_with_auth("/items?limit=0", &token).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["pagination"]["limit"], 1);

    // limit=200 should clamp to 100
    let (status, body) = app.get_with_auth("/items?limit=200", &token).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["pagination"]["limit"], 100);
}

#[tokio::test]
async fn list_items_page_zero_clamps_to_1() {
    let app = TestApp::new().await;
    let token = app.setup_user_token(TEST_EMAIL, TEST_PASSWORD).await;

    let (status, body) = app.get_with_auth("/items?page=0", &token).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["pagination"]["page"], 1);
}

#[tokio::test]
async fn list_items_filter_status() {
    let app = TestApp::new().await;
    let token = app.setup_user_token(TEST_EMAIL, TEST_PASSWORD).await;

    let item = app
        .create_item(&token, &serde_json::json!({ "name": "bought" }))
        .await;
    let id = item["id"].as_str().unwrap();
    app.create_item(&token, &serde_json::json!({ "name": "active_one" }))
        .await;

    app.put_json_with_auth(
        &format!("/items/{id}"),
        &serde_json::json!({ "status": "purchased" }),
        &token,
    )
    .await;

    // Filter purchased
    let (_, body) = app.get_with_auth("/items?status=purchased", &token).await;
    assert_eq!(body["data"].as_array().unwrap().len(), 1);
    assert_eq!(body["data"][0]["name"], "bought");

    // Filter active
    let (_, body) = app.get_with_auth("/items?status=active", &token).await;
    assert_eq!(body["data"].as_array().unwrap().len(), 1);
    assert_eq!(body["data"][0]["name"], "active_one");
}

#[tokio::test]
async fn list_items_invalid_sort_400() {
    let app = TestApp::new().await;
    let token = app.setup_user_token(TEST_EMAIL, TEST_PASSWORD).await;

    let (status, resp) = app.get_with_auth("/items?sort=nonexistent", &token).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_error(&resp, "BAD_REQUEST");
}

#[tokio::test]
async fn list_items_invalid_order_400() {
    let app = TestApp::new().await;
    let token = app.setup_user_token(TEST_EMAIL, TEST_PASSWORD).await;

    let (status, resp) = app.get_with_auth("/items?order=random", &token).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_error(&resp, "BAD_REQUEST");
}

#[tokio::test]
async fn list_items_invalid_status_filter_400() {
    let app = TestApp::new().await;
    let token = app.setup_user_token(TEST_EMAIL, TEST_PASSWORD).await;

    let (status, resp) = app.get_with_auth("/items?status=deleted", &token).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_error(&resp, "BAD_REQUEST");
}

#[tokio::test]
async fn list_items_filter_category() {
    let app = TestApp::new().await;
    let token = app.setup_user_token(TEST_EMAIL, TEST_PASSWORD).await;

    let cat_id: uuid::Uuid = sqlx::query_scalar("SELECT id FROM categories LIMIT 1")
        .fetch_one(&app.db)
        .await
        .unwrap();

    app.create_item(
        &token,
        &serde_json::json!({ "name": "with_cat", "category_id": cat_id.to_string() }),
    )
    .await;
    app.create_item(&token, &serde_json::json!({ "name": "no_cat" }))
        .await;

    let (status, body) = app
        .get_with_auth(&format!("/items?category_id={cat_id}"), &token)
        .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"].as_array().unwrap().len(), 1);
    assert_eq!(body["data"][0]["name"], "with_cat");
}

#[tokio::test]
async fn list_items_sort_priority() {
    let app = TestApp::new().await;
    let token = app.setup_user_token(TEST_EMAIL, TEST_PASSWORD).await;

    app.create_item(&token, &serde_json::json!({ "name": "low", "priority": 3 }))
        .await;
    app.create_item(
        &token,
        &serde_json::json!({ "name": "high", "priority": 1 }),
    )
    .await;
    app.create_item(
        &token,
        &serde_json::json!({ "name": "medium", "priority": 2 }),
    )
    .await;

    let (_, body) = app
        .get_with_auth("/items?sort=priority&order=asc", &token)
        .await;

    let items = body["data"].as_array().unwrap();
    assert_eq!(items[0]["name"], "high");
    assert_eq!(items[1]["name"], "medium");
    assert_eq!(items[2]["name"], "low");
}

#[tokio::test]
async fn list_items_sort_name() {
    let app = TestApp::new().await;
    let token = app.setup_user_token(TEST_EMAIL, TEST_PASSWORD).await;

    app.create_item(&token, &serde_json::json!({ "name": "Banana" }))
        .await;
    app.create_item(&token, &serde_json::json!({ "name": "Apple" }))
        .await;
    app.create_item(&token, &serde_json::json!({ "name": "Cherry" }))
        .await;

    let (_, body) = app
        .get_with_auth("/items?sort=name&order=asc", &token)
        .await;

    let items = body["data"].as_array().unwrap();
    assert_eq!(items[0]["name"], "Apple");
    assert_eq!(items[1]["name"], "Banana");
    assert_eq!(items[2]["name"], "Cherry");
}

#[tokio::test]
async fn list_items_empty_200() {
    let app = TestApp::new().await;
    let token = app.setup_user_token(TEST_EMAIL, TEST_PASSWORD).await;

    let (status, body) = app.get_with_auth("/items", &token).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"].as_array().unwrap().len(), 0);
    assert_eq!(body["pagination"]["total"], 0);
}

#[tokio::test]
async fn list_items_without_auth_401() {
    let app = TestApp::new().await;

    let (status, _) = app.get_no_auth("/items").await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn list_items_isolation_between_users() {
    let app = TestApp::new().await;
    let token_a = app.setup_user_token("a@example.com", TEST_PASSWORD).await;
    let token_b = app.setup_user_token("b@example.com", TEST_PASSWORD).await;

    app.create_item(&token_a, &serde_json::json!({ "name": "a_item" }))
        .await;
    app.create_item(&token_b, &serde_json::json!({ "name": "b_item" }))
        .await;

    let (_, body_a) = app.get_with_auth("/items", &token_a).await;
    let (_, body_b) = app.get_with_auth("/items", &token_b).await;

    assert_eq!(body_a["pagination"]["total"], 1);
    assert_eq!(body_a["data"][0]["name"], "a_item");
    assert_eq!(body_b["pagination"]["total"], 1);
    assert_eq!(body_b["data"][0]["name"], "b_item");
}

// ── Update ──────────────────────────────────────────────────────────

#[tokio::test]
async fn update_item_200() {
    let app = TestApp::new().await;
    let token = app.setup_user_token(TEST_EMAIL, TEST_PASSWORD).await;

    let item = app
        .create_item(&token, &serde_json::json!({ "name": "original" }))
        .await;
    let id = item["id"].as_str().unwrap();
    let original_updated_at = item["updated_at"].as_str().unwrap().to_string();

    // Delay so the DB trigger's NOW() advances past the original timestamp.
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    let (status, updated) = app
        .put_json_with_auth(
            &format!("/items/{id}"),
            &serde_json::json!({
                "name": "renamed",
                "description": "added description",
                "priority": 1,
            }),
            &token,
        )
        .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(updated["name"], "renamed");
    assert_eq!(updated["description"], "added description");
    assert_eq!(updated["priority"], 1);
    // Verify updated_at trigger fired
    assert_ne!(
        updated["updated_at"].as_str().unwrap(),
        original_updated_at,
        "updated_at should change after update"
    );
}

#[tokio::test]
async fn update_item_status_purchased() {
    let app = TestApp::new().await;
    let token = app.setup_user_token(TEST_EMAIL, TEST_PASSWORD).await;

    let item = app
        .create_item(&token, &serde_json::json!({ "name": "to buy" }))
        .await;
    let id = item["id"].as_str().unwrap();
    assert!(item["purchased_at"].is_null());

    let (status, updated) = app
        .put_json_with_auth(
            &format!("/items/{id}"),
            &serde_json::json!({ "status": "purchased" }),
            &token,
        )
        .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(updated["status"], "purchased");
    assert!(
        updated["purchased_at"].is_string(),
        "purchased_at should be auto-set by trigger"
    );
}

#[tokio::test]
async fn update_item_purchased_to_active_clears_purchased_at() {
    let app = TestApp::new().await;
    let token = app.setup_user_token(TEST_EMAIL, TEST_PASSWORD).await;

    let item = app
        .create_item(&token, &serde_json::json!({ "name": "flip" }))
        .await;
    let id = item["id"].as_str().unwrap();

    // Mark purchased
    let (_, purchased) = app
        .put_json_with_auth(
            &format!("/items/{id}"),
            &serde_json::json!({ "status": "purchased" }),
            &token,
        )
        .await;
    assert!(purchased["purchased_at"].is_string());

    // Revert to active
    let (status, active) = app
        .put_json_with_auth(
            &format!("/items/{id}"),
            &serde_json::json!({ "status": "active" }),
            &token,
        )
        .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(active["status"], "active");
    assert!(
        active["purchased_at"].is_null(),
        "purchased_at should be cleared when reverting to active"
    );
}

#[tokio::test]
async fn update_item_invalid_status_400() {
    let app = TestApp::new().await;
    let token = app.setup_user_token(TEST_EMAIL, TEST_PASSWORD).await;

    let item = app
        .create_item(&token, &serde_json::json!({ "name": "test" }))
        .await;
    let id = item["id"].as_str().unwrap();

    let (status, resp) = app
        .put_json_with_auth(
            &format!("/items/{id}"),
            &serde_json::json!({ "status": "deleted" }),
            &token,
        )
        .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_error(&resp, "BAD_REQUEST");
}

#[tokio::test]
async fn update_item_negative_price_400() {
    let app = TestApp::new().await;
    let token = app.setup_user_token(TEST_EMAIL, TEST_PASSWORD).await;

    let item = app
        .create_item(&token, &serde_json::json!({ "name": "test" }))
        .await;
    let id = item["id"].as_str().unwrap();

    let (status, resp) = app
        .put_json_with_auth(
            &format!("/items/{id}"),
            &serde_json::json!({ "estimated_price": "-5.00" }),
            &token,
        )
        .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_error(&resp, "BAD_REQUEST");
}

#[tokio::test]
async fn update_item_invalid_priority_400() {
    let app = TestApp::new().await;
    let token = app.setup_user_token(TEST_EMAIL, TEST_PASSWORD).await;

    let item = app
        .create_item(&token, &serde_json::json!({ "name": "test" }))
        .await;
    let id = item["id"].as_str().unwrap();

    let (status, resp) = app
        .put_json_with_auth(
            &format!("/items/{id}"),
            &serde_json::json!({ "priority": 4 }),
            &token,
        )
        .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_error(&resp, "BAD_REQUEST");
}

#[tokio::test]
async fn update_item_not_found_404() {
    let app = TestApp::new().await;
    let token = app.setup_user_token(TEST_EMAIL, TEST_PASSWORD).await;

    let fake_id = uuid::Uuid::new_v4();
    let (status, resp) = app
        .put_json_with_auth(
            &format!("/items/{fake_id}"),
            &serde_json::json!({ "name": "ghost" }),
            &token,
        )
        .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_error(&resp, "NOT_FOUND");
}

#[tokio::test]
async fn update_item_other_user_404() {
    let app = TestApp::new().await;
    let token1 = app
        .setup_user_token("owner@example.com", TEST_PASSWORD)
        .await;
    let token2 = app
        .setup_user_token("other@example.com", TEST_PASSWORD)
        .await;

    let item = app
        .create_item(&token1, &serde_json::json!({ "name": "mine" }))
        .await;
    let id = item["id"].as_str().unwrap();

    let (status, resp) = app
        .put_json_with_auth(
            &format!("/items/{id}"),
            &serde_json::json!({ "name": "stolen" }),
            &token2,
        )
        .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_error(&resp, "NOT_FOUND");
}

#[tokio::test]
async fn update_deleted_item_404() {
    let app = TestApp::new().await;
    let token = app.setup_user_token(TEST_EMAIL, TEST_PASSWORD).await;

    let item = app
        .create_item(&token, &serde_json::json!({ "name": "doomed" }))
        .await;
    let id = item["id"].as_str().unwrap();

    app.delete_with_auth(&format!("/items/{id}"), &token).await;

    let (status, resp) = app
        .put_json_with_auth(
            &format!("/items/{id}"),
            &serde_json::json!({ "name": "revived?" }),
            &token,
        )
        .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_error(&resp, "NOT_FOUND");
}

#[tokio::test]
async fn update_item_without_auth_401() {
    let app = TestApp::new().await;
    let token = app.setup_user_token(TEST_EMAIL, TEST_PASSWORD).await;

    let item = app
        .create_item(&token, &serde_json::json!({ "name": "x" }))
        .await;
    let id = item["id"].as_str().unwrap();

    // PUT without auth
    let req = axum::http::Request::builder()
        .method("PUT")
        .uri(format!("/items/{id}"))
        .header(axum::http::header::CONTENT_TYPE, "application/json")
        .body(axum::body::Body::from(
            serde_json::to_vec(&serde_json::json!({ "name": "hack" })).unwrap(),
        ))
        .unwrap();

    let resp = app.router.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

// ── Status transition guards ─────────────────────────────────────────

#[tokio::test]
async fn purchase_already_purchased_409() {
    let app = TestApp::new().await;
    let token = app.setup_user_token(TEST_EMAIL, TEST_PASSWORD).await;

    let item = app
        .create_item(&token, &serde_json::json!({ "name": "gadget" }))
        .await;
    let id = item["id"].as_str().unwrap();

    // First purchase — should succeed
    let (status, _) = app
        .put_json_with_auth(
            &format!("/items/{id}"),
            &serde_json::json!({ "status": "purchased" }),
            &token,
        )
        .await;
    assert_eq!(status, StatusCode::OK);

    // Second purchase — should be 409
    let (status, resp) = app
        .put_json_with_auth(
            &format!("/items/{id}"),
            &serde_json::json!({ "status": "purchased" }),
            &token,
        )
        .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_error(&resp, "CONFLICT");
}

#[tokio::test]
async fn restore_already_active_409() {
    let app = TestApp::new().await;
    let token = app.setup_user_token(TEST_EMAIL, TEST_PASSWORD).await;

    let item = app
        .create_item(&token, &serde_json::json!({ "name": "already active" }))
        .await;
    let id = item["id"].as_str().unwrap();

    // Item is already active — setting active again should be 409
    let (status, resp) = app
        .put_json_with_auth(
            &format!("/items/{id}"),
            &serde_json::json!({ "status": "active" }),
            &token,
        )
        .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_error(&resp, "CONFLICT");
}

#[tokio::test]
async fn purchase_then_restore_then_purchase_200() {
    let app = TestApp::new().await;
    let token = app.setup_user_token(TEST_EMAIL, TEST_PASSWORD).await;

    let item = app
        .create_item(&token, &serde_json::json!({ "name": "cycle" }))
        .await;
    let id = item["id"].as_str().unwrap();

    // active → purchased
    let (status, _) = app
        .put_json_with_auth(
            &format!("/items/{id}"),
            &serde_json::json!({ "status": "purchased" }),
            &token,
        )
        .await;
    assert_eq!(status, StatusCode::OK);

    // purchased → active (restore)
    let (status, restored) = app
        .put_json_with_auth(
            &format!("/items/{id}"),
            &serde_json::json!({ "status": "active" }),
            &token,
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(restored["status"], "active");
    assert!(restored["purchased_at"].is_null());

    // active → purchased (again)
    let (status, repurchased) = app
        .put_json_with_auth(
            &format!("/items/{id}"),
            &serde_json::json!({ "status": "purchased" }),
            &token,
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(repurchased["status"], "purchased");
    assert!(repurchased["purchased_at"].is_string());
}

#[tokio::test]
async fn purchase_deleted_item_404() {
    let app = TestApp::new().await;
    let token = app.setup_user_token(TEST_EMAIL, TEST_PASSWORD).await;

    let item = app
        .create_item(&token, &serde_json::json!({ "name": "to delete" }))
        .await;
    let id = item["id"].as_str().unwrap();

    app.delete_with_auth(&format!("/items/{id}"), &token).await;

    let (status, resp) = app
        .put_json_with_auth(
            &format!("/items/{id}"),
            &serde_json::json!({ "status": "purchased" }),
            &token,
        )
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_error(&resp, "NOT_FOUND");
}

#[tokio::test]
async fn update_non_status_fields_on_purchased_item_200() {
    let app = TestApp::new().await;
    let token = app.setup_user_token(TEST_EMAIL, TEST_PASSWORD).await;

    let item = app
        .create_item(&token, &serde_json::json!({ "name": "original" }))
        .await;
    let id = item["id"].as_str().unwrap();

    // Mark purchased
    let (status, _) = app
        .put_json_with_auth(
            &format!("/items/{id}"),
            &serde_json::json!({ "status": "purchased" }),
            &token,
        )
        .await;
    assert_eq!(status, StatusCode::OK);

    // Update only non-status fields — should not trigger guard
    let (status, updated) = app
        .put_json_with_auth(
            &format!("/items/{id}"),
            &serde_json::json!({ "name": "renamed" }),
            &token,
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(updated["name"], "renamed");
    assert_eq!(updated["status"], "purchased");
}

#[tokio::test]
async fn update_name_and_same_status_409() {
    let app = TestApp::new().await;
    let token = app.setup_user_token(TEST_EMAIL, TEST_PASSWORD).await;

    let item = app
        .create_item(&token, &serde_json::json!({ "name": "mixed" }))
        .await;
    let id = item["id"].as_str().unwrap();

    // Mark purchased
    let (status, _) = app
        .put_json_with_auth(
            &format!("/items/{id}"),
            &serde_json::json!({ "status": "purchased" }),
            &token,
        )
        .await;
    assert_eq!(status, StatusCode::OK);

    // Send name + same status — entire request should be rejected with 409
    let (status, resp) = app
        .put_json_with_auth(
            &format!("/items/{id}"),
            &serde_json::json!({ "name": "renamed", "status": "purchased" }),
            &token,
        )
        .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_error(&resp, "CONFLICT");
}

// ── Delete ──────────────────────────────────────────────────────────

#[tokio::test]
async fn delete_item_204() {
    let app = TestApp::new().await;
    let token = app.setup_user_token(TEST_EMAIL, TEST_PASSWORD).await;

    let item = app
        .create_item(&token, &serde_json::json!({ "name": "to delete" }))
        .await;
    let id = item["id"].as_str().unwrap();

    let (status, body) = app.delete_with_auth(&format!("/items/{id}"), &token).await;

    assert_eq!(status, StatusCode::NO_CONTENT);
    assert!(body.is_null(), "204 response should have no body");
}

#[tokio::test]
async fn delete_item_not_in_list() {
    let app = TestApp::new().await;
    let token = app.setup_user_token(TEST_EMAIL, TEST_PASSWORD).await;

    let item = app
        .create_item(&token, &serde_json::json!({ "name": "gone" }))
        .await;
    let id = item["id"].as_str().unwrap();

    app.delete_with_auth(&format!("/items/{id}"), &token).await;

    // Not in list
    let (_, body) = app.get_with_auth("/items", &token).await;
    assert_eq!(body["data"].as_array().unwrap().len(), 0);

    // Not findable by ID
    let (status, resp) = app.get_with_auth(&format!("/items/{id}"), &token).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_error(&resp, "NOT_FOUND");
}

#[tokio::test]
async fn delete_already_deleted_item_404() {
    let app = TestApp::new().await;
    let token = app.setup_user_token(TEST_EMAIL, TEST_PASSWORD).await;

    let item = app
        .create_item(&token, &serde_json::json!({ "name": "double_del" }))
        .await;
    let id = item["id"].as_str().unwrap();

    let (status, _) = app.delete_with_auth(&format!("/items/{id}"), &token).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (status, resp) = app.delete_with_auth(&format!("/items/{id}"), &token).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_error(&resp, "NOT_FOUND");
}

#[tokio::test]
async fn delete_item_other_user_404() {
    let app = TestApp::new().await;
    let token1 = app
        .setup_user_token("del_owner@example.com", TEST_PASSWORD)
        .await;
    let token2 = app
        .setup_user_token("del_other@example.com", TEST_PASSWORD)
        .await;

    let item = app
        .create_item(&token1, &serde_json::json!({ "name": "protected" }))
        .await;
    let id = item["id"].as_str().unwrap();

    let (status, resp) = app.delete_with_auth(&format!("/items/{id}"), &token2).await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_error(&resp, "NOT_FOUND");
}

#[tokio::test]
async fn delete_item_without_auth_401() {
    let app = TestApp::new().await;
    let token = app.setup_user_token(TEST_EMAIL, TEST_PASSWORD).await;

    let item = app
        .create_item(&token, &serde_json::json!({ "name": "x" }))
        .await;
    let id = item["id"].as_str().unwrap();

    let req = axum::http::Request::builder()
        .method("DELETE")
        .uri(format!("/items/{id}"))
        .body(axum::body::Body::empty())
        .unwrap();

    let resp = app.router.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

// ── Claim ───────────────────────────────────────────────────────────

#[tokio::test]
async fn claim_item_204() {
    let app = TestApp::new().await;
    let owner_token = app
        .setup_user_token("claim-owner@example.com", TEST_PASSWORD)
        .await;
    let claimer_token = app
        .setup_user_token("claim-claimer@example.com", TEST_PASSWORD)
        .await;

    let item = app
        .create_item(&owner_token, &serde_json::json!({ "name": "gift" }))
        .await;
    let id = item["id"].as_str().unwrap();

    let (status, _) = app
        .post_with_auth(&format!("/items/{id}/claim"), &claimer_token)
        .await;

    assert_eq!(status, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn claim_own_item_400() {
    let app = TestApp::new().await;
    let token = app
        .setup_user_token("claim-self@example.com", TEST_PASSWORD)
        .await;

    let item = app
        .create_item(&token, &serde_json::json!({ "name": "my item" }))
        .await;
    let id = item["id"].as_str().unwrap();

    let (status, resp) = app
        .post_with_auth(&format!("/items/{id}/claim"), &token)
        .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_error(&resp, "BAD_REQUEST");
}

#[tokio::test]
async fn claim_nonexistent_item_404() {
    let app = TestApp::new().await;
    let token = app
        .setup_user_token("claim-noexist@example.com", TEST_PASSWORD)
        .await;

    let fake_id = uuid::Uuid::new_v4();
    let (status, resp) = app
        .post_with_auth(&format!("/items/{fake_id}/claim"), &token)
        .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_error(&resp, "NOT_FOUND");
}

#[tokio::test]
async fn claim_already_claimed_409() {
    let app = TestApp::new().await;
    let owner_token = app
        .setup_user_token("claim-double-owner@example.com", TEST_PASSWORD)
        .await;
    let claimer1_token = app
        .setup_user_token("claim-double-1@example.com", TEST_PASSWORD)
        .await;
    let claimer2_token = app
        .setup_user_token("claim-double-2@example.com", TEST_PASSWORD)
        .await;

    let item = app
        .create_item(&owner_token, &serde_json::json!({ "name": "contested" }))
        .await;
    let id = item["id"].as_str().unwrap();

    // First claim succeeds
    let (status, _) = app
        .post_with_auth(&format!("/items/{id}/claim"), &claimer1_token)
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Second claim should be 409
    let (status, resp) = app
        .post_with_auth(&format!("/items/{id}/claim"), &claimer2_token)
        .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_error(&resp, "CONFLICT");
}

#[tokio::test]
async fn claim_purchased_item_400() {
    let app = TestApp::new().await;
    let owner_token = app
        .setup_user_token("claim-purch-owner@example.com", TEST_PASSWORD)
        .await;
    let claimer_token = app
        .setup_user_token("claim-purch-claimer@example.com", TEST_PASSWORD)
        .await;

    let item = app
        .create_item(&owner_token, &serde_json::json!({ "name": "bought" }))
        .await;
    let id = item["id"].as_str().unwrap();

    // Mark purchased
    app.put_json_with_auth(
        &format!("/items/{id}"),
        &serde_json::json!({ "status": "purchased" }),
        &owner_token,
    )
    .await;

    let (status, resp) = app
        .post_with_auth(&format!("/items/{id}/claim"), &claimer_token)
        .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_error(&resp, "BAD_REQUEST");
}

#[tokio::test]
async fn claim_deleted_item_404() {
    let app = TestApp::new().await;
    let owner_token = app
        .setup_user_token("claim-del-owner@example.com", TEST_PASSWORD)
        .await;
    let claimer_token = app
        .setup_user_token("claim-del-claimer@example.com", TEST_PASSWORD)
        .await;

    let item = app
        .create_item(&owner_token, &serde_json::json!({ "name": "deleted" }))
        .await;
    let id = item["id"].as_str().unwrap();

    app.delete_with_auth(&format!("/items/{id}"), &owner_token)
        .await;

    let (status, resp) = app
        .post_with_auth(&format!("/items/{id}/claim"), &claimer_token)
        .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_error(&resp, "NOT_FOUND");
}

#[tokio::test]
async fn claim_unauthenticated_401() {
    let app = TestApp::new().await;
    let token = app
        .setup_user_token("claim-unauth@example.com", TEST_PASSWORD)
        .await;

    let item = app
        .create_item(&token, &serde_json::json!({ "name": "x" }))
        .await;
    let id = item["id"].as_str().unwrap();

    let (status, _) = app.post_empty(&format!("/items/{id}/claim")).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

// ── Unclaim ─────────────────────────────────────────────────────────

#[tokio::test]
async fn unclaim_item_204() {
    let app = TestApp::new().await;
    let owner_token = app
        .setup_user_token("unclaim-owner@example.com", TEST_PASSWORD)
        .await;
    let claimer_token = app
        .setup_user_token("unclaim-claimer@example.com", TEST_PASSWORD)
        .await;

    let item = app
        .create_item(&owner_token, &serde_json::json!({ "name": "gift" }))
        .await;
    let id = item["id"].as_str().unwrap();

    app.post_with_auth(&format!("/items/{id}/claim"), &claimer_token)
        .await;

    let (status, _) = app
        .delete_with_auth(&format!("/items/{id}/claim"), &claimer_token)
        .await;

    assert_eq!(status, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn unclaim_not_claimer_401() {
    let app = TestApp::new().await;
    let owner_token = app
        .setup_user_token("unclaim-nc-owner@example.com", TEST_PASSWORD)
        .await;
    let claimer_token = app
        .setup_user_token("unclaim-nc-claimer@example.com", TEST_PASSWORD)
        .await;
    let other_token = app
        .setup_user_token("unclaim-nc-other@example.com", TEST_PASSWORD)
        .await;

    let item = app
        .create_item(&owner_token, &serde_json::json!({ "name": "reserved" }))
        .await;
    let id = item["id"].as_str().unwrap();

    app.post_with_auth(&format!("/items/{id}/claim"), &claimer_token)
        .await;

    let (status, resp) = app
        .delete_with_auth(&format!("/items/{id}/claim"), &other_token)
        .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_error(&resp, "UNAUTHORIZED");
}

#[tokio::test]
async fn unclaim_not_claimed_409() {
    let app = TestApp::new().await;
    let owner_token = app
        .setup_user_token("unclaim-notcl-owner@example.com", TEST_PASSWORD)
        .await;
    let other_token = app
        .setup_user_token("unclaim-notcl-other@example.com", TEST_PASSWORD)
        .await;

    let item = app
        .create_item(&owner_token, &serde_json::json!({ "name": "unclaimed" }))
        .await;
    let id = item["id"].as_str().unwrap();

    let (status, resp) = app
        .delete_with_auth(&format!("/items/{id}/claim"), &other_token)
        .await;

    assert_eq!(status, StatusCode::CONFLICT);
    assert_error(&resp, "CONFLICT");
}

#[tokio::test]
async fn unclaim_nonexistent_404() {
    let app = TestApp::new().await;
    let token = app
        .setup_user_token("unclaim-noexist@example.com", TEST_PASSWORD)
        .await;

    let fake_id = uuid::Uuid::new_v4();
    let (status, resp) = app
        .delete_with_auth(&format!("/items/{fake_id}/claim"), &token)
        .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_error(&resp, "NOT_FOUND");
}

#[tokio::test]
async fn unclaim_unauthenticated_401() {
    let app = TestApp::new().await;
    let token = app
        .setup_user_token("unclaim-unauth@example.com", TEST_PASSWORD)
        .await;

    let item = app
        .create_item(&token, &serde_json::json!({ "name": "x" }))
        .await;
    let id = item["id"].as_str().unwrap();

    let req = axum::http::Request::builder()
        .method("DELETE")
        .uri(format!("/items/{id}/claim"))
        .body(axum::body::Body::empty())
        .unwrap();

    let resp = app.router.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

// ── Claim integrity ─────────────────────────────────────────────────

#[tokio::test]
async fn claim_sets_is_claimed_true() {
    let app = TestApp::new().await;
    let owner_token = app
        .setup_user_token("claim-flag-owner@example.com", TEST_PASSWORD)
        .await;
    let claimer_token = app
        .setup_user_token("claim-flag-claimer@example.com", TEST_PASSWORD)
        .await;

    let item = app
        .create_item(&owner_token, &serde_json::json!({ "name": "flagtest" }))
        .await;
    let id = item["id"].as_str().unwrap();

    app.post_with_auth(&format!("/items/{id}/claim"), &claimer_token)
        .await;

    let (status, fetched) = app
        .get_with_auth(&format!("/items/{id}"), &owner_token)
        .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(fetched["is_claimed"], true);
}

#[tokio::test]
async fn unclaim_resets_is_claimed_false() {
    let app = TestApp::new().await;
    let owner_token = app
        .setup_user_token("unclaim-flag-owner@example.com", TEST_PASSWORD)
        .await;
    let claimer_token = app
        .setup_user_token("unclaim-flag-claimer@example.com", TEST_PASSWORD)
        .await;

    let item = app
        .create_item(&owner_token, &serde_json::json!({ "name": "flagtest2" }))
        .await;
    let id = item["id"].as_str().unwrap();

    app.post_with_auth(&format!("/items/{id}/claim"), &claimer_token)
        .await;
    app.delete_with_auth(&format!("/items/{id}/claim"), &claimer_token)
        .await;

    let (status, fetched) = app
        .get_with_auth(&format!("/items/{id}"), &owner_token)
        .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(fetched["is_claimed"], false);
}

#[tokio::test]
async fn claim_then_unclaim_then_reclaim() {
    let app = TestApp::new().await;
    let owner_token = app
        .setup_user_token("reclaim-owner@example.com", TEST_PASSWORD)
        .await;
    let claimer_token = app
        .setup_user_token("reclaim-claimer@example.com", TEST_PASSWORD)
        .await;

    let item = app
        .create_item(&owner_token, &serde_json::json!({ "name": "recycle" }))
        .await;
    let id = item["id"].as_str().unwrap();

    // Claim
    let (status, _) = app
        .post_with_auth(&format!("/items/{id}/claim"), &claimer_token)
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Unclaim
    let (status, _) = app
        .delete_with_auth(&format!("/items/{id}/claim"), &claimer_token)
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Reclaim
    let (status, _) = app
        .post_with_auth(&format!("/items/{id}/claim"), &claimer_token)
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (_, fetched) = app
        .get_with_auth(&format!("/items/{id}"), &owner_token)
        .await;
    assert_eq!(fetched["is_claimed"], true);
}

#[tokio::test]
async fn new_item_is_claimed_false() {
    let app = TestApp::new().await;
    let token = app
        .setup_user_token("newitem-claim@example.com", TEST_PASSWORD)
        .await;

    let item = app
        .create_item(&token, &serde_json::json!({ "name": "fresh" }))
        .await;

    assert_eq!(item["is_claimed"], false);
}

#[tokio::test]
async fn claim_same_claimer_twice_409() {
    let app = TestApp::new().await;
    let owner_token = app
        .setup_user_token("claim-idem-owner@example.com", TEST_PASSWORD)
        .await;
    let claimer_token = app
        .setup_user_token("claim-idem-claimer@example.com", TEST_PASSWORD)
        .await;

    let item = app
        .create_item(&owner_token, &serde_json::json!({ "name": "idem" }))
        .await;
    let id = item["id"].as_str().unwrap();

    // First claim
    let (status, _) = app
        .post_with_auth(&format!("/items/{id}/claim"), &claimer_token)
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Same claimer again → 409
    let (status, resp) = app
        .post_with_auth(&format!("/items/{id}/claim"), &claimer_token)
        .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_error(&resp, "CONFLICT");
}

#[tokio::test]
async fn claim_then_owner_deletes_item() {
    let app = TestApp::new().await;
    let owner_token = app
        .setup_user_token("claim-then-del-owner@example.com", TEST_PASSWORD)
        .await;
    let claimer_token = app
        .setup_user_token("claim-then-del-claimer@example.com", TEST_PASSWORD)
        .await;

    let item = app
        .create_item(&owner_token, &serde_json::json!({ "name": "doomed" }))
        .await;
    let id = item["id"].as_str().unwrap();

    // Claim
    let (status, _) = app
        .post_with_auth(&format!("/items/{id}/claim"), &claimer_token)
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Owner deletes
    let (status, _) = app
        .delete_with_auth(&format!("/items/{id}"), &owner_token)
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Unclaim should now 404
    let (status, resp) = app
        .delete_with_auth(&format!("/items/{id}/claim"), &claimer_token)
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_error(&resp, "NOT_FOUND");
}

#[tokio::test]
async fn is_claimed_visible_in_list_endpoint() {
    let app = TestApp::new().await;
    let owner_token = app
        .setup_user_token("claim-list-owner@example.com", TEST_PASSWORD)
        .await;
    let claimer_token = app
        .setup_user_token("claim-list-claimer@example.com", TEST_PASSWORD)
        .await;

    let item = app
        .create_item(&owner_token, &serde_json::json!({ "name": "listcheck" }))
        .await;
    let id = item["id"].as_str().unwrap();

    // Before claim — list shows is_claimed: false
    let (_, body) = app.get_with_auth("/items", &owner_token).await;
    let items = body["data"].as_array().unwrap();
    assert_eq!(items[0]["is_claimed"], false);

    // Claim
    app.post_with_auth(&format!("/items/{id}/claim"), &claimer_token)
        .await;

    // After claim — list shows is_claimed: true
    let (_, body) = app.get_with_auth("/items", &owner_token).await;
    let items = body["data"].as_array().unwrap();
    assert_eq!(items[0]["is_claimed"], true);
}

// ── Shared circles ──────────────────────────────────────────────────

#[tokio::test]
async fn get_item_includes_shared_circles() {
    let app = TestApp::new().await;
    let token = app
        .setup_user_token("shared-circles@example.com", TEST_PASSWORD)
        .await;

    // Create item
    let item = app
        .create_item(&token, &serde_json::json!({ "name": "shared item" }))
        .await;
    let item_id = item["id"].as_str().unwrap();

    // Create circle
    let (status, circle) = app
        .post_json_with_auth(
            "/circles",
            &serde_json::json!({ "name": "Famille" }),
            &token,
        )
        .await;
    assert_eq!(status, StatusCode::CREATED);
    let circle_id = circle["id"].as_str().unwrap();

    // Share item to circle
    let (status, _) = app
        .post_json_with_auth(
            &format!("/circles/{circle_id}/items"),
            &serde_json::json!({ "item_id": item_id }),
            &token,
        )
        .await;
    assert!(
        status.is_success(),
        "share item should succeed, got {status}"
    );

    // Get item — should include shared_circles
    let (status, body) = app
        .get_with_auth(&format!("/items/{item_id}"), &token)
        .await;
    assert_eq!(status, StatusCode::OK);

    let shared_circles = body["shared_circles"]
        .as_array()
        .expect("shared_circles should be an array");
    assert!(
        !shared_circles.is_empty(),
        "shared_circles should have at least one entry"
    );
    assert_eq!(shared_circles[0]["id"], circle_id);
    assert_eq!(shared_circles[0]["name"], "Famille");
}

#[tokio::test]
async fn list_items_includes_shared_circles() {
    let app = TestApp::new().await;
    let token = app
        .setup_user_token("shared-circles-list@example.com", TEST_PASSWORD)
        .await;

    // Create item
    let item = app
        .create_item(&token, &serde_json::json!({ "name": "listed item" }))
        .await;
    let item_id = item["id"].as_str().unwrap();

    // Create circle
    let (status, circle) = app
        .post_json_with_auth("/circles", &serde_json::json!({ "name": "Amis" }), &token)
        .await;
    assert_eq!(status, StatusCode::CREATED);
    let circle_id = circle["id"].as_str().unwrap();

    // Share item to circle
    let (status, _) = app
        .post_json_with_auth(
            &format!("/circles/{circle_id}/items"),
            &serde_json::json!({ "item_id": item_id }),
            &token,
        )
        .await;
    assert!(
        status.is_success(),
        "share item should succeed, got {status}"
    );

    // List items — first item should have non-empty shared_circles
    let (status, body) = app.get_with_auth("/items", &token).await;
    assert_eq!(status, StatusCode::OK);

    let items = body["data"].as_array().unwrap();
    assert!(!items.is_empty(), "should have at least one item");

    let first = &items[0];
    let shared_circles = first["shared_circles"]
        .as_array()
        .expect("shared_circles should be an array in list response");
    assert!(
        !shared_circles.is_empty(),
        "shared_circles should be non-empty after sharing"
    );
}

#[tokio::test]
async fn unshare_removes_from_shared_circles() {
    let app = TestApp::new().await;
    let token = app
        .setup_user_token("shared-circles-unshr@example.com", TEST_PASSWORD)
        .await;

    // Create item
    let item = app
        .create_item(&token, &serde_json::json!({ "name": "unshare item" }))
        .await;
    let item_id = item["id"].as_str().unwrap();

    // Create circle
    let (status, circle) = app
        .post_json_with_auth(
            "/circles",
            &serde_json::json!({ "name": "Collègues" }),
            &token,
        )
        .await;
    assert_eq!(status, StatusCode::CREATED);
    let circle_id = circle["id"].as_str().unwrap();

    // Share item to circle
    let (status, _) = app
        .post_json_with_auth(
            &format!("/circles/{circle_id}/items"),
            &serde_json::json!({ "item_id": item_id }),
            &token,
        )
        .await;
    assert!(
        status.is_success(),
        "share item should succeed, got {status}"
    );

    // Verify it is shared
    let (_, body) = app
        .get_with_auth(&format!("/items/{item_id}"), &token)
        .await;
    let shared = body["shared_circles"].as_array().unwrap();
    assert!(!shared.is_empty(), "should be shared before unshare");

    // Unshare: DELETE /circles/{circle_id}/items/{item_id}
    let (status, _) = app
        .delete_with_auth(&format!("/circles/{circle_id}/items/{item_id}"), &token)
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Get item again — shared_circles should be empty
    let (status, body) = app
        .get_with_auth(&format!("/items/{item_id}"), &token)
        .await;
    assert_eq!(status, StatusCode::OK);

    let shared_circles = body["shared_circles"]
        .as_array()
        .expect("shared_circles should be an array");
    assert!(
        shared_circles.is_empty(),
        "shared_circles should be empty after unshare"
    );
}

// ── Cache consistency ───────────────────────────────────────────────

#[tokio::test]
async fn list_items_consistent_after_mutations() {
    let app = TestApp::new().await;
    let token = app.setup_user_token(TEST_EMAIL, TEST_PASSWORD).await;

    // List (cache miss)
    let (_, body) = app.get_with_auth("/items", &token).await;
    assert_eq!(body["pagination"]["total"], 0);

    // Create
    let item = app
        .create_item(&token, &serde_json::json!({ "name": "cached" }))
        .await;
    let id = item["id"].as_str().unwrap();

    // List should reflect creation
    let (_, body) = app.get_with_auth("/items", &token).await;
    assert_eq!(body["pagination"]["total"], 1);

    // Update
    app.put_json_with_auth(
        &format!("/items/{id}"),
        &serde_json::json!({ "name": "updated" }),
        &token,
    )
    .await;

    let (_, body) = app.get_with_auth("/items", &token).await;
    assert_eq!(body["data"][0]["name"], "updated");

    // Delete
    app.delete_with_auth(&format!("/items/{id}"), &token).await;

    let (_, body) = app.get_with_auth("/items", &token).await;
    assert_eq!(body["pagination"]["total"], 0);
}

// ═══════════════════════════════════════════════════════════════════════
// Secondary sort: same priority ordered by created_at DESC
// ═══════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn sort_priority_secondary_by_created_at() {
    let app = TestApp::new().await;
    let token = app.setup_user_token(TEST_EMAIL, TEST_PASSWORD).await;

    // Create 3 items with same priority, small delays for distinct created_at
    app.create_item(
        &token,
        &serde_json::json!({ "name": "First", "priority": 2 }),
    )
    .await;
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    app.create_item(
        &token,
        &serde_json::json!({ "name": "Second", "priority": 2 }),
    )
    .await;
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    app.create_item(
        &token,
        &serde_json::json!({ "name": "Third", "priority": 2 }),
    )
    .await;

    // Sort by priority DESC — within same priority, newest first (created_at DESC)
    let (_, body) = app
        .get_with_auth("/items?sort=priority&order=desc", &token)
        .await;
    let items = body["data"].as_array().unwrap();
    assert_eq!(
        items[0]["name"], "Third",
        "newest item first within same priority"
    );
    assert_eq!(items[1]["name"], "Second");
    assert_eq!(
        items[2]["name"], "First",
        "oldest item last within same priority"
    );
}

#[tokio::test]
async fn sort_priority_asc_secondary_by_created_at() {
    let app = TestApp::new().await;
    let token = app.setup_user_token(TEST_EMAIL, TEST_PASSWORD).await;

    // Priority 1 items (created in order: A then B)
    app.create_item(
        &token,
        &serde_json::json!({ "name": "LowA", "priority": 1 }),
    )
    .await;
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    app.create_item(
        &token,
        &serde_json::json!({ "name": "LowB", "priority": 1 }),
    )
    .await;
    // Priority 3 item
    app.create_item(
        &token,
        &serde_json::json!({ "name": "High", "priority": 3 }),
    )
    .await;

    // Sort by priority ASC — low first, within same priority newest first
    let (_, body) = app
        .get_with_auth("/items?sort=priority&order=asc", &token)
        .await;
    let items = body["data"].as_array().unwrap();
    assert_eq!(items[0]["name"], "LowB", "newest low-priority first");
    assert_eq!(items[1]["name"], "LowA", "oldest low-priority second");
    assert_eq!(items[2]["name"], "High", "high priority last in ASC");
}

#[tokio::test]
async fn sort_name_secondary_by_created_at() {
    let app = TestApp::new().await;
    let token = app.setup_user_token(TEST_EMAIL, TEST_PASSWORD).await;

    // Two items with same name
    app.create_item(&token, &serde_json::json!({ "name": "Same" }))
        .await;
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    app.create_item(&token, &serde_json::json!({ "name": "Same" }))
        .await;

    // Sort by name — same names should be ordered by created_at DESC
    let (_, body) = app
        .get_with_auth("/items?sort=name&order=asc", &token)
        .await;
    let items = body["data"].as_array().unwrap();
    assert_eq!(items.len(), 2);
    // Newer one should come first within same name
    let first_created = items[0]["created_at"].as_str().unwrap();
    let second_created = items[1]["created_at"].as_str().unwrap();
    assert!(
        first_created > second_created,
        "newer item first within same name"
    );
}

#[tokio::test]
async fn sort_created_at_no_secondary_sort() {
    let app = TestApp::new().await;
    let token = app.setup_user_token(TEST_EMAIL, TEST_PASSWORD).await;

    app.create_item(&token, &serde_json::json!({ "name": "Old" }))
        .await;
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    app.create_item(&token, &serde_json::json!({ "name": "New" }))
        .await;

    // Sort by created_at DESC — should NOT add secondary sort
    let (_, body) = app
        .get_with_auth("/items?sort=created_at&order=desc", &token)
        .await;
    let items = body["data"].as_array().unwrap();
    assert_eq!(items[0]["name"], "New");
    assert_eq!(items[1]["name"], "Old");
}

// ═══════════════════════════════════════════════════════════════════════
// shared_circles includes rule-based shares (all/categories)
// ═══════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn shared_circles_includes_rule_all() {
    let app = TestApp::new().await;
    let token = app
        .setup_user_token("rule-all@test.com", TEST_PASSWORD)
        .await;

    // Create an item
    let item = app
        .create_item(&token, &serde_json::json!({ "name": "RuleAllItem" }))
        .await;
    let item_id = item["id"].as_str().unwrap();

    // Create a circle
    let (_, circle) = app
        .post_json_with_auth(
            "/circles",
            &serde_json::json!({ "name": "AllCircle" }),
            &token,
        )
        .await;
    let circle_id = circle["id"].as_str().unwrap();

    // Set share rule to "all" (no circle_items record created)
    let (status, _) = app
        .put_json_with_auth(
            &format!("/circles/{circle_id}/share-rule"),
            &serde_json::json!({ "share_mode": "all" }),
            &token,
        )
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Get item — shared_circles should include the circle
    let (_, body) = app
        .get_with_auth(&format!("/items/{item_id}"), &token)
        .await;
    let circles = body["shared_circles"].as_array().unwrap();
    assert!(
        circles.iter().any(|c| c["id"].as_str() == Some(circle_id)),
        "shared_circles must include circle with rule 'all'"
    );
}

#[tokio::test]
async fn shared_circles_includes_rule_categories() {
    let app = TestApp::new().await;
    let token = app
        .setup_user_token("rule-cat@test.com", TEST_PASSWORD)
        .await;

    // Get a category
    let (_, cats) = app.get_with_auth("/categories", &token).await;
    let cat_id = cats.as_array().unwrap()[0]["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Create item in that category
    let item = app
        .create_item(
            &token,
            &serde_json::json!({ "name": "CatItem", "category_id": cat_id }),
        )
        .await;
    let item_id = item["id"].as_str().unwrap();

    // Create circle + set category rule
    let (_, circle) = app
        .post_json_with_auth(
            "/circles",
            &serde_json::json!({ "name": "CatCircle" }),
            &token,
        )
        .await;
    let circle_id = circle["id"].as_str().unwrap();

    let (status, _) = app
        .put_json_with_auth(
            &format!("/circles/{circle_id}/share-rule"),
            &serde_json::json!({ "share_mode": "categories", "category_ids": [cat_id] }),
            &token,
        )
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Item should show circle in shared_circles
    let (_, body) = app
        .get_with_auth(&format!("/items/{item_id}"), &token)
        .await;
    let circles = body["shared_circles"].as_array().unwrap();
    assert!(
        circles.iter().any(|c| c["id"].as_str() == Some(circle_id)),
        "shared_circles must include circle with matching category rule"
    );
}

#[tokio::test]
async fn shared_circles_excludes_non_matching_category() {
    let app = TestApp::new().await;
    let token = app
        .setup_user_token("rule-nocat@test.com", TEST_PASSWORD)
        .await;

    // Create item WITHOUT category
    let item = app
        .create_item(&token, &serde_json::json!({ "name": "NoCatItem" }))
        .await;
    let item_id = item["id"].as_str().unwrap();

    // Get a category (item doesn't have it)
    let (_, cats) = app.get_with_auth("/categories", &token).await;
    let cat_id = cats.as_array().unwrap()[0]["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Create circle with category rule
    let (_, circle) = app
        .post_json_with_auth(
            "/circles",
            &serde_json::json!({ "name": "MismatchCircle" }),
            &token,
        )
        .await;
    let circle_id = circle["id"].as_str().unwrap();

    let (status, _) = app
        .put_json_with_auth(
            &format!("/circles/{circle_id}/share-rule"),
            &serde_json::json!({ "share_mode": "categories", "category_ids": [cat_id] }),
            &token,
        )
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Item should NOT show this circle (category doesn't match)
    let (_, body) = app
        .get_with_auth(&format!("/items/{item_id}"), &token)
        .await;
    let circles = body["shared_circles"].as_array().unwrap();
    assert!(
        !circles.iter().any(|c| c["id"].as_str() == Some(circle_id)),
        "shared_circles must NOT include circle when category doesn't match"
    );
}

#[tokio::test]
async fn shared_circles_private_item_excluded_from_rules() {
    let app = TestApp::new().await;
    let token = app
        .setup_user_token("rule-priv@test.com", TEST_PASSWORD)
        .await;

    // Create a private item
    let item = app
        .create_item(
            &token,
            &serde_json::json!({ "name": "PrivItem", "is_private": true }),
        )
        .await;
    let item_id = item["id"].as_str().unwrap();

    // Create circle with rule "all"
    let (_, circle) = app
        .post_json_with_auth(
            "/circles",
            &serde_json::json!({ "name": "PrivCircle" }),
            &token,
        )
        .await;
    let circle_id = circle["id"].as_str().unwrap();

    let (status, _) = app
        .put_json_with_auth(
            &format!("/circles/{circle_id}/share-rule"),
            &serde_json::json!({ "share_mode": "all" }),
            &token,
        )
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Private item should NOT show circle
    let (_, body) = app
        .get_with_auth(&format!("/items/{item_id}"), &token)
        .await;
    let circles = body["shared_circles"].as_array().unwrap();
    assert!(
        !circles.iter().any(|c| c["id"].as_str() == Some(circle_id)),
        "private items must NOT show rule-based circles"
    );
}

#[tokio::test]
async fn shared_circles_no_duplicate_when_both_rule_and_selection() {
    let app = TestApp::new().await;
    let token = app
        .setup_user_token("rule-dup@test.com", TEST_PASSWORD)
        .await;

    let item = app
        .create_item(&token, &serde_json::json!({ "name": "DupItem" }))
        .await;
    let item_id = item["id"].as_str().unwrap();

    // Create circle
    let (_, circle) = app
        .post_json_with_auth(
            "/circles",
            &serde_json::json!({ "name": "DupCircle" }),
            &token,
        )
        .await;
    let circle_id = circle["id"].as_str().unwrap();

    // Share via circle_items (selection)
    app.post_json_with_auth(
        &format!("/circles/{circle_id}/items"),
        &serde_json::json!({ "item_id": item_id }),
        &token,
    )
    .await;

    // Also set rule "all"
    app.put_json_with_auth(
        &format!("/circles/{circle_id}/share-rule"),
        &serde_json::json!({ "share_mode": "all" }),
        &token,
    )
    .await;

    // Should appear only ONCE in shared_circles
    let (_, body) = app
        .get_with_auth(&format!("/items/{item_id}"), &token)
        .await;
    let circles = body["shared_circles"].as_array().unwrap();
    let count = circles
        .iter()
        .filter(|c| c["id"].as_str() == Some(circle_id))
        .count();
    assert_eq!(
        count, 1,
        "circle must appear only once even with both rule and selection"
    );
}

// ── Link URL Validation ────────────────────────────────────────────

#[tokio::test]
async fn create_item_with_valid_links() {
    let app = TestApp::new().await;
    let token = app
        .setup_user_token("links-valid@example.com", TEST_PASSWORD)
        .await;

    let body = serde_json::json!({
        "name": "Gadget",
        "links": ["https://example.com/product", "https://shop.example.org/item?id=42"]
    });
    let (status, item) = app.post_json_with_auth("/items", &body, &token).await;

    assert_eq!(status, StatusCode::CREATED);
    let links = item["links"].as_array().unwrap();
    assert_eq!(links.len(), 2);
    assert_eq!(links[0], "https://example.com/product");
    assert_eq!(links[1], "https://shop.example.org/item?id=42");
}

#[tokio::test]
async fn create_item_with_invalid_link_rejected() {
    let app = TestApp::new().await;
    let token = app
        .setup_user_token("links-invalid@example.com", TEST_PASSWORD)
        .await;

    let body = serde_json::json!({
        "name": "Gadget",
        "links": ["not-a-url"]
    });
    let (status, resp) = app.post_json_with_auth("/items", &body, &token).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_error(&resp, "BAD_REQUEST");
}

#[tokio::test]
async fn create_item_with_mixed_valid_invalid_rejected() {
    let app = TestApp::new().await;
    let token = app
        .setup_user_token("links-mixed@example.com", TEST_PASSWORD)
        .await;

    let body = serde_json::json!({
        "name": "Gadget",
        "links": ["https://valid.com/page", "just-text"]
    });
    let (status, resp) = app.post_json_with_auth("/items", &body, &token).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_error(&resp, "BAD_REQUEST");
}

#[tokio::test]
async fn update_item_with_invalid_link_rejected() {
    let app = TestApp::new().await;
    let token = app
        .setup_user_token("links-update@example.com", TEST_PASSWORD)
        .await;

    let item = app
        .create_item(&token, &serde_json::json!({ "name": "Original" }))
        .await;
    let id = item["id"].as_str().unwrap();

    let (status, resp) = app
        .put_json_with_auth(
            &format!("/items/{id}"),
            &serde_json::json!({ "links": ["ftp://bad-scheme.com"] }),
            &token,
        )
        .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_error(&resp, "BAD_REQUEST");
}

#[tokio::test]
async fn create_item_with_empty_links_ok() {
    let app = TestApp::new().await;
    let token = app
        .setup_user_token("links-empty@example.com", TEST_PASSWORD)
        .await;

    // Empty array
    let body = serde_json::json!({
        "name": "No links",
        "links": []
    });
    let (status, _) = app.post_json_with_auth("/items", &body, &token).await;
    assert_eq!(status, StatusCode::CREATED);

    // Array with empty strings (should be accepted — filtered by frontend)
    let body2 = serde_json::json!({
        "name": "Empty strings",
        "links": ["", ""]
    });
    let (status2, _) = app.post_json_with_auth("/items", &body2, &token).await;
    assert_eq!(status2, StatusCode::CREATED);
}

#[tokio::test]
async fn create_item_with_http_link_ok() {
    let app = TestApp::new().await;
    let token = app
        .setup_user_token("links-http@example.com", TEST_PASSWORD)
        .await;

    let body = serde_json::json!({
        "name": "HTTP item",
        "links": ["http://example.com/page"]
    });
    let (status, item) = app.post_json_with_auth("/items", &body, &token).await;

    assert_eq!(status, StatusCode::CREATED);
    let links = item["links"].as_array().unwrap();
    assert_eq!(links.len(), 1);
    assert_eq!(links[0], "http://example.com/page");
}
