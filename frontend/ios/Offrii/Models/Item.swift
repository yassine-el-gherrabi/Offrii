import Foundation

struct Item: Codable, Identifiable, Equatable {
    let id: UUID
    let name: String
    let description: String?
    let url: String?
    let estimatedPrice: Decimal?
    let priority: Int
    let categoryId: UUID?
    let status: String
    let purchasedAt: Date?
    let createdAt: Date
    let updatedAt: Date
    let isClaimed: Bool

    enum CodingKeys: String, CodingKey {
        case id, name, description, url, priority, status
        case estimatedPrice = "estimated_price"
        case categoryId = "category_id"
        case purchasedAt = "purchased_at"
        case createdAt = "created_at"
        case updatedAt = "updated_at"
        case isClaimed = "is_claimed"
    }

    var priorityLabel: String {
        switch priority {
        case 1: return String(localized: "priority.low")
        case 2: return String(localized: "priority.medium")
        case 3: return String(localized: "priority.high")
        default: return String(localized: "priority.medium")
        }
    }

    var isActive: Bool { status == "active" }
    var isPurchased: Bool { status == "purchased" }
}
