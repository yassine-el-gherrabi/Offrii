import Foundation

struct AppNotification: Codable, Identifiable, Equatable {
    let id: UUID
    let type: String
    let title: String
    let body: String
    let read: Bool
    let circleId: UUID?
    let itemId: UUID?
    let actorId: UUID?
    let actorName: String?
    let createdAt: Date

    enum CodingKeys: String, CodingKey {
        case id, type, title, body, read
        case circleId = "circle_id"
        case itemId = "item_id"
        case actorId = "actor_id"
        case actorName = "actor_name"
        case createdAt = "created_at"
    }

    /// Client-side localized title (uses push.{type}.title key, fallback to stored title)
    var localizedTitle: String {
        let key = "push.\(type).title"
        let localized = NSLocalizedString(key, comment: "")
        return localized != key ? localized : title
    }

    /// Client-side localized body (uses notif.{type} key with actor name, fallback to stored body)
    var localizedBody: String {
        let key = "notif.\(type)"
        let localized = NSLocalizedString(key, comment: "")
        if localized == key {
            return body
        }
        if let name = actorName {
            return String(format: localized, name)
        }
        return localized.replacingOccurrences(of: "%@", with: "")
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
        case "wish_message":
            return "message.fill"
        default:
            return "bell.fill"
        }
    }
}

struct UnreadCountResponse: Codable {
    let count: Int
}
