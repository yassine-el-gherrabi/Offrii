use std::sync::LazyLock;
use std::time::Duration;

use sha1::{Digest, Sha1};

static HTTP_CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .user_agent("offrii-backend/0.1")
        .build()
        .expect("Failed to build HTTP client")
});

/// Check if a password appears in the Have I Been Pwned breached-passwords database
/// using the k-Anonymity range API.
///
/// **Fail-open**: if HIBP is unreachable or returns an error, we log a warning
/// and allow the password (returns `Ok(false)`).
pub async fn is_breached(password: &str) -> anyhow::Result<bool> {
    let hash = hex_sha1(password);
    let (prefix, suffix) = hash.split_at(5);

    let url = format!("https://api.pwnedpasswords.com/range/{prefix}");

    let response = match HTTP_CLIENT
        .get(&url)
        .header("Add-Padding", "true")
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!(error = %e, "HIBP API unreachable, failing open");
            return Ok(false);
        }
    };

    if !response.status().is_success() {
        tracing::warn!(status = %response.status(), "HIBP API returned error, failing open");
        return Ok(false);
    }

    let body = match response.text().await {
        Ok(t) => t,
        Err(e) => {
            tracing::warn!(error = %e, "Failed to read HIBP response, failing open");
            return Ok(false);
        }
    };

    Ok(body.lines().any(|line| {
        line.split(':')
            .next()
            .is_some_and(|h| h.eq_ignore_ascii_case(suffix))
    }))
}

fn hex_sha1(input: &str) -> String {
    let mut hasher = Sha1::new();
    hasher.update(input.as_bytes());
    format!("{:X}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha1_hash_is_correct() {
        // SHA-1("password") = 5BAA61E4C9B93F3F0682250B6CF8331B7EE68FD8
        assert_eq!(
            hex_sha1("password"),
            "5BAA61E4C9B93F3F0682250B6CF8331B7EE68FD8"
        );
    }

    #[test]
    fn parses_hibp_response_line() {
        let suffix = "1E4C9B93F3F0682250B6CF8331B7EE68FD8";
        let response_body = "0018A45C4D1DEF81644B54AB7F969B88D65:1\r\n\
                             1E4C9B93F3F0682250B6CF8331B7EE68FD8:3861493\r\n\
                             1E4F21D3B11E5F7C8A5C7D5D1F3B5A3E2C1:0";

        let found = response_body.lines().any(|line| {
            line.split(':')
                .next()
                .is_some_and(|h| h.eq_ignore_ascii_case(suffix))
        });
        assert!(found);
    }

    #[test]
    fn does_not_match_absent_suffix() {
        let suffix = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
        let response_body = "0018A45C4D1DEF81644B54AB7F969B88D65:1\r\n\
                             1E4C9B93F3F0682250B6CF8331B7EE68FD8:3861493";

        let found = response_body.lines().any(|line| {
            line.split(':')
                .next()
                .is_some_and(|h| h.eq_ignore_ascii_case(suffix))
        });
        assert!(!found);
    }
}
