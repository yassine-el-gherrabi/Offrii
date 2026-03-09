import Foundation

struct CircleMember: Codable, Identifiable, Equatable {
    let userId: UUID
    let username: String
    let displayName: String?
    let role: String
    let joinedAt: Date
    var id: UUID { userId }

    enum CodingKeys: String, CodingKey {
        case userId = "user_id"
        case username
        case displayName = "display_name"
        case role
        case joinedAt = "joined_at"
    }
}

struct CircleDetailResponse: Codable {
    let id: UUID
    let name: String?
    let isDirect: Bool
    let ownerId: UUID
    var members: [CircleMember]
    let createdAt: Date

    enum CodingKeys: String, CodingKey {
        case id
        case name
        case isDirect = "is_direct"
        case ownerId = "owner_id"
        case members
        case createdAt = "created_at"
    }
}

struct CircleItemResponse: Codable, Identifiable {
    let id: UUID
    let name: String
    let description: String?
    let url: String?
    let estimatedPrice: Decimal?
    let priority: Int16
    let categoryId: UUID?
    let status: String
    let isClaimed: Bool
    let claimedBy: ClaimedByInfo?
    let sharedAt: Date
    let sharedBy: UUID

    enum CodingKeys: String, CodingKey {
        case id
        case name
        case description
        case url
        case estimatedPrice = "estimated_price"
        case priority
        case categoryId = "category_id"
        case status
        case isClaimed = "is_claimed"
        case claimedBy = "claimed_by"
        case sharedAt = "shared_at"
        case sharedBy = "shared_by"
    }
}

struct ClaimedByInfo: Codable, Equatable {
    let userId: UUID
    let username: String

    enum CodingKeys: String, CodingKey {
        case userId = "user_id"
        case username
    }
}

struct CircleEventResponse: Codable, Identifiable {
    let id: UUID
    let eventType: String
    let actorId: UUID?
    let actorUsername: String?
    let targetItemId: UUID?
    let targetItemName: String?
    let targetUserId: UUID?
    let targetUsername: String?
    let createdAt: Date

    enum CodingKeys: String, CodingKey {
        case id
        case eventType = "event_type"
        case actorId = "actor_id"
        case actorUsername = "actor_username"
        case targetItemId = "target_item_id"
        case targetItemName = "target_item_name"
        case targetUserId = "target_user_id"
        case targetUsername = "target_username"
        case createdAt = "created_at"
    }
}

struct FeedResponse: Codable {
    let events: [CircleEventResponse]
    let total: Int
    let page: Int
    let perPage: Int

    enum CodingKeys: String, CodingKey {
        case events
        case total
        case page
        case perPage = "per_page"
    }
}
