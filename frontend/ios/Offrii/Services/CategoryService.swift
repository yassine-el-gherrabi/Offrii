import Foundation

// MARK: - Category Service

/// Provides high-level operations for managing item categories.
///
/// All methods delegate to `APIClient.shared` using the corresponding
/// `APIEndpoint` cases.
final class CategoryService: Sendable {
    static let shared = CategoryService()

    private let client = APIClient.shared

    private init() {}

    // MARK: - List Categories

    /// Fetches all categories for the authenticated user, ordered by position.
    ///
    /// - Returns: An array of `CategoryResponse` objects.
    func listCategories() async throws -> [CategoryResponse] {
        try await client.request(.listCategories)
    }
}
