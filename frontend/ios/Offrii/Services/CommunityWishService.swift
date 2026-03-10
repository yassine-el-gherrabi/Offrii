import Foundation

// MARK: - Community Wish Service

final class CommunityWishService: Sendable {
    static let shared = CommunityWishService()

    private let client = APIClient.shared

    private init() {}

    // MARK: - List Wishes (public feed)

    func listWishes(
        category: String? = nil,
        page: Int = 1,
        limit: Int = 20
    ) async throws -> PaginatedResponse<CommunityWish> {
        let query = ListCommunityWishesQuery(category: category, page: page, limit: limit)
        return try await client.request(.listCommunityWishes(query))
    }

    // MARK: - List My Wishes

    func listMyWishes() async throws -> [MyWish] {
        try await client.request(.listMyCommunityWishes)
    }

    // MARK: - Get Wish Detail

    func getWish(id: UUID) async throws -> WishDetail {
        try await client.request(.getCommunityWish(id: id))
    }

    // MARK: - Create Wish

    func createWish(
        title: String,
        description: String? = nil,
        category: WishCategory,
        isAnonymous: Bool = false,
        imageUrl: String? = nil,
        links: [String]? = nil
    ) async throws -> MyWish {
        let body = CreateCommunityWishBody(
            title: title,
            description: description,
            category: category.rawValue,
            isAnonymous: isAnonymous,
            imageUrl: imageUrl,
            links: links
        )
        return try await client.request(.createCommunityWish(body))
    }

    // MARK: - Update Wish

    func updateWish(
        id: UUID,
        title: String? = nil,
        description: String? = nil,
        category: String? = nil,
        imageUrl: String? = nil,
        links: [String]? = nil
    ) async throws -> MyWish {
        let body = UpdateCommunityWishBody(
            title: title,
            description: description,
            category: category,
            imageUrl: imageUrl,
            links: links
        )
        return try await client.request(.updateCommunityWish(id: id, body: body))
    }

    // MARK: - Actions

    func closeWish(id: UUID) async throws {
        try await client.requestVoid(.closeCommunityWish(id: id))
    }

    func reopenWish(id: UUID) async throws {
        try await client.requestVoid(.reopenCommunityWish(id: id))
    }

    func offerWish(id: UUID) async throws {
        try await client.requestVoid(.offerCommunityWish(id: id))
    }

    func withdrawOffer(id: UUID) async throws {
        try await client.requestVoid(.withdrawOfferCommunityWish(id: id))
    }

    func rejectOffer(id: UUID) async throws {
        try await client.requestVoid(.rejectOfferCommunityWish(id: id))
    }

    func confirmWish(id: UUID) async throws {
        try await client.requestVoid(.confirmCommunityWish(id: id))
    }

    // MARK: - Report

    func reportWish(id: UUID, reason: WishReportReason) async throws {
        let body = ReportCommunityWishBody(reason: reason.rawValue)
        try await client.requestVoid(.reportCommunityWish(id: id, body: body))
    }
}
