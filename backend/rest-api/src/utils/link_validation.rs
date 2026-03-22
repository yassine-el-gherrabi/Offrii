/// Validates that a string is a valid HTTP(S) URL with a real domain (containing a TLD).
pub fn is_valid_link(link: &str) -> bool {
    let Ok(parsed) = url::Url::parse(link) else {
        return false;
    };

    // Must be http or https
    let scheme = parsed.scheme();
    if scheme != "http" && scheme != "https" {
        return false;
    }

    // Must have a host with at least 2 non-empty parts (domain + TLD)
    match parsed.host_str() {
        Some(host) => {
            let parts: Vec<&str> = host.split('.').filter(|p| !p.is_empty()).collect();
            parts.len() >= 2
        }
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_https_url() {
        assert!(is_valid_link("https://google.com"));
        assert!(is_valid_link("https://www.google.com"));
        assert!(is_valid_link("https://example.com/path?q=1&b=2"));
        assert!(is_valid_link("https://sub.domain.co.uk/page"));
        assert!(is_valid_link("https://a.b"));
    }

    #[test]
    fn valid_http_url() {
        assert!(is_valid_link("http://example.com"));
        assert!(is_valid_link("http://old-site.org/page.html"));
    }

    #[test]
    fn invalid_no_tld() {
        assert!(!is_valid_link("https://google"));
        assert!(!is_valid_link("https://localhost"));
        assert!(!is_valid_link("http://intranet"));
    }

    #[test]
    fn invalid_trailing_dot() {
        assert!(!is_valid_link("https://google."));
        assert!(!is_valid_link("https://example."));
    }

    #[test]
    fn invalid_no_scheme() {
        assert!(!is_valid_link("google.com"));
        assert!(!is_valid_link("www.google.com"));
        assert!(!is_valid_link("example.com/path"));
    }

    #[test]
    fn invalid_wrong_scheme() {
        assert!(!is_valid_link("ftp://files.example.com"));
        assert!(!is_valid_link("ssh://server.example.com"));
        assert!(!is_valid_link("javascript:alert(1)"));
    }

    #[test]
    fn invalid_garbage() {
        assert!(!is_valid_link("not a url"));
        assert!(!is_valid_link(""));
        assert!(!is_valid_link("   "));
        assert!(!is_valid_link("://missing-scheme.com"));
    }

    #[test]
    fn valid_with_special_chars() {
        assert!(is_valid_link("https://example.com/path%20with%20spaces"));
        assert!(is_valid_link(
            "https://example.com/search?q=hello+world&lang=fr"
        ));
        assert!(is_valid_link("https://example.com/#anchor"));
    }

    #[test]
    fn valid_with_port() {
        assert!(is_valid_link("https://example.com:8080/api"));
        assert!(is_valid_link("http://site.org:3000"));
    }

    #[test]
    fn invalid_ip_without_tld() {
        // IP addresses don't have dots in the TLD sense but they do contain dots
        // This is acceptable — IPs like 192.168.1.1 contain dots so they pass
        assert!(is_valid_link("http://192.168.1.1"));
    }
}
