mod common;

use axum::http::StatusCode;
use common::{TEST_PASSWORD, TestApp, assert_error};

const TEST_EMAIL: &str = "categories@example.com";

// ── List ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn list_categories_returns_6_defaults() {
    let app = TestApp::new().await;
    let token = app.setup_user_token(TEST_EMAIL, TEST_PASSWORD).await;

    let (status, body) = app.get_with_auth("/categories", &token).await;

    assert_eq!(status, StatusCode::OK);
    let cats = body.as_array().expect("response should be an array");
    assert_eq!(cats.len(), 6);
    for cat in cats {
        assert_eq!(cat["is_default"], true);
    }
}

#[tokio::test]
async fn list_categories_without_auth_401() {
    let app = TestApp::new().await;

    let (status, body) = app.get_no_auth("/categories").await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_error(&body, "UNAUTHORIZED");
}

#[tokio::test]
async fn list_categories_isolation_between_users() {
    let app = TestApp::new().await;
    let token_a = app
        .setup_user_token("cat-isolation-a@example.com", TEST_PASSWORD)
        .await;
    let token_b = app
        .setup_user_token("cat-isolation-b@example.com", TEST_PASSWORD)
        .await;

    // User A creates a custom category
    app.create_category(&token_a, &serde_json::json!({ "name": "Custom A" }))
        .await;

    // User A sees 7 categories
    let (_, body_a) = app.get_with_auth("/categories", &token_a).await;
    assert_eq!(body_a.as_array().unwrap().len(), 7);

    // User B still sees only 6
    let (_, body_b) = app.get_with_auth("/categories", &token_b).await;
    assert_eq!(body_b.as_array().unwrap().len(), 6);
}

#[tokio::test]
async fn list_categories_sorted_by_position() {
    let app = TestApp::new().await;
    let token = app
        .setup_user_token("cat-sort@example.com", TEST_PASSWORD)
        .await;

    let (_, body) = app.get_with_auth("/categories", &token).await;
    let cats = body.as_array().unwrap();

    let positions: Vec<i64> = cats
        .iter()
        .map(|c| c["position"].as_i64().unwrap())
        .collect();
    let mut sorted = positions.clone();
    sorted.sort();
    assert_eq!(positions, sorted);
}

// ── Create ───────────────────────────────────────────────────────────

#[tokio::test]
async fn create_category_201() {
    let app = TestApp::new().await;
    let token = app
        .setup_user_token("cat-create@example.com", TEST_PASSWORD)
        .await;

    let cat = app
        .create_category(
            &token,
            &serde_json::json!({ "name": "Voyages", "icon": "plane" }),
        )
        .await;

    assert_eq!(cat["name"], "Voyages");
    assert_eq!(cat["icon"], "plane");
    assert_eq!(cat["is_default"], false);
    assert_eq!(cat["position"], 7); // 6 defaults + 1
    assert!(cat["id"].is_string());
    assert!(cat["created_at"].is_string());
}

#[tokio::test]
async fn create_category_name_only_201() {
    let app = TestApp::new().await;
    let token = app
        .setup_user_token("cat-nameonly@example.com", TEST_PASSWORD)
        .await;

    let cat = app
        .create_category(&token, &serde_json::json!({ "name": "Minimal" }))
        .await;

    assert_eq!(cat["name"], "Minimal");
    assert!(cat["icon"].is_null());
}

#[tokio::test]
async fn create_category_duplicate_name_409() {
    let app = TestApp::new().await;
    let token = app
        .setup_user_token("cat-dup@example.com", TEST_PASSWORD)
        .await;

    app.create_category(&token, &serde_json::json!({ "name": "Unique" }))
        .await;

    let (status, body) = app
        .post_json_with_auth(
            "/categories",
            &serde_json::json!({ "name": "Unique" }),
            &token,
        )
        .await;

    assert_eq!(status, StatusCode::CONFLICT);
    assert_error(&body, "CONFLICT");
}

#[tokio::test]
async fn create_category_same_name_different_users_201() {
    let app = TestApp::new().await;
    let token_a = app
        .setup_user_token("cat-same-a@example.com", TEST_PASSWORD)
        .await;
    let token_b = app
        .setup_user_token("cat-same-b@example.com", TEST_PASSWORD)
        .await;

    app.create_category(&token_a, &serde_json::json!({ "name": "Shared Name" }))
        .await;
    // Should succeed for user B
    app.create_category(&token_b, &serde_json::json!({ "name": "Shared Name" }))
        .await;
}

#[tokio::test]
async fn create_category_empty_name_400() {
    let app = TestApp::new().await;
    let token = app
        .setup_user_token("cat-empty@example.com", TEST_PASSWORD)
        .await;

    let (status, body) = app
        .post_json_with_auth("/categories", &serde_json::json!({ "name": "" }), &token)
        .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_error(&body, "BAD_REQUEST");
}

