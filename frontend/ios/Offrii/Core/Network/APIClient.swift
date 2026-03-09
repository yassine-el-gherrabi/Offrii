import Foundation
import os

// MARK: - API Client

/// Central HTTP client for communicating with the Offrii REST API.
///
/// Responsibilities:
/// - Builds `URLRequest` from `APIEndpoint` definitions
/// - Injects `Authorization: Bearer` headers from `KeychainService`
/// - Transparently retries once on 401 after refreshing tokens
/// - Maps HTTP status codes to typed `APIError` values
/// - Parses the backend error envelope `{ "error": { "code", "message" } }`
final class APIClient: Sendable {
    static let shared = APIClient()

    private let session: URLSession
    private let decoder: JSONDecoder
    private let encoder: JSONEncoder
    private let logger = Logger(subsystem: "com.offrii", category: "APIClient")

    private init(session: URLSession = .shared) {
        self.session = session

        let decoder = JSONDecoder()
        decoder.dateDecodingStrategy = .custom { decoder in
            let container = try decoder.singleValueContainer()
            let string = try container.decode(String.self)

            let formatter = ISO8601DateFormatter()
            formatter.formatOptions = [.withInternetDateTime, .withFractionalSeconds]
            if let date = formatter.date(from: string) { return date }

            formatter.formatOptions = [.withInternetDateTime]
            if let date = formatter.date(from: string) { return date }

            throw DecodingError.dataCorruptedError(
                in: container, debugDescription: "Cannot decode date: \(string)"
            )
        }
        self.decoder = decoder

        let encoder = JSONEncoder()
        encoder.dateEncodingStrategy = .iso8601
        encoder.keyEncodingStrategy = .convertToSnakeCase
        self.encoder = encoder
    }

    // MARK: - Public Request

    /// Performs a request and decodes the response body into `T`.
    ///
    /// - Parameter endpoint: The API endpoint to call.
    /// - Returns: The decoded response.
    /// - Throws: `APIError` on failure.
    func request<T: Decodable>(_ endpoint: APIEndpoint) async throws -> T {
        let urlRequest = try buildRequest(for: endpoint)
        return try await execute(urlRequest, endpoint: endpoint, isRetry: false)
    }

    /// Performs a request that returns no meaningful body (e.g. 204 No Content).
    ///
    /// - Parameter endpoint: The API endpoint to call.
    /// - Throws: `APIError` on failure.
    func requestVoid(_ endpoint: APIEndpoint) async throws {
        let urlRequest = try buildRequest(for: endpoint)
        let _: EmptyResponse = try await execute(urlRequest, endpoint: endpoint, isRetry: false)
    }

    // MARK: - Build Request

    private func buildRequest(for endpoint: APIEndpoint) throws -> URLRequest {
        guard let url = endpoint.url else {
            throw APIError.invalidURL
        }

        var request = URLRequest(url: url)
        request.httpMethod = endpoint.method.rawValue
        request.setValue("application/json", forHTTPHeaderField: "Accept")

        // Skip ngrok browser warning in dev builds.
        #if DEBUG
        request.setValue("true", forHTTPHeaderField: "ngrok-skip-browser-warning")
        #endif

        // Inject bearer token for authenticated endpoints.
        if endpoint.requiresAuth, let token = KeychainService.shared.accessToken {
            request.setValue("Bearer \(token)", forHTTPHeaderField: "Authorization")
        }

        // Encode JSON body.
        if let body = endpoint.body {
            request.setValue("application/json", forHTTPHeaderField: "Content-Type")
            request.httpBody = try encodeBody(body)
        }

        return request
    }

    private func encodeBody(_ body: any Encodable) throws -> Data {
        try encoder.encode(AnyEncodable(body))
    }

    // MARK: - Execute

