import Foundation

// MARK: - Wish Message Service

final class WishMessageService: Sendable {
    static let shared = WishMessageService()

    private let client = APIClient.shared

    private init() {}

    // MARK: - List Messages

    func listMessages(
        wishId: UUID,
        page: Int = 1,
        limit: Int = 50
    ) async throws -> PaginatedResponse<WishMessage> {
        let query = ListWishMessagesQuery(page: page, limit: limit)
        return try await client.request(.listWishMessages(wishId: wishId, query: query))
    }

    // MARK: - Send Message

    func sendMessage(wishId: UUID, body: String) async throws -> WishMessage {
        let messageBody = SendWishMessageBody(body: body)
        return try await client.request(.sendWishMessage(wishId: wishId, body: messageBody))
    }
}
