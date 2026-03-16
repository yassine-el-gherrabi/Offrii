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
    let avatarUrl: String?
    let reminderFreq: String?
    let reminderTime: String?
    let timezone: String?
    let locale: String?
    let createdAt: Date
    let updatedAt: Date?

    enum CodingKeys: String, CodingKey {
        case id, email, username, timezone, locale
        case displayName = "display_name"
        case avatarUrl = "avatar_url"
        case reminderFreq = "reminder_freq"
        case reminderTime = "reminder_time"
        case createdAt = "created_at"
        case updatedAt = "updated_at"
    }

    func toUser() -> User {
        User(
            id: id, email: email, username: username,
            displayName: displayName, avatarUrl: avatarUrl,
            reminderFreq: reminderFreq ?? "never",
            reminderTime: reminderTime ?? "09:00",
            timezone: timezone ?? TimeZone.current.identifier,
            locale: locale ?? Locale.current.language.languageCode?.identifier ?? "fr",
            createdAt: createdAt, updatedAt: updatedAt ?? createdAt
        )
    }
}

struct ItemsListResponse: Codable {
    let data: [Item]
    let pagination: PaginationResponse

    var items: [Item] { data }
    var total: Int { pagination.total }
}

struct PaginationResponse: Codable {
    let total: Int
    let page: Int
    let limit: Int
    let totalPages: Int
    let hasMore: Bool

    enum CodingKeys: String, CodingKey {
        case total, page, limit
        case totalPages = "total_pages"
        case hasMore = "has_more"
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

struct ShareLinkResponse: Codable, Identifiable {
    let id: UUID
    let token: String
    let url: String?
    let label: String?
    let permissions: String?
    let scope: String?
    let isActive: Bool?
    let createdAt: Date
    let expiresAt: Date?

    enum CodingKeys: String, CodingKey {
        case id, token, url, label, permissions, scope
        case isActive = "is_active"
        case createdAt = "created_at"
        case expiresAt = "expires_at"
    }

    /// Display URL — always provided by backend, fallback to token-based URL
    var displayUrl: String {
        url ?? "https://offrii.com/shared/\(token)"
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
