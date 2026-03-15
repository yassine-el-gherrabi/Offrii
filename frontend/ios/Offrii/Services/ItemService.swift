import Foundation

// MARK: - Item Service

final class ItemService: Sendable {
    static let shared = ItemService()

    private let client = APIClient.shared

    private init() {}

    // MARK: - List Items

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

    func createItem(
        name: String,
        description: String? = nil,
        url: String? = nil,
        estimatedPrice: Decimal? = nil,
        priority: Int? = nil,
        categoryId: UUID? = nil,
        imageUrl: String? = nil,
        links: [String]? = nil,
        isPrivate: Bool? = nil
    ) async throws -> Item {
        let body = CreateItemBody(
            name: name,
            description: description,
            url: url,
            estimatedPrice: estimatedPrice,
            priority: priority.map { Int16($0) },
            categoryId: categoryId,
            imageUrl: imageUrl,
            links: links,
            isPrivate: isPrivate
        )
        return try await client.request(.createItem(body))
    }

    // MARK: - Get Item

    func getItem(id: UUID) async throws -> Item {
        try await client.request(.getItem(id: id))
    }

    // MARK: - Update Item

    func updateItem(
        id: UUID,
        name: String? = nil,
        description: String? = nil,
        url: String? = nil,
        estimatedPrice: Decimal? = nil,
        priority: Int? = nil,
        categoryId: UUID? = nil,
        status: String? = nil,
        imageUrl: String? = nil,
        links: [String]? = nil,
        isPrivate: Bool? = nil
    ) async throws -> Item {
        let body = UpdateItemBody(
            name: name,
            description: description,
            url: url,
            estimatedPrice: estimatedPrice,
            priority: priority.map { Int16($0) },
            categoryId: categoryId,
            status: status,
            imageUrl: imageUrl,
            links: links,
            isPrivate: isPrivate
        )
        return try await client.request(.updateItem(id: id, body: body))
    }

    // MARK: - Delete Item

    func deleteItem(id: UUID) async throws {
        try await client.requestVoid(.deleteItem(id: id))
    }

    // MARK: - Batch Delete

    func batchDelete(ids: [UUID]) async throws {
        let body = BatchDeleteItemsBody(ids: ids)
        try await client.requestVoid(.batchDeleteItems(body))
    }

    // MARK: - Owner Unclaim Web

    func ownerUnclaimWeb(id: UUID) async throws {
        try await client.requestVoid(.ownerUnclaimWeb(id: id))
    }

    // MARK: - Upload Image

    func uploadImage(_ imageData: Data) async throws -> String {
        try await client.uploadImage(imageData)
    }
}
