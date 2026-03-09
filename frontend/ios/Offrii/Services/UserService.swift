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
    ///   - reminderFreq: New reminder frequency, or nil to leave unchanged.
    ///   - reminderTime: New reminder time (HH:mm:ss), or nil to leave unchanged.
    ///   - timezone: New timezone identifier, or nil to leave unchanged.
    ///   - locale: New locale code, or nil to leave unchanged.
    /// - Returns: The updated `UserProfileResponse`.
    func updateProfile(
        displayName: String? = nil,
        username: String? = nil,
        reminderFreq: String? = nil,
        reminderTime: String? = nil,
        timezone: String? = nil,
        locale: String? = nil
    ) async throws -> UserProfileResponse {
        let body = UpdateProfileBody(
            displayName: displayName,
            username: username,
            reminderFreq: reminderFreq,
            reminderTime: reminderTime,
            timezone: timezone,
            locale: locale
        )
        return try await client.request(.updateProfile(body))
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
}
