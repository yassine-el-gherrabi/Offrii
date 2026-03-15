import Foundation

final class CircleService: Sendable {
    static let shared = CircleService()
    private let client = APIClient.shared
    private init() {}

    func listCircles() async throws -> [OffriiCircle] {
        try await client.request(.listCircles)
    }

    func createCircle(name: String) async throws -> OffriiCircle {
        try await client.request(.createCircle(CreateCircleBody(name: name)))
    }

    func getCircle(id: UUID) async throws -> CircleDetailResponse {
        try await client.request(.getCircle(id: id))
    }

    func updateCircle(id: UUID, name: String) async throws -> OffriiCircle {
        try await client.request(.updateCircle(id: id, body: UpdateCircleBody(name: name)))
    }

    func deleteCircle(id: UUID) async throws {
        try await client.requestVoid(.deleteCircle(id: id))
    }

    func createDirectCircle(userId: UUID) async throws -> OffriiCircle {
        try await client.request(.createDirectCircle(userId: userId))
    }

    func addMember(circleId: UUID, userId: UUID) async throws {
        try await client.requestVoid(.addMemberToCircle(circleId: circleId, body: AddMemberBody(userId: userId)))
    }

    func removeMember(circleId: UUID, userId: UUID) async throws {
        try await client.requestVoid(.removeMember(circleId: circleId, userId: userId))
    }

    func shareItem(circleId: UUID, itemId: UUID) async throws {
        try await client.requestVoid(.shareItemToCircle(circleId: circleId, body: ShareItemBody(itemId: itemId)))
    }

    func listItems(circleId: UUID) async throws -> [CircleItemResponse] {
        try await client.request(.listCircleItems(circleId: circleId))
    }

    func getItem(circleId: UUID, itemId: UUID) async throws -> CircleItemResponse {
        try await client.request(.getCircleItem(circleId: circleId, itemId: itemId))
    }

    func unshareItem(circleId: UUID, itemId: UUID) async throws {
        try await client.requestVoid(.unshareItem(circleId: circleId, itemId: itemId))
    }

    func getFeed(circleId: UUID, page: Int = 1, perPage: Int = 20) async throws -> FeedResponse {
        try await client.request(.getCircleFeed(circleId: circleId, page: page, perPage: perPage))
    }

    func claimItem(itemId: UUID) async throws {
        try await client.requestVoid(.claimItem(id: itemId))
    }

    func unclaimItem(itemId: UUID) async throws {
        try await client.requestVoid(.unclaimItem(id: itemId))
    }
}
