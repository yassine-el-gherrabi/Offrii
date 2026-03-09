import Foundation
import os

// MARK: - Auth Manager

/// Manages authentication state for the app.
///
/// Uses `@Observable` (Observation framework) so SwiftUI views can react
/// to changes in `currentUser` and `isAuthenticated`.
///
/// All token storage is delegated to `KeychainService`.
/// Network calls are delegated to `APIClient`.
@Observable
@MainActor
final class AuthManager {
    private(set) var currentUser: User?
    private let keychain = KeychainService.shared
    private let client = APIClient.shared
    private let logger = Logger(subsystem: "com.offrii", category: "AuthManager")

    /// Whether the user has a valid access token.
    var isAuthenticated: Bool {
        keychain.accessToken != nil
    }

    // MARK: - Register

    /// Creates a new account and stores the returned tokens.
    func register(email: String, password: String, displayName: String? = nil) async throws {
        let body = RegisterBody(
            email: email,
            password: password,
            displayName: displayName
        )

        let response: AuthResponse = try await client.request(.register(body))
        storeTokens(response.tokens)
        currentUser = response.user.toUser()
        logger.info("User registered: \(response.user.id)")
    }

    // MARK: - Login

    /// Authenticates with email and password, stores the returned tokens.
    func login(email: String, password: String) async throws {
        let body = LoginBody(email: email, password: password)

        let response: AuthResponse = try await client.request(.login(body))
        storeTokens(response.tokens)
        currentUser = response.user.toUser()
        logger.info("User logged in: \(response.user.id)")
    }

    // MARK: - Logout

    /// Invalidates the current session on the server and clears local state.
    func logout() async {
        do {
            try await client.requestVoid(.logout)
            logger.info("User logged out on server")
        } catch {
            logger.warning("Server logout failed: \(error.localizedDescription)")
        }

        clearAuthState()
    }

    // MARK: - Refresh Tokens

    /// Refreshes the access/refresh token pair.
    func refreshTokens() async throws {
        guard let refreshToken = keychain.refreshToken else {
            clearAuthState()
            throw APIError.unauthorized("No refresh token available")
        }

        let body = RefreshBody(refreshToken: refreshToken)
        let response: RefreshResponse = try await client.request(.refresh(body))
        keychain.accessToken = response.tokens.accessToken
        keychain.refreshToken = response.tokens.refreshToken
        logger.info("Tokens refreshed")
    }

    // MARK: - Load Current User

    /// Fetches the current user profile from the server.
    func loadCurrentUser() async throws {
        let profile: UserProfileResponse = try await client.request(.getProfile)
        currentUser = profile.toUser()
        logger.info("Loaded current user: \(profile.id)")
    }

    // MARK: - Private Helpers

    private func storeTokens(_ tokens: AuthTokens) {
        keychain.accessToken = tokens.accessToken
        keychain.refreshToken = tokens.refreshToken
    }

    private func clearAuthState() {
        keychain.clearAll()
        currentUser = nil
    }
}
