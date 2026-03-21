use std::net::IpAddr;
use std::time::Duration;

/// OpenGraph metadata extracted from a URL.
pub struct OgMetadata {
    pub image_url: Option<String>,
    pub title: Option<String>,
    pub site_name: Option<String>,
}

/// Check if an IP address is private/reserved (SSRF protection).
fn is_private_ip(ip: &IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            v4.is_loopback()             // 127.0.0.0/8
                || v4.is_private()       // 10.0.0.0/8, 172.16.0.0/12, 192.168.0.0/16
                || v4.is_link_local()    // 169.254.0.0/16 (cloud metadata!)
                || v4.is_broadcast()
                || v4.is_unspecified()
                || v4.octets()[0] == 100 && v4.octets()[1] >= 64 && v4.octets()[1] <= 127 // CGNAT
        }
        IpAddr::V6(v6) => v6.is_loopback() || v6.is_unspecified(),
    }
}

/// Fetch OpenGraph metadata from a URL.
///
/// - SSRF protection: blocks private/reserved IPs
/// - Timeout: 10 seconds
/// - Max response body: 1 MB
/// - Follows up to 5 redirects
/// - Falls back to twitter:image / twitter:title if no OG tags
pub async fn fetch_og_metadata(url: &str) -> Result<OgMetadata, anyhow::Error> {
    // Validate URL scheme
    let parsed = reqwest::Url::parse(url)?;
    if !matches!(parsed.scheme(), "http" | "https") {
        anyhow::bail!("only http/https URLs are allowed");
    }

    // Resolve hostname and block private IPs (SSRF protection)
    if let Some(host) = parsed.host_str() {
        use tokio::net::lookup_host;
        let lookup = format!("{}:{}", host, parsed.port_or_known_default().unwrap_or(443));
        if let Ok(addrs) = lookup_host(&lookup).await {
            for addr in addrs {
                if is_private_ip(&addr.ip()) {
                    anyhow::bail!("cannot fetch URLs pointing to private/internal addresses");
                }
            }
        }
    }

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36")
        .redirect(reqwest::redirect::Policy::limited(5))
        .build()?;

    let resp = client.get(url).send().await?;

    // Limit body size to 1 MB
    let bytes = resp.bytes().await?;
    if bytes.len() > 1_048_576 {
        anyhow::bail!("response body too large");
    }

    let html = String::from_utf8_lossy(&bytes);
    let document = scraper::Html::parse_document(&html);

    // Try og: tags first, fall back to twitter: tags
    let og_image =
        extract_meta(&document, "og:image").or_else(|| extract_meta(&document, "twitter:image"));
    let og_title =
        extract_meta(&document, "og:title").or_else(|| extract_meta(&document, "twitter:title"));
    let og_site_name = extract_meta(&document, "og:site_name");

    Ok(OgMetadata {
        image_url: og_image,
        title: og_title,
        site_name: og_site_name,
    })
}

fn extract_meta(doc: &scraper::Html, property: &str) -> Option<String> {
    // Try property attribute (OG standard)
    let selector_prop = scraper::Selector::parse(&format!("meta[property=\"{property}\"]")).ok()?;
    if let Some(el) = doc.select(&selector_prop).next()
        && let Some(content) = el.value().attr("content")
    {
        let trimmed = content.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }

    // Try name attribute (twitter cards, some sites)
    let selector_name = scraper::Selector::parse(&format!("meta[name=\"{property}\"]")).ok()?;
    if let Some(el) = doc.select(&selector_name).next()
        && let Some(content) = el.value().attr("content")
    {
        let trimmed = content.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_og_tags() {
        let html = r#"
        <html><head>
            <meta property="og:image" content="https://example.com/img.jpg" />
            <meta property="og:title" content="Test Page" />
            <meta property="og:site_name" content="Example" />
        </head><body></body></html>"#;

        let doc = scraper::Html::parse_document(html);
        assert_eq!(
            extract_meta(&doc, "og:image"),
            Some("https://example.com/img.jpg".into())
        );
        assert_eq!(extract_meta(&doc, "og:title"), Some("Test Page".into()));
        assert_eq!(extract_meta(&doc, "og:site_name"), Some("Example".into()));
    }

    #[test]
    fn returns_none_for_missing_tags() {
        let html = "<html><head></head><body></body></html>";
        let doc = scraper::Html::parse_document(html);
        assert_eq!(extract_meta(&doc, "og:image"), None);
        assert_eq!(extract_meta(&doc, "og:title"), None);
    }

    #[test]
    fn falls_back_to_twitter_tags() {
        let html = r#"
        <html><head>
            <meta name="twitter:image" content="https://example.com/tw.jpg" />
            <meta name="twitter:title" content="Tweet Title" />
        </head><body></body></html>"#;

        let doc = scraper::Html::parse_document(html);
        assert_eq!(
            extract_meta(&doc, "twitter:image"),
            Some("https://example.com/tw.jpg".into())
        );
        assert_eq!(
            extract_meta(&doc, "twitter:title"),
            Some("Tweet Title".into())
        );
    }

    #[test]
    fn ignores_empty_content() {
        let html = r#"<html><head>
            <meta property="og:image" content="   " />
        </head><body></body></html>"#;

        let doc = scraper::Html::parse_document(html);
        assert_eq!(extract_meta(&doc, "og:image"), None);
    }
}
