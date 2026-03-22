import Foundation
import SwiftUI

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
    let emailVerified: Bool?
    let usernameCustomized: Bool?
    let createdAt: Date
    let updatedAt: Date?

    enum CodingKeys: String, CodingKey {
        case id, email, username
        case displayName = "display_name"
        case avatarUrl = "avatar_url"
        case emailVerified = "email_verified"
        case usernameCustomized = "username_customized"
        case createdAt = "created_at"
        case updatedAt = "updated_at"
    }

    func toUser() -> User {
        User(
            id: id, email: email, username: username,
            displayName: displayName, avatarUrl: avatarUrl,
            emailVerified: emailVerified,
            usernameCustomized: usernameCustomized,
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

struct CategoryResponse: Codable, Equatable, CategoryChipItem {
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

    // MARK: - CategoryChipItem

    var chipLabel: String { name }
    var chipIcon: String { CategoryStyle(icon: icon).sfSymbol }
    var chipColor: Color { CategoryStyle(icon: icon).chipColor }
}

struct ShareLinkResponse: Codable, Identifiable, Equatable, Sendable {
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
    let profile: UserProfileResponse
    let items: [ExportItem]
    let categories: [CategoryResponse]
    let circles: [ExportCircle]
    let friends: [ExportFriend]
    let communityWishes: [ExportWish]
    let wishMessages: [ExportMessage]
    let exportedAt: Date

    enum CodingKeys: String, CodingKey {
        case profile, items, categories, circles, friends
        case communityWishes = "community_wishes"
        case wishMessages = "wish_messages"
        case exportedAt = "exported_at"
    }
}

struct ExportItem: Codable {
    let id: UUID
    let name: String
    let description: String?
    let estimatedPrice: String?
    let priority: Int
    let categoryId: UUID?
    let status: String
    let isPrivate: Bool
    let imageUrl: String?
    let links: [String]?
    let createdAt: Date

    enum CodingKeys: String, CodingKey {
        case id, name, description, priority, status, links
        case estimatedPrice = "estimated_price"
        case categoryId = "category_id"
        case isPrivate = "is_private"
        case imageUrl = "image_url"
        case createdAt = "created_at"
    }
}

struct ExportCircle: Codable {
    let id: UUID
    let name: String?
    let isDirect: Bool
    let memberCount: Int

    enum CodingKeys: String, CodingKey {
        case id, name
        case isDirect = "is_direct"
        case memberCount = "member_count"
    }
}

struct ExportFriend: Codable {
    let userId: UUID
    let username: String
    let displayName: String?

    enum CodingKeys: String, CodingKey {
        case username
        case userId = "user_id"
        case displayName = "display_name"
    }
}

struct ExportWish: Codable {
    let id: UUID
    let title: String
    let description: String?
    let category: String
    let status: String

    enum CodingKeys: String, CodingKey {
        case id, title, description, category, status
    }
}

struct ExportMessage: Codable {
    let id: UUID
    let wishId: UUID
    let body: String
    let createdAt: Date

    enum CodingKeys: String, CodingKey {
        case id, body
        case wishId = "wish_id"
        case createdAt = "created_at"
    }
}

struct APIErrorResponse: Codable {
    let error: APIErrorDetail
}

struct APIErrorDetail: Codable {
    let code: String
    let message: String
}
