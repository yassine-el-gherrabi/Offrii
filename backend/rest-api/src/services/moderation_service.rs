use std::time::Duration;

use async_trait::async_trait;

use crate::traits;

// ── ModerationResult ─────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum ModerationResult {
    Approved,
    Flagged { reason: String },
}

// ── OpenAI Moderation API implementation ─────────────────────────────

pub struct OpenAIModerationService {
    client: reqwest::Client,
    api_key: String,
    base_url: String,
}

impl OpenAIModerationService {
    pub fn new(api_key: String) -> Self {
        Self::with_base_url(api_key, "https://api.openai.com".to_string())
    }

    pub fn with_base_url(api_key: String, base_url: String) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("failed to build reqwest client");

        Self {
            client,
            api_key,
            base_url,
        }
    }
}

#[async_trait]
impl traits::ModerationService for OpenAIModerationService {
    async fn check_wish(
        &self,
        title: &str,
        description: Option<&str>,
        category: &str,
        image_url: Option<&str>,
        links: Option<&[String]>,
    ) -> Result<ModerationResult, anyhow::Error> {
        let desc = description.unwrap_or("");
        let mut text_parts = vec![title.to_string(), desc.to_string(), category.to_string()];
        if let Some(l) = links {
            for link in l {
                text_parts.push(link.clone());
            }
        }
        let combined_text = text_parts.join("\n");

        // Build input: multi-modal if image_url is present, text-only otherwise
        let input = if let Some(img_url) = image_url {
            serde_json::json!([
                { "type": "text", "text": combined_text },
                { "type": "image_url", "image_url": { "url": img_url } }
            ])
        } else {
            serde_json::json!(combined_text)
        };

        let body = serde_json::json!({
            "model": "omni-moderation-latest",
            "input": input
        });

        let url = format!("{}/v1/moderations", self.base_url);
        let resp = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&body)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!(
                "OpenAI moderation API returned status {status}: {text}"
            ));
        }

        let json: serde_json::Value = resp.json().await?;

        let flagged = json["results"]
            .get(0)
            .and_then(|r| r["flagged"].as_bool())
            .unwrap_or(false);

        if !flagged {
            return Ok(ModerationResult::Approved);
        }

        // Extract flagged categories
        let mut flagged_cats = Vec::new();
        if let Some(categories) = json["results"]
            .get(0)
            .and_then(|r| r["categories"].as_object())
        {
            for (cat, val) in categories {
                if val.as_bool() == Some(true) {
                    flagged_cats.push(cat.clone());
                }
            }
        }

        let reason = if flagged_cats.is_empty() {
            "content flagged by moderation".to_string()
        } else {
            format!("flagged categories: {}", flagged_cats.join(", "))
        };

        Ok(ModerationResult::Flagged { reason })
    }
}

// ── Noop implementation (for dev/test) ───────────────────────────────

pub struct NoopModerationService;

