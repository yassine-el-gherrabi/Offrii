import Foundation

/// Auto-prefixes https:// if no scheme is present, then validates.
func normalizeURL(_ string: String) -> String {
    let trimmed = string.trimmingCharacters(in: .whitespaces)
    if trimmed.isEmpty { return trimmed }
    if trimmed.hasPrefix("http://") || trimmed.hasPrefix("https://") {
        return trimmed
    }
    return "https://\(trimmed)"
}

/// Validates that a string is a valid URL with http(s) scheme and a real domain (with TLD).
func isValidURL(_ string: String) -> Bool {
    let normalized = normalizeURL(string)
    guard let url = URL(string: normalized),
          let scheme = url.scheme?.lowercased(),
          scheme == "http" || scheme == "https",
          let host = url.host
    else {
        return false
    }
    // Host must have at least 2 non-empty parts (domain + TLD)
    let parts = host.split(separator: ".")
    return parts.count >= 2 && parts.allSatisfy { !$0.isEmpty }
}
