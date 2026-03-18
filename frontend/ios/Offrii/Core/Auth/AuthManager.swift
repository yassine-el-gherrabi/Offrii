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
        let user = response.user.toUser()
        currentUser = user
        cacheUser(user)
        logger.info("User registered: \(response.user.id)")
    }

    // MARK: - Login

    /// Authenticates with email/username and password, stores the returned tokens.
    func login(identifier: String, password: String) async throws {
        let body = LoginBody(identifier: identifier, password: password)

        let response: AuthResponse = try await client.request(.login(body))
        storeTokens(response.tokens)
        let user = response.user.toUser()
        currentUser = user
        cacheUser(user)
        logger.info("User logged in: \(response.user.id)")
    }

    // MARK: - Google Sign-In

    /// Authenticates via Google ID token, stores the returned tokens.
    @discardableResult
    func loginWithGoogle(idToken: String, displayName: String? = nil) async throws -> Bool {
        let body = GoogleAuthBody(idToken: idToken, displayName: displayName)
        let response: AuthResponse = try await client.request(.googleAuth(body))
        storeTokens(response.tokens)
        let user = response.user.toUser()
        currentUser = user
        cacheUser(user)
        logger.info("User signed in with Google: \(response.user.id)")
        return response.isNewUser
    }

    // MARK: - Apple Sign-In

    /// Authenticates via Apple ID token, stores the returned tokens.
    @discardableResult
    func loginWithApple(idToken: String, displayName: String? = nil) async throws -> Bool {
        let body = AppleAuthBody(idToken: idToken, displayName: displayName)
        let response: AuthResponse = try await client.request(.appleAuth(body))
        storeTokens(response.tokens)
        let user = response.user.toUser()
        currentUser = user
        cacheUser(user)
        logger.info("User signed in with Apple: \(response.user.id)")
        return response.isNewUser
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
        let user = profile.toUser()
        currentUser = user
        cacheUser(user)
        logger.info("Loaded current user: \(profile.id)")
    }

    // MARK: - Splash Launch

    /// Attempts to refresh tokens and load user profile at launch.
    /// Returns `true` if the user session is valid and loaded.
    func refreshAndLoadUser() async -> Bool {
        guard keychain.accessToken != nil else { return false }
        do {
            try await refreshTokens()
            try await loadCurrentUser()
            return true
        } catch {
            logger.warning("Launch refresh failed: \(error.localizedDescription)")
            clearAuthState()
            return false
        }
    }

    /// Restores the cached user from UserDefaults for instant display.
    func restoreCachedUser() {
        guard let data = UserDefaults.standard.data(forKey: "cachedUser"),
              let user = try? JSONDecoder().decode(User.self, from: data) else { return }
        currentUser = user
    }

    // MARK: - Private Helpers

    private func storeTokens(_ tokens: AuthTokens) {
        keychain.accessToken = tokens.accessToken
        keychain.refreshToken = tokens.refreshToken
    }

    private func cacheUser(_ user: User) {
        if let data = try? JSONEncoder().encode(user) {
            UserDefaults.standard.set(data, forKey: "cachedUser")
        }
    }

    private func clearAuthState() {
        keychain.clearAll()
        currentUser = nil
        UserDefaults.standard.removeObject(forKey: "cachedUser")
    }
}
