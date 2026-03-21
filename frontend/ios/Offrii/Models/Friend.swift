import Foundation

struct FriendResponse: Codable, Identifiable, Equatable, Sendable {
    let userId: UUID
    let username: String
    let displayName: String?
    let since: Date
    let sharedItemCount: Int
    var id: UUID { userId }

    enum CodingKeys: String, CodingKey {
        case userId = "user_id"
        case username
        case displayName = "display_name"
        case since
        case sharedItemCount = "shared_item_count"
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        userId = try container.decode(UUID.self, forKey: .userId)
        username = try container.decode(String.self, forKey: .username)
        displayName = try container.decodeIfPresent(String.self, forKey: .displayName)
        since = try container.decode(Date.self, forKey: .since)
        sharedItemCount = try container.decodeIfPresent(Int.self, forKey: .sharedItemCount) ?? 0
    }
}

struct FriendRequestResponse: Codable, Identifiable, Equatable, Sendable {
    let id: UUID
    let fromUserId: UUID
    let fromUsername: String
    let fromDisplayName: String?
    let status: String
    let createdAt: Date

    enum CodingKeys: String, CodingKey {
        case id
        case fromUserId = "from_user_id"
        case fromUsername = "from_username"
        case fromDisplayName = "from_display_name"
        case status
        case createdAt = "created_at"
    }
}

struct SentFriendRequestResponse: Codable, Identifiable, Equatable, Sendable {
    let id: UUID
    let toUserId: UUID
    let toUsername: String
    let toDisplayName: String?
    let status: String
    let createdAt: Date

    enum CodingKeys: String, CodingKey {
        case id
        case toUserId = "to_user_id"
        case toUsername = "to_username"
        case toDisplayName = "to_display_name"
        case status
        case createdAt = "created_at"
    }
}

struct UserSearchResult: Codable, Identifiable {
    let username: String
    let displayName: String?
    let isFriend: Bool?
    let isPending: Bool?
    var id: String { username }

    enum CodingKeys: String, CodingKey {
        case username
        case displayName = "display_name"
        case isFriend = "is_friend"
        case isPending = "is_pending"
    }
}
