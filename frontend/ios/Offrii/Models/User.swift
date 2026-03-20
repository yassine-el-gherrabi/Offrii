import Foundation

struct User: Codable, Identifiable, Equatable {
    let id: UUID
    let email: String
    let username: String
    let displayName: String?
    let avatarUrl: String?
    let reminderFreq: String
    let reminderTime: String
    let timezone: String
    let locale: String
    let emailVerified: Bool?
    let usernameCustomized: Bool?
    let createdAt: Date
    let updatedAt: Date

    enum CodingKeys: String, CodingKey {
        case id, email, username
        case displayName = "display_name"
        case avatarUrl = "avatar_url"
        case reminderFreq = "reminder_freq"
        case reminderTime = "reminder_time"
        case timezone, locale
        case emailVerified = "email_verified"
        case usernameCustomized = "username_customized"
        case createdAt = "created_at"
        case updatedAt = "updated_at"
    }
}
