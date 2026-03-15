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
    let imageUrl: String?
    let links: [String]?
    let ogImageUrl: String?
    let ogTitle: String?
    let ogSiteName: String?
    let sharedAt: Date
    let sharedBy: UUID

    enum CodingKeys: String, CodingKey {
        case id, name, description, url, priority, status, links
        case estimatedPrice = "estimated_price"
        case categoryId = "category_id"
        case isClaimed = "is_claimed"
        case claimedBy = "claimed_by"
        case imageUrl = "image_url"
        case ogImageUrl = "og_image_url"
        case ogTitle = "og_title"
        case ogSiteName = "og_site_name"
        case sharedAt = "shared_at"
        case sharedBy = "shared_by"
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        id = try container.decode(UUID.self, forKey: .id)
        name = try container.decode(String.self, forKey: .name)
        description = try container.decodeIfPresent(String.self, forKey: .description)
        url = try container.decodeIfPresent(String.self, forKey: .url)
        priority = try container.decode(Int16.self, forKey: .priority)
        categoryId = try container.decodeIfPresent(UUID.self, forKey: .categoryId)
        status = try container.decode(String.self, forKey: .status)
        isClaimed = try container.decode(Bool.self, forKey: .isClaimed)
        claimedBy = try container.decodeIfPresent(ClaimedByInfo.self, forKey: .claimedBy)
        imageUrl = try container.decodeIfPresent(String.self, forKey: .imageUrl)
        links = try container.decodeIfPresent([String].self, forKey: .links)
        ogImageUrl = try container.decodeIfPresent(String.self, forKey: .ogImageUrl)
        ogTitle = try container.decodeIfPresent(String.self, forKey: .ogTitle)
        ogSiteName = try container.decodeIfPresent(String.self, forKey: .ogSiteName)
        sharedAt = try container.decode(Date.self, forKey: .sharedAt)
        sharedBy = try container.decode(UUID.self, forKey: .sharedBy)

        if let stringValue = try? container.decodeIfPresent(String.self, forKey: .estimatedPrice) {
            estimatedPrice = Decimal(string: stringValue)
        } else {
            estimatedPrice = try? container.decodeIfPresent(Decimal.self, forKey: .estimatedPrice)
        }
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
    let data: [CircleEventResponse]
    let pagination: PaginationMeta

    var events: [CircleEventResponse] { data }
    var total: Int { pagination.total }
    var page: Int { pagination.page }
}
