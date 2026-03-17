import Foundation

struct OffriiCircle: Codable, Identifiable, Equatable {
    let id: UUID
    let name: String?
    let isDirect: Bool
    let ownerId: UUID
    let imageUrl: String?
    let memberCount: Int
    let unreservedItemCount: Int
    let lastActivity: String?
    let lastActivityAt: Date?
    let memberNames: [String]
    let memberAvatars: [String?]
    let createdAt: Date

    enum CodingKeys: String, CodingKey {
        case id
        case name
        case isDirect = "is_direct"
        case ownerId = "owner_id"
        case imageUrl = "image_url"
        case memberCount = "member_count"
        case unreservedItemCount = "unreserved_item_count"
        case lastActivity = "last_activity"
        case lastActivityAt = "last_activity_at"
        case memberNames = "member_names"
        case memberAvatars = "member_avatars"
        case createdAt = "created_at"
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        id = try container.decode(UUID.self, forKey: .id)
        name = try container.decodeIfPresent(String.self, forKey: .name)
        isDirect = try container.decode(Bool.self, forKey: .isDirect)
        ownerId = try container.decode(UUID.self, forKey: .ownerId)
        imageUrl = try container.decodeIfPresent(String.self, forKey: .imageUrl)
        memberCount = try container.decode(Int.self, forKey: .memberCount)
        unreservedItemCount = try container.decodeIfPresent(Int.self, forKey: .unreservedItemCount) ?? 0
        lastActivity = try container.decodeIfPresent(String.self, forKey: .lastActivity)
        lastActivityAt = try container.decodeIfPresent(Date.self, forKey: .lastActivityAt)
        memberNames = try container.decodeIfPresent([String].self, forKey: .memberNames) ?? []
        memberAvatars = try container.decodeIfPresent([String?].self, forKey: .memberAvatars) ?? []
        createdAt = try container.decode(Date.self, forKey: .createdAt)
    }
}
