import Foundation

struct Category: Codable, Identifiable, Equatable {
    let id: UUID
    let name: String
    let icon: String?
    let isDefault: Bool
    let position: Int
    let createdAt: Date

    enum CodingKeys: String, CodingKey {
        case id, name, icon, position
        case isDefault = "is_default"
        case createdAt = "created_at"
    }
}
