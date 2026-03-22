import Foundation

// MARK: - API Error

/// Typed error cases matching the Rust/Axum backend error response format.
///
/// The backend returns errors as:
/// ```json
/// { "error": { "code": "UNAUTHORIZED", "message": "invalid token" } }
/// ```
enum APIError: LocalizedError {
    case invalidURL
    case unauthorized(String)
    case badRequest(String)
    case notFound(String)
    case conflict(String)
    case tooManyRequests(String)
    case serverError
    case networkError(Error)
    case decodingError(Error)
    case unknown(Int, String)

    var errorDescription: String? {
        switch self {
        case .invalidURL:
            return String(localized: "error.invalidURL")
        case .unauthorized(let message):
            return message
        case .badRequest(let message):
            return message
        case .notFound(let message):
            return message
        case .conflict(let message):
            return message
        case .tooManyRequests(let message):
            return message
        case .serverError:
            return String(localized: "error.serverError")
        case .networkError:
            return String(localized: "error.networkError")
        case .decodingError:
            return String(localized: "error.decodingError")
        case .unknown(_, let message):
            return message
        }
    }
}

// MARK: - Backend Error Response

/// Mirrors the `{ "error": { "code": "...", "message": "..." } }` JSON envelope.
struct ErrorResponseBody: Decodable {
    let error: ErrorDetail

    struct ErrorDetail: Decodable {
        let code: String
        let message: String
    }
}