#[tokio::test]
async fn create_category_name_too_long_400() {
    let app = TestApp::new().await;
    let token = app
        .setup_user_token("cat-long@example.com", TEST_PASSWORD)
        .await;

    let long_name = "a".repeat(101);
    let (status, body) = app
        .post_json_with_auth(
            "/categories",
            &serde_json::json!({ "name": long_name }),
            &token,
        )
        .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_error(&body, "BAD_REQUEST");
}

#[tokio::test]
async fn create_category_without_auth_401() {
    let app = TestApp::new().await;

    let (status, body) = app
        .post_json_with_auth(
            "/categories",
            &serde_json::json!({ "name": "test" }),
            "invalid-token",
        )
        .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_error(&body, "UNAUTHORIZED");
}

#[tokio::test]
async fn create_category_duplicate_name_with_default_409() {
    let app = TestApp::new().await;
    let token = app
        .setup_user_token("cat-dup-default@example.com", TEST_PASSWORD)
        .await;

    // "Tech" is one of the 6 defaults copied on registration
    let (status, body) = app
        .post_json_with_auth(
            "/categories",
            &serde_json::json!({ "name": "Tech" }),
            &token,
        )
        .await;

    assert_eq!(status, StatusCode::CONFLICT);
    assert_error(&body, "CONFLICT");
}

#[tokio::test]
async fn create_category_icon_too_long_400() {
    let app = TestApp::new().await;
    let token = app
        .setup_user_token("cat-icon-long@example.com", TEST_PASSWORD)
        .await;

    let long_icon = "a".repeat(51);
    let (status, body) = app
        .post_json_with_auth(
            "/categories",
            &serde_json::json!({ "name": "Valid", "icon": long_icon }),
            &token,
        )
        .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_error(&body, "BAD_REQUEST");
}

// ── Update ───────────────────────────────────────────────────────────

#[tokio::test]
async fn update_category_name_200() {
    let app = TestApp::new().await;
    let token = app
        .setup_user_token("cat-upd-name@example.com", TEST_PASSWORD)
        .await;

    let cat = app
        .create_category(&token, &serde_json::json!({ "name": "Original" }))
        .await;
    let id = cat["id"].as_str().unwrap();

    let (status, body) = app
        .put_json_with_auth(
            &format!("/categories/{id}"),
            &serde_json::json!({ "name": "Renamed" }),
            &token,
        )
        .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["name"], "Renamed");
    assert_eq!(body["id"], id);
}

#[tokio::test]
async fn update_category_icon_200() {
    let app = TestApp::new().await;
    let token = app
        .setup_user_token("cat-upd-icon@example.com", TEST_PASSWORD)
        .await;

    let cat = app
        .create_category(
            &token,
            &serde_json::json!({ "name": "WithIcon", "icon": "star" }),
        )
        .await;
    let id = cat["id"].as_str().unwrap();

    let (status, body) = app
        .put_json_with_auth(
            &format!("/categories/{id}"),
            &serde_json::json!({ "icon": "heart" }),
            &token,
        )
        .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["icon"], "heart");
}

#[tokio::test]
async fn update_default_category_name_200() {
    let app = TestApp::new().await;
    let token = app
        .setup_user_token("cat-upd-default@example.com", TEST_PASSWORD)
        .await;

    // Get a default category id
    let (_, list) = app.get_with_auth("/categories", &token).await;
    let default_cat = list
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["is_default"] == true)
        .expect("should have a default category");
    let id = default_cat["id"].as_str().unwrap();

    let (status, body) = app
        .put_json_with_auth(
            &format!("/categories/{id}"),
            &serde_json::json!({ "name": "Renamed Default" }),
            &token,
        )
        .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["name"], "Renamed Default");
    assert_eq!(body["is_default"], true);
}

#[tokio::test]
async fn update_category_duplicate_name_409() {
    let app = TestApp::new().await;
    let token = app
        .setup_user_token("cat-upd-dup@example.com", TEST_PASSWORD)
        .await;

    app.create_category(&token, &serde_json::json!({ "name": "First" }))
        .await;
    let second = app
        .create_category(&token, &serde_json::json!({ "name": "Second" }))
        .await;
    let second_id = second["id"].as_str().unwrap();

    let (status, body) = app
        .put_json_with_auth(
            &format!("/categories/{second_id}"),
            &serde_json::json!({ "name": "First" }),
            &token,
        )
        .await;

    assert_eq!(status, StatusCode::CONFLICT);
    assert_error(&body, "CONFLICT");
}

