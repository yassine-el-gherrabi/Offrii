import Foundation

struct FriendResponse: Codable, Identifiable {
    let userId: UUID
    let username: String
    let displayName: String?
    let since: Date
    var id: UUID { userId }

    enum CodingKeys: String, CodingKey {
        case userId = "user_id"
        case username
        case displayName = "display_name"
        case since
    }
}

struct FriendRequestResponse: Codable, Identifiable {
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

struct SentFriendRequestResponse: Codable, Identifiable {
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
    var id: String { username }

    enum CodingKeys: String, CodingKey {
        case username
        case displayName = "display_name"
    }
}
