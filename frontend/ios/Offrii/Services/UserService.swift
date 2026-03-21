import Foundation

// MARK: - User Service

/// Provides high-level operations for user profile management.
///
/// All methods delegate to `APIClient.shared` using the corresponding
/// `APIEndpoint` cases.
final class UserService: Sendable {
    static let shared = UserService()

    private let client = APIClient.shared

    private init() {}

    // MARK: - Get Profile

    /// Fetches the authenticated user's profile.
    ///
    /// - Returns: A `UserProfileResponse` with current profile data.
    func getProfile() async throws -> UserProfileResponse {
        try await client.request(.getProfile)
    }

    // MARK: - Update Profile

    /// Updates the authenticated user's profile. Only non-nil fields are sent.
    ///
    /// - Parameters:
    ///   - displayName: New display name, or nil to leave unchanged.
    ///   - username: New username, or nil to leave unchanged.
    /// - Returns: The updated `UserProfileResponse`.
    func updateProfile(
        displayName: String? = nil,
        username: String? = nil
    ) async throws -> UserProfileResponse {
        let body = UpdateProfileBody(
            displayName: displayName,
            username: username,
            avatarUrl: nil
        )
        return try await client.request(.updateProfile(body))
    }

    // MARK: - Email Change

    func requestEmailChange(newEmail: String) async throws {
        try await client.requestVoid(.requestEmailChange(ChangeEmailBody(newEmail: newEmail)))
    }

    // MARK: - Delete Account

    /// Permanently deletes the authenticated user's account and all associated data.
    func deleteAccount() async throws {
        try await client.requestVoid(.deleteAccount)
    }

    // MARK: - Export Data

    /// Exports all user data (profile, items, categories) as a single response.
    ///
    /// - Returns: A `UserDataExport` containing the complete user data set.
    func exportData() async throws -> UserDataExport {
        try await client.request(.exportData)
    }

    // MARK: - Email Verification

    func verifyEmail(token: String) async throws {
        try await client.requestVoid(.verifyEmail(VerifyEmailBody(token: token)))
    }

    func resendVerification() async throws {
        try await client.requestVoid(.resendVerification)
    }
}