#[tokio::test]
async fn update_category_not_found_404() {
    let app = TestApp::new().await;
    let token = app
        .setup_user_token("cat-upd-404@example.com", TEST_PASSWORD)
        .await;

    let fake_id = uuid::Uuid::new_v4();
    let (status, body) = app
        .put_json_with_auth(
            &format!("/categories/{fake_id}"),
            &serde_json::json!({ "name": "Nope" }),
            &token,
        )
        .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_error(&body, "NOT_FOUND");
}

#[tokio::test]
async fn update_category_other_user_404() {
    let app = TestApp::new().await;
    let token_a = app
        .setup_user_token("cat-upd-other-a@example.com", TEST_PASSWORD)
        .await;
    let token_b = app
        .setup_user_token("cat-upd-other-b@example.com", TEST_PASSWORD)
        .await;

    let cat = app
        .create_category(&token_a, &serde_json::json!({ "name": "Private" }))
        .await;
    let id = cat["id"].as_str().unwrap();

    let (status, body) = app
        .put_json_with_auth(
            &format!("/categories/{id}"),
            &serde_json::json!({ "name": "Hijacked" }),
            &token_b,
        )
        .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_error(&body, "NOT_FOUND");
}

#[tokio::test]
async fn update_category_without_auth_401() {
    let app = TestApp::new().await;
    let fake_id = uuid::Uuid::new_v4();

    let (status, body) = app
        .put_json_with_auth(
            &format!("/categories/{fake_id}"),
            &serde_json::json!({ "name": "Nope" }),
            "invalid-token",
        )
        .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_error(&body, "UNAUTHORIZED");
}

#[tokio::test]
async fn update_category_position_200() {
    let app = TestApp::new().await;
    let token = app
        .setup_user_token("cat-upd-pos@example.com", TEST_PASSWORD)
        .await;

    let cat = app
        .create_category(&token, &serde_json::json!({ "name": "Movable" }))
        .await;
    let id = cat["id"].as_str().unwrap();

    let (status, body) = app
        .put_json_with_auth(
            &format!("/categories/{id}"),
            &serde_json::json!({ "position": 1 }),
            &token,
        )
        .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["position"], 1);
    assert_eq!(body["name"], "Movable");
}

#[tokio::test]
async fn update_category_negative_position_400() {
    let app = TestApp::new().await;
    let token = app
        .setup_user_token("cat-upd-negpos@example.com", TEST_PASSWORD)
        .await;

    let cat = app
        .create_category(&token, &serde_json::json!({ "name": "NegPos" }))
        .await;
    let id = cat["id"].as_str().unwrap();

    let (status, body) = app
        .put_json_with_auth(
            &format!("/categories/{id}"),
            &serde_json::json!({ "position": -1 }),
            &token,
        )
        .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_error(&body, "BAD_REQUEST");
}

#[tokio::test]
async fn update_category_empty_body_200() {
    let app = TestApp::new().await;
    let token = app
        .setup_user_token("cat-upd-empty@example.com", TEST_PASSWORD)
        .await;

    let cat = app
        .create_category(&token, &serde_json::json!({ "name": "NoChange" }))
        .await;
    let id = cat["id"].as_str().unwrap();

    let (status, body) = app
        .put_json_with_auth(&format!("/categories/{id}"), &serde_json::json!({}), &token)
        .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["name"], "NoChange");
    assert_eq!(body["id"], id);
}

// ── Delete ───────────────────────────────────────────────────────────

#[tokio::test]
async fn delete_custom_category_204() {
    let app = TestApp::new().await;
    let token = app
        .setup_user_token("cat-del@example.com", TEST_PASSWORD)
        .await;

    let cat = app
        .create_category(&token, &serde_json::json!({ "name": "ToDelete" }))
        .await;
    let id = cat["id"].as_str().unwrap();

    let (status, _) = app
        .delete_with_auth(&format!("/categories/{id}"), &token)
        .await;

    assert_eq!(status, StatusCode::NO_CONTENT);

    // Verify it's gone
    let (_, list) = app.get_with_auth("/categories", &token).await;
    let names: Vec<&str> = list
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["name"].as_str().unwrap())
        .collect();
    assert!(!names.contains(&"ToDelete"));
}

#[tokio::test]
async fn delete_default_category_400() {
    let app = TestApp::new().await;
    let token = app
        .setup_user_token("cat-del-default@example.com", TEST_PASSWORD)
        .await;

    // Get a default category id
    let (_, list) = app.get_with_auth("/categories", &token).await;
    let default_cat = list
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["is_default"] == true)
        .expect("should have a default category");
    let id = default_cat["id"].as_str().unwrap();

    let (status, body) = app
        .delete_with_auth(&format!("/categories/{id}"), &token)
        .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_error(&body, "BAD_REQUEST");
}

