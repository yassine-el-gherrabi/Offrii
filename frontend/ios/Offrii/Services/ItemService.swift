import Foundation

// MARK: - Item Service

/// Provides high-level operations for managing wishlist items.
///
/// All methods delegate to `APIClient.shared` using the corresponding
/// `APIEndpoint` cases. JSON encoding/decoding, authentication, and
/// token refresh are handled transparently by the client.
final class ItemService: Sendable {
    static let shared = ItemService()

    private let client = APIClient.shared

    private init() {}

    // MARK: - List Items

    /// Fetches a paginated, filterable list of the authenticated user's items.
    ///
    /// - Parameters:
    ///   - status: Optional status filter (e.g. "active", "purchased").
    ///   - categoryId: Optional category UUID to filter by.
    ///   - sort: Optional sort field (e.g. "created_at", "priority").
    ///   - order: Optional sort direction ("asc" or "desc").
    ///   - page: Page number (1-based). Defaults to 1.
    ///   - perPage: Number of items per page. Defaults to 20.
    /// - Returns: An `ItemsListResponse` containing items and pagination metadata.
    func listItems(
        status: String? = nil,
        categoryId: UUID? = nil,
        sort: String? = nil,
        order: String? = nil,
        page: Int = 1,
        perPage: Int = 20
    ) async throws -> ItemsListResponse {
        let query = ListItemsQuery(
            status: status,
            categoryId: categoryId,
            sort: sort,
            order: order,
            page: page,
            perPage: perPage
        )
        return try await client.request(.listItems(query))
    }

    // MARK: - Create Item

    /// Creates a new wishlist item.
    ///
    /// - Parameters:
    ///   - name: The item name (required, 1-255 characters).
    ///   - description: Optional description (max 5000 characters).
    ///   - url: Optional URL for the item (max 2048 characters).
    ///   - estimatedPrice: Optional estimated price.
    ///   - priority: Optional priority level (1 = low, 2 = medium, 3 = high).
    ///   - categoryId: Optional category UUID to assign.
    /// - Returns: The newly created `Item`.
    func createItem(
        name: String,
        description: String? = nil,
        url: String? = nil,
        estimatedPrice: Decimal? = nil,
        priority: Int? = nil,
        categoryId: UUID? = nil
    ) async throws -> Item {
        let body = CreateItemBody(
            name: name,
            description: description,
            url: url,
            estimatedPrice: estimatedPrice,
            priority: priority.map { Int16($0) },
            categoryId: categoryId
        )
        return try await client.request(.createItem(body))
    }

    // MARK: - Get Item

    /// Fetches a single item by its identifier.
    ///
    /// - Parameter id: The UUID of the item to retrieve.
    /// - Returns: The matching `Item`.
    func getItem(id: UUID) async throws -> Item {
        try await client.request(.getItem(id: id))
    }

    // MARK: - Update Item

    /// Updates an existing item. Only non-nil fields are sent to the API.
    ///
    /// - Parameters:
    ///   - id: The UUID of the item to update.
    ///   - name: New name, or nil to leave unchanged.
    ///   - description: New description, or nil to leave unchanged.
    ///   - url: New URL, or nil to leave unchanged.
    ///   - estimatedPrice: New price, or nil to leave unchanged.
    ///   - priority: New priority, or nil to leave unchanged.
    ///   - categoryId: New category UUID, or nil to leave unchanged.
    ///   - status: New status string, or nil to leave unchanged.
    /// - Returns: The updated `Item`.
    func updateItem(
        id: UUID,
        name: String? = nil,
        description: String? = nil,
        url: String? = nil,
        estimatedPrice: Decimal? = nil,
        priority: Int? = nil,
        categoryId: UUID? = nil,
        status: String? = nil
    ) async throws -> Item {
        let body = UpdateItemBody(
            name: name,
            description: description,
            url: url,
            estimatedPrice: estimatedPrice,
            priority: priority.map { Int16($0) },
            categoryId: categoryId,
            status: status
        )
        return try await client.request(.updateItem(id: id, body: body))
    }

    // MARK: - Delete Item

    /// Permanently deletes an item.
    ///
    /// - Parameter id: The UUID of the item to delete.
    func deleteItem(id: UUID) async throws {
        try await client.requestVoid(.deleteItem(id: id))
    }
}