    private func execute<T: Decodable>(
        _ request: URLRequest,
        endpoint: APIEndpoint,
        isRetry: Bool
    ) async throws -> T {
        let data: Data
        let response: URLResponse

        do {
            (data, response) = try await session.data(for: request)
        } catch {
            logger.error("Network error: \(error.localizedDescription)")
            throw APIError.networkError(error)
        }

        guard let httpResponse = response as? HTTPURLResponse else {
            throw APIError.unknown(0, "Invalid response type")
        }

        let statusCode = httpResponse.statusCode

        // Handle 401 with automatic token refresh (once).
        if statusCode == 401 && !isRetry && endpoint.requiresAuth {
            do {
                try await refreshTokens()
                var retryRequest = try buildRequest(for: endpoint)
                // Re-inject the fresh token.
                if let freshToken = KeychainService.shared.accessToken {
                    retryRequest.setValue(
                        "Bearer \(freshToken)",
                        forHTTPHeaderField: "Authorization"
                    )
                }
                return try await execute(retryRequest, endpoint: endpoint, isRetry: true)
            } catch {
                // Refresh failed -- clear auth state and propagate.
                KeychainService.shared.clearAll()
                throw APIError.unauthorized("Session expired")
            }
        }

        // Success range.
        if (200..<300).contains(statusCode) {
            // 204 No Content -- return EmptyResponse if that is what the caller expects.
            if statusCode == 204 || data.isEmpty {
                if let empty = EmptyResponse() as? T {
                    return empty
                }
                throw APIError.decodingError(
                    DecodingError.dataCorrupted(
                        .init(codingPath: [], debugDescription: "Expected data but got empty response")
                    )
                )
            }

            do {
                return try decoder.decode(T.self, from: data)
            } catch {
                logger.error("Decoding error: \(error.localizedDescription)")
                throw APIError.decodingError(error)
            }
        }

        // Error responses -- parse the backend error envelope.
        let errorMessage = parseErrorMessage(from: data)

        switch statusCode {
        case 400:
            throw APIError.badRequest(errorMessage)
        case 401:
            throw APIError.unauthorized(errorMessage)
        case 404:
            throw APIError.notFound(errorMessage)
        case 409:
            throw APIError.conflict(errorMessage)
        case 500..<600:
            throw APIError.serverError
        default:
            throw APIError.unknown(statusCode, errorMessage)
        }
    }

    // MARK: - Token Refresh

    private func refreshTokens() async throws {
        guard let refreshToken = KeychainService.shared.refreshToken else {
            throw APIError.unauthorized("No refresh token available")
        }

        let endpoint = APIEndpoint.refresh(RefreshBody(refreshToken: refreshToken))

        guard let url = endpoint.url else {
            throw APIError.invalidURL
        }

        var request = URLRequest(url: url)
        request.httpMethod = endpoint.method.rawValue
        request.setValue("application/json", forHTTPHeaderField: "Accept")
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")
        request.httpBody = try encodeBody(RefreshBody(refreshToken: refreshToken))

        let (data, response) = try await session.data(for: request)

        guard let httpResponse = response as? HTTPURLResponse,
              (200..<300).contains(httpResponse.statusCode) else {
            throw APIError.unauthorized("Token refresh failed")
        }

        let refreshResponse = try decoder.decode(RefreshTokenResponse.self, from: data)
        KeychainService.shared.accessToken = refreshResponse.tokens.accessToken
        KeychainService.shared.refreshToken = refreshResponse.tokens.refreshToken
    }

    // MARK: - Error Parsing

    /// Attempts to extract the `message` field from the backend error envelope.
    private func parseErrorMessage(from data: Data) -> String {
        if let errorBody = try? decoder.decode(ErrorResponseBody.self, from: data) {
            return errorBody.error.message
        }
        return String(data: data, encoding: .utf8) ?? "An unknown error occurred"
    }
}

// MARK: - Supporting Types

/// Used to erase the concrete `Encodable` type for `JSONEncoder.encode`.
private struct AnyEncodable: Encodable {
    private let encodeClosure: (Encoder) throws -> Void

    init(_ value: any Encodable) {
        self.encodeClosure = { encoder in
            try value.encode(to: encoder)
        }
    }

    func encode(to encoder: Encoder) throws {
        try encodeClosure(encoder)
    }
}

/// Placeholder type for endpoints that return no body (204 No Content).
struct EmptyResponse: Decodable {
    init() {}
}

/// Mirrors the refresh endpoint response: `{ "tokens": { ... } }`.
private struct RefreshTokenResponse: Decodable {
    let tokens: TokenPairResponse

    struct TokenPairResponse: Decodable {
        let accessToken: String
        let refreshToken: String
    }
}