#[tokio::test]
async fn delete_category_not_found_404() {
    let app = TestApp::new().await;
    let token = app
        .setup_user_token("cat-del-404@example.com", TEST_PASSWORD)
        .await;

    let fake_id = uuid::Uuid::new_v4();
    let (status, body) = app
        .delete_with_auth(&format!("/categories/{fake_id}"), &token)
        .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_error(&body, "NOT_FOUND");
}

#[tokio::test]
async fn delete_category_other_user_404() {
    let app = TestApp::new().await;
    let token_a = app
        .setup_user_token("cat-del-other-a@example.com", TEST_PASSWORD)
        .await;
    let token_b = app
        .setup_user_token("cat-del-other-b@example.com", TEST_PASSWORD)
        .await;

    let cat = app
        .create_category(&token_a, &serde_json::json!({ "name": "Mine" }))
        .await;
    let id = cat["id"].as_str().unwrap();

    let (status, body) = app
        .delete_with_auth(&format!("/categories/{id}"), &token_b)
        .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_error(&body, "NOT_FOUND");
}

#[tokio::test]
async fn delete_category_without_auth_401() {
    let app = TestApp::new().await;
    let fake_id = uuid::Uuid::new_v4();

    let (status, body) = app
        .delete_with_auth(&format!("/categories/{fake_id}"), "invalid-token")
        .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_error(&body, "UNAUTHORIZED");
}

#[tokio::test]
async fn delete_category_nullifies_items() {
    let app = TestApp::new().await;
    let token = app
        .setup_user_token("cat-del-null@example.com", TEST_PASSWORD)
        .await;

    // Create a custom category
    let cat = app
        .create_category(&token, &serde_json::json!({ "name": "Ephemeral" }))
        .await;
    let cat_id = cat["id"].as_str().unwrap();

    // Create an item in that category
    let item = app
        .create_item(
            &token,
            &serde_json::json!({ "name": "Linked item", "category_id": cat_id }),
        )
        .await;
    let item_id = item["id"].as_str().unwrap();
    assert_eq!(item["category_id"], cat_id);

    // Delete the category
    let (status, _) = app
        .delete_with_auth(&format!("/categories/{cat_id}"), &token)
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Item should now have category_id = null
    let (status, item_after) = app
        .get_with_auth(&format!("/items/{item_id}"), &token)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        item_after["category_id"].is_null(),
        "category_id should be null after category deletion, got: {}",
        item_after["category_id"]
    );
}

#[tokio::test]
async fn delete_category_twice_404() {
    let app = TestApp::new().await;
    let token = app
        .setup_user_token("cat-del-twice@example.com", TEST_PASSWORD)
        .await;

    let cat = app
        .create_category(&token, &serde_json::json!({ "name": "DoubleDelete" }))
        .await;
    let id = cat["id"].as_str().unwrap();

    // First delete succeeds
    let (status, _) = app
        .delete_with_auth(&format!("/categories/{id}"), &token)
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Second delete returns 404
    let (status, body) = app
        .delete_with_auth(&format!("/categories/{id}"), &token)
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_error(&body, "NOT_FOUND");
}

// ── Cache consistency ────────────────────────────────────────────────

#[tokio::test]
async fn list_categories_consistent_after_mutations() {
    let app = TestApp::new().await;
    let token = app
        .setup_user_token("cat-cache@example.com", TEST_PASSWORD)
        .await;

    // Initial list: 6 defaults
    let (_, list) = app.get_with_auth("/categories", &token).await;
    assert_eq!(list.as_array().unwrap().len(), 6);

    // Create → list should see 7
    let cat = app
        .create_category(&token, &serde_json::json!({ "name": "CacheTest" }))
        .await;
    let id = cat["id"].as_str().unwrap();

    let (_, list) = app.get_with_auth("/categories", &token).await;
    assert_eq!(list.as_array().unwrap().len(), 7);

    // Update → list should reflect new name
    app.put_json_with_auth(
        &format!("/categories/{id}"),
        &serde_json::json!({ "name": "CacheTestUpdated" }),
        &token,
    )
    .await;

    let (_, list) = app.get_with_auth("/categories", &token).await;
    let names: Vec<&str> = list
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["name"].as_str().unwrap())
        .collect();
    assert!(names.contains(&"CacheTestUpdated"));
    assert!(!names.contains(&"CacheTest"));

    // Delete → list should be back to 6
    app.delete_with_auth(&format!("/categories/{id}"), &token)
        .await;

    let (_, list) = app.get_with_auth("/categories", &token).await;
    assert_eq!(list.as_array().unwrap().len(), 6);
}
