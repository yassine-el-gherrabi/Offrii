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

#[tokio::test]
async fn list_categories_same_for_all_users() {
    let app = TestApp::new().await;
    let token_a = app
        .setup_user_token("cat-same-a@example.com", TEST_PASSWORD)
        .await;
    let token_b = app
        .setup_user_token("cat-same-b@example.com", TEST_PASSWORD)
        .await;

    let (_, body_a) = app.get_with_auth("/categories", &token_a).await;
    let (_, body_b) = app.get_with_auth("/categories", &token_b).await;

    let ids_a: Vec<&str> = body_a
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["id"].as_str().unwrap())
        .collect();
    let ids_b: Vec<&str> = body_b
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["id"].as_str().unwrap())
        .collect();

    assert_eq!(
        ids_a, ids_b,
        "all users should see the same global categories"
    );
}

#[tokio::test]
async fn list_categories_has_expected_names() {
    let app = TestApp::new().await;
    let token = app
        .setup_user_token("cat-names@example.com", TEST_PASSWORD)
        .await;

    let (_, body) = app.get_with_auth("/categories", &token).await;
    let names: Vec<&str> = body
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["name"].as_str().unwrap())
        .collect();

    assert!(names.contains(&"Tech"));
    assert!(names.contains(&"Mode"));
    assert!(names.contains(&"Maison"));
    assert!(names.contains(&"Loisirs"));
    assert!(names.contains(&"Santé"));
    assert!(names.contains(&"Autre"));
}

#[tokio::test]
async fn list_categories_has_expected_icons() {
    let app = TestApp::new().await;
    let token = app
        .setup_user_token("cat-icons@example.com", TEST_PASSWORD)
        .await;

    let (_, body) = app.get_with_auth("/categories", &token).await;
    let icons: Vec<&str> = body
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["icon"].as_str().unwrap())
        .collect();

    assert!(icons.contains(&"laptop"));
    assert!(icons.contains(&"tshirt"));
    assert!(icons.contains(&"home"));
    assert!(icons.contains(&"gamepad"));
    assert!(icons.contains(&"heart"));
    assert!(icons.contains(&"tag"));
}

#[tokio::test]
async fn list_categories_has_expected_fields() {
    let app = TestApp::new().await;
    let token = app
        .setup_user_token("cat-fields@example.com", TEST_PASSWORD)
        .await;

    let (_, body) = app.get_with_auth("/categories", &token).await;
    let cat = &body.as_array().unwrap()[0];

    assert!(cat["id"].is_string(), "should have id");
    assert!(cat["name"].is_string(), "should have name");
    assert!(cat["icon"].is_string(), "should have icon");
    assert!(cat["is_default"].is_boolean(), "should have is_default");
    assert!(cat["position"].is_number(), "should have position");
    assert!(cat["created_at"].is_string(), "should have created_at");
}

// ── CRUD endpoints removed ───────────────────────────────────────────

#[tokio::test]
async fn create_category_endpoint_not_found() {
    let app = TestApp::new().await;
    let token = app
        .setup_user_token("cat-no-create@example.com", TEST_PASSWORD)
        .await;

    let (status, _) = app
        .post_json_with_auth(
            "/categories",
            &serde_json::json!({ "name": "Custom" }),
            &token,
        )
        .await;

    assert_eq!(status, StatusCode::METHOD_NOT_ALLOWED);
}

#[tokio::test]
async fn delete_category_endpoint_not_found() {
    let app = TestApp::new().await;
    let token = app
        .setup_user_token("cat-no-delete@example.com", TEST_PASSWORD)
        .await;

    let (_, body) = app.get_with_auth("/categories", &token).await;
    let id = body.as_array().unwrap()[0]["id"].as_str().unwrap();

    let (status, _) = app
        .delete_with_auth(&format!("/categories/{id}"), &token)
        .await;

    // No route exists for DELETE /categories/{id} — returns 404 or 405
    assert!(
        status == StatusCode::NOT_FOUND || status == StatusCode::METHOD_NOT_ALLOWED,
        "expected 404 or 405, got {status}"
    );
}
