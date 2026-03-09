import Foundation

struct OffriiCircle: Codable, Identifiable, Equatable {
    let id: UUID
    let name: String?
    let isDirect: Bool
    let ownerId: UUID
    let memberCount: Int
    let createdAt: Date

    enum CodingKeys: String, CodingKey {
        case id
        case name
        case isDirect = "is_direct"
        case ownerId = "owner_id"
        case memberCount = "member_count"
        case createdAt = "created_at"
    }
}
