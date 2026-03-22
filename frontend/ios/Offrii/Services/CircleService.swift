import Foundation

final class CircleService: Sendable {
    static let shared = CircleService()
    private let client = APIClient.shared
    private init() {}

    func listCircles() async throws -> [OffriiCircle] {
        let response: PaginatedResponse<OffriiCircle> = try await client.request(.listCircles)
        return response.data
    }

    func createCircle(name: String) async throws -> OffriiCircle {
        try await client.request(.createCircle(CreateCircleBody(name: name)))
    }

    func getCircle(id: UUID) async throws -> CircleDetailResponse {
        try await client.request(.getCircle(id: id))
    }

    func updateCircle(id: UUID, name: String, imageUrl: String?? = nil) async throws -> OffriiCircle {
        try await client.request(.updateCircle(id: id, body: UpdateCircleBody(name: name, imageUrl: imageUrl)))
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

    func createInvite(circleId: UUID, maxUses: Int? = 5, expiresInHours: Int? = 72) async throws -> CircleInviteResponse {
        try await client.request(
            .createCircleInvite(circleId: circleId, body: CreateCircleInviteBody(maxUses: maxUses, expiresInHours: expiresInHours))
        )
    }

    func listInvites(circleId: UUID) async throws -> [CircleInviteResponse] {
        let response: PaginatedResponse<CircleInviteResponse> = try await client.request(.listCircleInvites(circleId: circleId))
        return response.data
    }

    func revokeInvite(circleId: UUID, inviteId: UUID) async throws {
        try await client.requestVoid(.revokeCircleInvite(circleId: circleId, inviteId: inviteId))
    }

    func joinViaInvite(token: String) async throws -> JoinCircleResponse {
        try await client.request(.joinCircleViaInvite(token: token))
    }

    func transferOwnership(circleId: UUID, userId: UUID) async throws {
        try await client.requestVoid(.transferCircleOwnership(circleId: circleId, body: TransferOwnershipBody(userId: userId)))
    }

    func listReservations() async throws -> [ReservationResponse] {
        let response: PaginatedResponse<ReservationResponse> = try await client.request(.listReservations)
        return response.data
    }

    func shareItem(circleId: UUID, itemId: UUID) async throws {
        try await client.requestVoid(.shareItemToCircle(circleId: circleId, body: ShareItemBody(itemId: itemId)))
    }

    func batchShareItems(circleId: UUID, itemIds: [UUID]) async throws {
        try await client.requestVoid(.batchShareItems(circleId: circleId, body: BatchShareBody(itemIds: itemIds)))
    }

    func listItems(circleId: UUID) async throws -> [CircleItemResponse] {
        let response: PaginatedResponse<CircleItemResponse> = try await client.request(.listCircleItems(circleId: circleId))
        return response.data
    }

    func getItem(circleId: UUID, itemId: UUID) async throws -> CircleItemResponse {
        try await client.request(.getCircleItem(circleId: circleId, itemId: itemId))
    }

    func unshareItem(circleId: UUID, itemId: UUID) async throws {
        try await client.requestVoid(.unshareItem(circleId: circleId, itemId: itemId))
    }

    func listMyShareRules() async throws -> [CircleShareRuleSummary] {
        let response: PaginatedResponse<CircleShareRuleSummary> = try await client.request(.listMyShareRules)
        return response.data
    }

    func getShareRule(circleId: UUID) async throws -> ShareRuleResponse {
        try await client.request(.getShareRule(circleId: circleId))
    }

    func setShareRule(circleId: UUID, mode: String, categoryIds: [UUID] = []) async throws {
        try await client.requestVoid(.setShareRule(
            circleId: circleId,
            body: SetShareRuleBody(shareMode: mode, categoryIds: categoryIds)
        ))
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
