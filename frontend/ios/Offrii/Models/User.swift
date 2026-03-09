import Foundation

struct User: Codable, Identifiable, Equatable {
    let id: UUID
    let email: String
    let username: String
    let displayName: String?
    let reminderFreq: String
    let reminderTime: String
    let timezone: String
    let locale: String
    let createdAt: Date
    let updatedAt: Date

    enum CodingKeys: String, CodingKey {
        case id, email, username
        case displayName = "display_name"
        case reminderFreq = "reminder_freq"
        case reminderTime = "reminder_time"
        case timezone, locale
        case createdAt = "created_at"
        case updatedAt = "updated_at"
    }
}
