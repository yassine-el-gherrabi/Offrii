import Foundation

struct AppNotification: Codable, Identifiable {
    let id: UUID
    let type: String
    let title: String
    let body: String
    let read: Bool
    let circleId: UUID?
    let itemId: UUID?
    let actorId: UUID?
    let createdAt: Date

    enum CodingKeys: String, CodingKey {
        case id, type, title, body, read
        case circleId = "circle_id"
        case itemId = "item_id"
        case actorId = "actor_id"
        case createdAt = "created_at"
    }

    var icon: String {
        switch type {
        case "friend_request", "friend_accepted", "friend_activity":
            return "person.badge.plus"
        case "circle_added", "circle_member_joined", "circle_activity":
            return "person.2.fill"
        case "item_shared":
            return "square.and.arrow.up"
        case "item_claimed":
            return "gift.fill"
        case "item_unclaimed":
            return "gift"
        case "item_received":
            return "checkmark.circle.fill"
        default:
            return "bell.fill"
        }
    }
}

struct UnreadCountResponse: Codable {
    let count: Int
}
