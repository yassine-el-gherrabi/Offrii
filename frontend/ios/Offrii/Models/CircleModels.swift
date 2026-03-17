import Foundation

struct CircleMember: Codable, Identifiable, Equatable {
    let userId: UUID
    let username: String
    let displayName: String?
    let avatarUrl: String?
    let role: String
    let joinedAt: Date
    var id: UUID { userId }

    enum CodingKeys: String, CodingKey {
        case userId = "user_id"
        case username
        case displayName = "display_name"
        case avatarUrl = "avatar_url"
        case role
        case joinedAt = "joined_at"
    }
}

struct CircleDetailResponse: Codable {
    let id: UUID
    let name: String?
    let isDirect: Bool
    let ownerId: UUID
    let imageUrl: String?
    var members: [CircleMember]
    let createdAt: Date

    enum CodingKeys: String, CodingKey {
        case id
        case name
        case isDirect = "is_direct"
        case ownerId = "owner_id"
        case imageUrl = "image_url"
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
    let categoryIcon: String?
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
    let sharedByName: String?
    let sharedByAvatarUrl: String?

    enum CodingKeys: String, CodingKey {
        case id, name, description, url, priority, status, links
        case estimatedPrice = "estimated_price"
        case categoryId = "category_id"
        case categoryIcon = "category_icon"
        case isClaimed = "is_claimed"
        case claimedBy = "claimed_by"
        case imageUrl = "image_url"
        case ogImageUrl = "og_image_url"
        case ogTitle = "og_title"
        case ogSiteName = "og_site_name"
        case sharedAt = "shared_at"
        case sharedBy = "shared_by"
        case sharedByName = "shared_by_name"
        case sharedByAvatarUrl = "shared_by_avatar_url"
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        id = try container.decode(UUID.self, forKey: .id)
        name = try container.decode(String.self, forKey: .name)
        description = try container.decodeIfPresent(String.self, forKey: .description)
        url = try container.decodeIfPresent(String.self, forKey: .url)
        priority = try container.decode(Int16.self, forKey: .priority)
        categoryId = try container.decodeIfPresent(UUID.self, forKey: .categoryId)
        categoryIcon = try container.decodeIfPresent(String.self, forKey: .categoryIcon)
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
        sharedByName = try container.decodeIfPresent(String.self, forKey: .sharedByName)
        sharedByAvatarUrl = try container.decodeIfPresent(String.self, forKey: .sharedByAvatarUrl)

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
    let displayName: String?

    /// Display name with username fallback
    var name: String {
        displayName ?? username
    }

    enum CodingKeys: String, CodingKey {
        case userId = "user_id"
        case username
        case displayName = "display_name"
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

struct CircleInviteResponse: Codable, Identifiable {
    let id: UUID
    let token: String
    let circleId: UUID
    let expiresAt: Date
    let maxUses: Int
    let useCount: Int
    let createdAt: Date

    enum CodingKeys: String, CodingKey {
        case id, token
        case circleId = "circle_id"
        case expiresAt = "expires_at"
        case maxUses = "max_uses"
        case useCount = "use_count"
        case createdAt = "created_at"
    }
}

struct JoinCircleResponse: Codable {
    let circleId: UUID
    let circleName: String?

    enum CodingKeys: String, CodingKey {
        case circleId = "circle_id"
        case circleName = "circle_name"
    }
}

struct ReservationResponse: Codable, Identifiable {
    let itemId: UUID
    let itemName: String
    let itemImageUrl: String?
    let itemEstimatedPrice: Decimal?
    let itemStatus: String
    let ownerName: String
    let ownerAvatarUrl: String?
    let circleId: UUID
    let circleName: String?
    let claimedAt: Date

    var id: UUID { itemId }

    enum CodingKeys: String, CodingKey {
        case itemName = "item_name"
        case itemImageUrl = "item_image_url"
        case itemEstimatedPrice = "item_estimated_price"
        case itemStatus = "item_status"
        case ownerName = "owner_name"
        case ownerAvatarUrl = "owner_avatar_url"
        case circleId = "circle_id"
        case circleName = "circle_name"
        case claimedAt = "claimed_at"
        case itemId = "item_id"
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        itemId = try container.decode(UUID.self, forKey: .itemId)
        itemName = try container.decode(String.self, forKey: .itemName)
        itemImageUrl = try container.decodeIfPresent(String.self, forKey: .itemImageUrl)
        itemStatus = try container.decode(String.self, forKey: .itemStatus)
        ownerName = try container.decode(String.self, forKey: .ownerName)
        ownerAvatarUrl = try container.decodeIfPresent(String.self, forKey: .ownerAvatarUrl)
        circleId = try container.decode(UUID.self, forKey: .circleId)
        circleName = try container.decodeIfPresent(String.self, forKey: .circleName)
        claimedAt = try container.decode(Date.self, forKey: .claimedAt)
        // Handle price as string or number
        if let str = try? container.decodeIfPresent(String.self, forKey: .itemEstimatedPrice) {
            itemEstimatedPrice = Decimal(string: str)
        } else {
            itemEstimatedPrice = try? container.decodeIfPresent(Decimal.self, forKey: .itemEstimatedPrice)
        }
    }
}
