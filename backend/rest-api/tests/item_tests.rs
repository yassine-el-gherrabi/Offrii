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
}

#[tokio::test]
async fn create_item_full_201() {
    let app = TestApp::new().await;
    let token = app.setup_user_token(TEST_EMAIL, TEST_PASSWORD).await;

    // Get a category for full test
    let user_id: uuid::Uuid =
        sqlx::query_scalar::<_, uuid::Uuid>("SELECT id FROM users WHERE email = $1")
            .bind(TEST_EMAIL)
            .fetch_one(&app.db)
            .await
            .unwrap();
    let cat_id: uuid::Uuid =
        sqlx::query_scalar("SELECT id FROM categories WHERE user_id = $1 LIMIT 1")
            .bind(user_id)
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
async fn create_item_other_users_category_400() {
    let app = TestApp::new().await;
    let _token1 = app
        .setup_user_token("catowner@example.com", TEST_PASSWORD)
        .await;
    let token2 = app
        .setup_user_token("catthief@example.com", TEST_PASSWORD)
        .await;

    // Get user1's category
    let user1_id: uuid::Uuid =
        sqlx::query_scalar::<_, uuid::Uuid>("SELECT id FROM users WHERE email = $1")
            .bind("catowner@example.com")
            .fetch_one(&app.db)
            .await
            .unwrap();
    let user1_cat: uuid::Uuid =
        sqlx::query_scalar("SELECT id FROM categories WHERE user_id = $1 LIMIT 1")
            .bind(user1_id)
            .fetch_one(&app.db)
            .await
            .unwrap();

    // User2 tries to use user1's category
    let body = serde_json::json!({
        "name": "test",
        "category_id": user1_cat.to_string(),
    });
    let (status, resp) = app.post_json_with_auth("/items", &body, &token2).await;

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
    assert_eq!(body["items"].as_array().unwrap().len(), 2);
    assert_eq!(body["total"], 2);
    assert_eq!(body["page"], 1);
    assert_eq!(body["per_page"], 50);
}

#[tokio::test]
async fn list_items_pagination() {
    let app = TestApp::new().await;
    let token = app.setup_user_token(TEST_EMAIL, TEST_PASSWORD).await;

    for i in 0..5 {
        app.create_item(&token, &serde_json::json!({ "name": format!("item{i}") }))
            .await;
    }

    let (status, body) = app.get_with_auth("/items?page=2&per_page=2", &token).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["items"].as_array().unwrap().len(), 2);
    assert_eq!(body["total"], 5);
    assert_eq!(body["page"], 2);
    assert_eq!(body["per_page"], 2);
}

#[tokio::test]
async fn list_items_page_beyond_total_returns_empty() {
    let app = TestApp::new().await;
    let token = app.setup_user_token(TEST_EMAIL, TEST_PASSWORD).await;

    app.create_item(&token, &serde_json::json!({ "name": "x" }))
        .await;

    let (status, body) = app
        .get_with_auth("/items?page=100&per_page=50", &token)
        .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["items"].as_array().unwrap().len(), 0);
    assert_eq!(body["total"], 1);
}

#[tokio::test]
async fn list_items_per_page_clamped() {
    let app = TestApp::new().await;
    let token = app.setup_user_token(TEST_EMAIL, TEST_PASSWORD).await;

    // per_page=0 should clamp to 1
    let (status, body) = app.get_with_auth("/items?per_page=0", &token).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["per_page"], 1);

    // per_page=200 should clamp to 100
    let (status, body) = app.get_with_auth("/items?per_page=200", &token).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["per_page"], 100);
}

#[tokio::test]
async fn list_items_page_zero_clamps_to_1() {
    let app = TestApp::new().await;
    let token = app.setup_user_token(TEST_EMAIL, TEST_PASSWORD).await;

    let (status, body) = app.get_with_auth("/items?page=0", &token).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["page"], 1);
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
    assert_eq!(body["items"].as_array().unwrap().len(), 1);
    assert_eq!(body["items"][0]["name"], "bought");

    // Filter active
    let (_, body) = app.get_with_auth("/items?status=active", &token).await;
    assert_eq!(body["items"].as_array().unwrap().len(), 1);
    assert_eq!(body["items"][0]["name"], "active_one");
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

    let user_id: uuid::Uuid =
        sqlx::query_scalar::<_, uuid::Uuid>("SELECT id FROM users WHERE email = $1")
            .bind(TEST_EMAIL)
            .fetch_one(&app.db)
            .await
            .unwrap();

    let cat_id: uuid::Uuid =
        sqlx::query_scalar("SELECT id FROM categories WHERE user_id = $1 LIMIT 1")
            .bind(user_id)
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
    assert_eq!(body["items"].as_array().unwrap().len(), 1);
    assert_eq!(body["items"][0]["name"], "with_cat");
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

    let items = body["items"].as_array().unwrap();
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

    let items = body["items"].as_array().unwrap();
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
    assert_eq!(body["items"].as_array().unwrap().len(), 0);
    assert_eq!(body["total"], 0);
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

    assert_eq!(body_a["total"], 1);
    assert_eq!(body_a["items"][0]["name"], "a_item");
    assert_eq!(body_b["total"], 1);
    assert_eq!(body_b["items"][0]["name"], "b_item");
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

    // Small delay to ensure updated_at changes
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

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
    assert_eq!(body["items"].as_array().unwrap().len(), 0);

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

// ── Cache consistency ───────────────────────────────────────────────

#[tokio::test]
async fn list_items_consistent_after_mutations() {
    let app = TestApp::new().await;
    let token = app.setup_user_token(TEST_EMAIL, TEST_PASSWORD).await;

    // List (cache miss)
    let (_, body) = app.get_with_auth("/items", &token).await;
    assert_eq!(body["total"], 0);

    // Create
    let item = app
        .create_item(&token, &serde_json::json!({ "name": "cached" }))
        .await;
    let id = item["id"].as_str().unwrap();

    // List should reflect creation
    let (_, body) = app.get_with_auth("/items", &token).await;
    assert_eq!(body["total"], 1);

    // Update
    app.put_json_with_auth(
        &format!("/items/{id}"),
        &serde_json::json!({ "name": "updated" }),
        &token,
    )
    .await;

    let (_, body) = app.get_with_auth("/items", &token).await;
    assert_eq!(body["items"][0]["name"], "updated");

    // Delete
    app.delete_with_auth(&format!("/items/{id}"), &token).await;

    let (_, body) = app.get_with_auth("/items", &token).await;
    assert_eq!(body["total"], 0);
}
