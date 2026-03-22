import Foundation

// MARK: - Push Token Service

/// Provides high-level operations for managing APNs push token registration.
///
/// All methods delegate to `APIClient.shared` using the corresponding
/// `APIEndpoint` cases.
final class PushTokenService: Sendable {
    static let shared = PushTokenService()

    private let client = APIClient.shared

    private init() {}

    // MARK: - Register Token

    /// Registers an APNs device token with the backend.
    ///
    /// - Parameters:
    ///   - token: The 64-character hex APNs device token.
    ///   - platform: The platform identifier (currently only "ios").
    func registerToken(token: String, platform: String = "ios") async throws {
        let body = RegisterPushTokenBody(token: token, platform: platform)
        try await client.requestVoid(.registerToken(body))
    }

    // MARK: - Unregister Token

    /// Removes a previously registered APNs device token from the backend.
    ///
    /// - Parameter token: The 64-character hex APNs device token to unregister.
    func unregisterToken(token: String) async throws {
        try await client.requestVoid(.unregisterToken(token: token))
    }
}