#[async_trait]
impl traits::ModerationService for NoopModerationService {
    async fn check_wish(
        &self,
        _title: &str,
        _description: Option<&str>,
        _category: &str,
        _image_url: Option<&str>,
        _links: Option<&[String]>,
    ) -> Result<ModerationResult, anyhow::Error> {
        Ok(ModerationResult::Approved)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::ModerationService;
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    async fn svc(server: &MockServer) -> OpenAIModerationService {
        OpenAIModerationService::with_base_url("sk-test".into(), server.uri())
    }

    #[tokio::test]
    async fn approved_result_parsing() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/moderations"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "results": [{ "flagged": false, "categories": {} }]
            })))
            .mount(&server)
            .await;

        let result = svc(&server)
            .await
            .check_wish("good title", None, "clothing", None, None)
            .await
            .unwrap();
        assert!(matches!(result, ModerationResult::Approved));
    }

    #[tokio::test]
    async fn flagged_single_category() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/moderations"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "results": [{
                    "flagged": true,
                    "categories": { "hate": true, "violence": false }
                }]
            })))
            .mount(&server)
            .await;

        let result = svc(&server)
            .await
            .check_wish("bad title", None, "other", None, None)
            .await
            .unwrap();
        match result {
            ModerationResult::Flagged { reason } => {
                assert!(reason.contains("hate"), "reason was: {reason}");
                assert!(!reason.contains("violence"), "reason was: {reason}");
            }
            _ => panic!("expected Flagged"),
        }
    }

    #[tokio::test]
    async fn flagged_multiple_categories() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/moderations"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "results": [{
                    "flagged": true,
                    "categories": { "hate": true, "violence": true, "sexual": false }
                }]
            })))
            .mount(&server)
            .await;

        let result = svc(&server)
            .await
            .check_wish("bad title", None, "other", None, None)
            .await
            .unwrap();
        match result {
            ModerationResult::Flagged { reason } => {
                assert!(reason.contains("hate"), "reason was: {reason}");
                assert!(reason.contains("violence"), "reason was: {reason}");
            }
            _ => panic!("expected Flagged"),
        }
    }

    #[tokio::test]
    async fn flagged_no_categories_uses_fallback_reason() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/moderations"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "results": [{ "flagged": true, "categories": {} }]
            })))
            .mount(&server)
            .await;

        let result = svc(&server)
            .await
            .check_wish("test", None, "other", None, None)
            .await
            .unwrap();
        match result {
            ModerationResult::Flagged { reason } => {
                assert_eq!(reason, "content flagged by moderation");
            }
            _ => panic!("expected Flagged"),
        }
    }

    #[tokio::test]
    async fn non_200_returns_error() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/moderations"))
            .respond_with(ResponseTemplate::new(429).set_body_string("rate limited"))
            .mount(&server)
            .await;

        let err = svc(&server)
            .await
            .check_wish("test", None, "other", None, None)
            .await
            .unwrap_err();
        assert!(err.to_string().contains("429"), "error was: {err}");
    }

    #[tokio::test]
    async fn missing_results_key_returns_approved() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/moderations"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
            .mount(&server)
            .await;

        let result = svc(&server)
            .await
            .check_wish("test", None, "other", None, None)
            .await
            .unwrap();
        assert!(matches!(result, ModerationResult::Approved));
    }

    #[tokio::test]
    async fn sends_bearer_token() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/moderations"))
            .and(header("Authorization", "Bearer sk-test"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "results": [{ "flagged": false, "categories": {} }]
            })))
            .expect(1)
            .mount(&server)
            .await;

        svc(&server)
            .await
            .check_wish("test", None, "other", None, None)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn text_only_payload_when_no_image() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/moderations"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "results": [{ "flagged": false, "categories": {} }]
            })))
            .mount(&server)
            .await;

        // No image_url → input should be a string, not array
        let result = svc(&server)
            .await
            .check_wish("my title", Some("desc"), "clothing", None, None)
            .await
            .unwrap();
        assert!(matches!(result, ModerationResult::Approved));

        let requests = server.received_requests().await.unwrap();
        assert_eq!(requests.len(), 1);
        let body: serde_json::Value = serde_json::from_slice(&requests[0].body).unwrap();
        assert!(
            body["input"].is_string(),
            "input should be a string for text-only"
        );
        assert!(body["input"].as_str().unwrap().contains("my title"));
    }

    #[tokio::test]
    async fn multimodal_payload_when_image_present() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/moderations"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "results": [{ "flagged": false, "categories": {} }]
            })))
            .mount(&server)
            .await;

        let result = svc(&server)
            .await
            .check_wish(
                "my title",
                None,
                "clothing",
                Some("https://img.example.com/pic.jpg"),
                None,
            )
            .await
            .unwrap();
        assert!(matches!(result, ModerationResult::Approved));

        let requests = server.received_requests().await.unwrap();
        assert_eq!(requests.len(), 1);
        let body: serde_json::Value = serde_json::from_slice(&requests[0].body).unwrap();
        assert!(
            body["input"].is_array(),
            "input should be array for multimodal"
        );
        let input = body["input"].as_array().unwrap();
        assert_eq!(input.len(), 2);
        assert_eq!(input[0]["type"].as_str(), Some("text"));
        assert_eq!(input[1]["type"].as_str(), Some("image_url"));
        assert_eq!(
            input[1]["image_url"]["url"].as_str(),
            Some("https://img.example.com/pic.jpg")
        );
    }

    #[tokio::test]
    async fn links_included_in_text_payload() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/moderations"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "results": [{ "flagged": false, "categories": {} }]
            })))
            .mount(&server)
            .await;

        let links = vec![
            "https://shop.com/item1".to_string(),
            "https://shop.com/item2".to_string(),
        ];
        svc(&server)
            .await
            .check_wish("title", None, "clothing", None, Some(&links))
            .await
            .unwrap();

        let requests = server.received_requests().await.unwrap();
        let body: serde_json::Value = serde_json::from_slice(&requests[0].body).unwrap();
        let text = body["input"].as_str().unwrap();
        assert!(text.contains("https://shop.com/item1"), "text was: {text}");
        assert!(text.contains("https://shop.com/item2"), "text was: {text}");
    }
}
