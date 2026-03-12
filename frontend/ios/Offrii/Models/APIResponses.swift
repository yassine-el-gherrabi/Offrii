import Foundation

struct AuthResponse: Codable {
    let user: UserProfileResponse
    let tokens: AuthTokens
    let isNewUser: Bool

    enum CodingKeys: String, CodingKey {
        case user, tokens
        case isNewUser = "is_new_user"
    }
}

struct RefreshResponse: Codable {
    let tokens: AuthTokens
}

struct UserProfileResponse: Codable {
    let id: UUID
    let email: String
    let username: String
    let displayName: String?
    let reminderFreq: String?
    let reminderTime: String?
    let timezone: String?
    let locale: String?
    let createdAt: Date
    let updatedAt: Date?

    enum CodingKeys: String, CodingKey {
        case id, email, username, timezone, locale
        case displayName = "display_name"
        case reminderFreq = "reminder_freq"
        case reminderTime = "reminder_time"
        case createdAt = "created_at"
        case updatedAt = "updated_at"
    }

    func toUser() -> User {
        User(
            id: id, email: email, username: username, displayName: displayName,
            reminderFreq: reminderFreq ?? "never",
            reminderTime: reminderTime ?? "09:00",
            timezone: timezone ?? TimeZone.current.identifier,
            locale: locale ?? Locale.current.language.languageCode?.identifier ?? "fr",
            createdAt: createdAt, updatedAt: updatedAt ?? createdAt
        )
    }
}

struct ItemsListResponse: Codable {
    let items: [Item]
    let total: Int
    let page: Int
    let perPage: Int

    enum CodingKeys: String, CodingKey {
        case items, total, page
        case perPage = "per_page"
    }
}

struct CategoryResponse: Codable {
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

    func toCategory() -> Category {
        Category(id: id, name: name, icon: icon, isDefault: isDefault, position: position, createdAt: createdAt)
    }
}

struct ShareLinkResponse: Codable {
    let id: UUID
    let token: String
    let url: String
    let createdAt: Date
    let expiresAt: Date?

    enum CodingKeys: String, CodingKey {
        case id, token, url
        case createdAt = "created_at"
        case expiresAt = "expires_at"
    }
}

struct SharedViewResponse: Codable {
    let userUsername: String
    let items: [Item]

    enum CodingKeys: String, CodingKey {
        case items
        case userUsername = "user_username"
    }
}

struct UserDataExport: Codable {
    let user: UserProfileResponse
    let items: [Item]
    let categories: [CategoryResponse]
}

struct APIErrorResponse: Codable {
    let error: APIErrorDetail
}

struct APIErrorDetail: Codable {
    let code: String
    let message: String
}
