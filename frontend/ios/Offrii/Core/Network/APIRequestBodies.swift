// swiftlint:disable file_length
import Foundation

// MARK: - Auth Body Types

struct RegisterBody: Encodable {
    let email: String
    let password: String
    let displayName: String?
    let termsAccepted: Bool

    enum CodingKeys: String, CodingKey {
        case email
        case password
        case displayName = "display_name"
        case termsAccepted = "terms_accepted"
    }
}

struct LoginBody: Encodable {
    let identifier: String
    let password: String
}

struct RefreshBody: Encodable {
    let refreshToken: String

    enum CodingKeys: String, CodingKey {
        case refreshToken = "refresh_token"
    }
}

struct ChangePasswordBody: Encodable {
    let currentPassword: String
    let newPassword: String

    enum CodingKeys: String, CodingKey {
        case currentPassword = "current_password"
        case newPassword = "new_password"
    }
}

struct ForgotPasswordBody: Encodable {
    let email: String
}

struct ResetPasswordBody: Encodable {
    let email: String
    let code: String
    let newPassword: String

    enum CodingKeys: String, CodingKey {
        case email
        case code
        case newPassword = "new_password"
    }
}

struct VerifyResetCodeBody: Encodable {
    let email: String
    let code: String
}

struct GoogleAuthBody: Encodable {
    let idToken: String
    let displayName: String?

    enum CodingKeys: String, CodingKey {
        case idToken = "id_token"
        case displayName = "display_name"
    }
}

struct AppleAuthBody: Encodable {
    let idToken: String
    let displayName: String?

    enum CodingKeys: String, CodingKey {
        case idToken = "id_token"
        case displayName = "display_name"
    }
}

struct VerifyEmailBody: Encodable {
    let token: String
}

// MARK: - Item Body Types

struct CreateItemBody: Encodable {
    let name: String
    let description: String?
    let url: String?
    let estimatedPrice: Decimal?
    let priority: Int16?
    let categoryId: UUID?
    let imageUrl: String?
    let links: [String]?
    let isPrivate: Bool?

    enum CodingKeys: String, CodingKey {
        case name, description, url, priority, links
        case estimatedPrice = "estimated_price"
        case categoryId = "category_id"
        case imageUrl = "image_url"
        case isPrivate = "is_private"
    }
}

struct UpdateItemBody: Encodable {
    let name: String?
    let description: String?
    let url: String?
    let estimatedPrice: Decimal?
    let priority: Int16?
    let categoryId: UUID?
    let status: String?
    /// `nil` = don't touch, `.some(nil)` = set to null, `.some("url")` = set value
    let imageUrl: String??
    let links: [String]?
    let isPrivate: Bool?

    enum CodingKeys: String, CodingKey {
        case name, description, url, priority, status, links
        case estimatedPrice = "estimated_price"
        case categoryId = "category_id"
        case imageUrl = "image_url"
        case isPrivate = "is_private"
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        try container.encodeIfPresent(name, forKey: .name)
        try container.encodeIfPresent(description, forKey: .description)
        try container.encodeIfPresent(url, forKey: .url)
        try container.encodeIfPresent(estimatedPrice, forKey: .estimatedPrice)
        try container.encodeIfPresent(priority, forKey: .priority)
        try container.encodeIfPresent(categoryId, forKey: .categoryId)
        try container.encodeIfPresent(status, forKey: .status)
        // Double-optional: nil = skip, .some(nil) = encode null, .some(url) = encode url
        if let imageUrl {
            try container.encode(imageUrl, forKey: .imageUrl)
        }
        try container.encodeIfPresent(links, forKey: .links)
        try container.encodeIfPresent(isPrivate, forKey: .isPrivate)
    }
}

struct ListItemsQuery {
    let status: String?
    let categoryId: UUID?
    let sort: String?
    let order: String?
    let page: Int?
    let perPage: Int?

    init(
        status: String? = nil,
        categoryId: UUID? = nil,
        sort: String? = nil,
        order: String? = nil,
        page: Int? = nil,
        perPage: Int? = nil
    ) {
        self.status = status
        self.categoryId = categoryId
        self.sort = sort
        self.order = order
        self.page = page
        self.perPage = perPage
    }
}

// MARK: - User Body Types

struct UpdateProfileBody: Encodable {
    let displayName: String?
    let username: String?
    /// `nil` = don't touch, `.some(nil)` = clear, `.some("url")` = set
    let avatarUrl: String??

    enum CodingKeys: String, CodingKey {
        case displayName = "display_name"
        case username
        case avatarUrl = "avatar_url"
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        try container.encodeIfPresent(displayName, forKey: .displayName)
        try container.encodeIfPresent(username, forKey: .username)
        if let avatarUrl {
            try container.encode(avatarUrl, forKey: .avatarUrl)
        }
    }
}

struct ChangeEmailBody: Encodable {
    let newEmail: String

    enum CodingKeys: String, CodingKey {
        case newEmail = "new_email"
    }
}

// MARK: - Push Token Body Types

struct RegisterPushTokenBody: Encodable {
    let token: String
    let platform: String
}

// MARK: - Share Link Body Types

struct CreateShareLinkBody: Encodable {
    let expiresAt: String?
    let label: String?
    let permissions: String?   // "view_only" or "view_and_claim"
    let scope: String?         // "all", "category", "selection"
    let scopeData: ScopeData?

    enum CodingKeys: String, CodingKey {
        case expiresAt = "expires_at"
        case label, permissions, scope
        case scopeData = "scope_data"
    }

    /// Convenience init for sharing the whole list
    static func shareAll() -> CreateShareLinkBody {
        CreateShareLinkBody(expiresAt: nil, label: nil, permissions: "view_and_claim", scope: "all", scopeData: nil)
    }

    /// Convenience init for sharing a single item
    static func shareItem(id: UUID) -> CreateShareLinkBody {
        CreateShareLinkBody(
            expiresAt: nil,
            label: nil,
            permissions: "view_and_claim",
            scope: "selection",
            scopeData: ScopeData(categoryId: nil, itemIds: [id.uuidString])
        )
    }
}

struct ScopeData: Encodable {
    let categoryId: String?
    let itemIds: [String]?

    enum CodingKeys: String, CodingKey {
        case categoryId = "category_id"
        case itemIds = "item_ids"
    }
}

struct UpdateShareLinkBody: Encodable {
    let label: String?
    let permissions: String?
    let isActive: Bool?
    let expiresAt: String?

    enum CodingKeys: String, CodingKey {
        case label, permissions
        case isActive = "is_active"
        case expiresAt = "expires_at"
    }
}

// MARK: - Batch Delete

struct BatchDeleteItemsBody: Encodable {
    let ids: [UUID]
}

// MARK: - Circle Body Types

struct CreateCircleBody: Encodable {
    let name: String
}

struct UpdateCircleBody: Encodable {
    let name: String
    /// `nil` = don't touch, `.some(nil)` = clear, `.some("url")` = set
    let imageUrl: String??

    enum CodingKeys: String, CodingKey {
        case name
        case imageUrl = "image_url"
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        try container.encode(name, forKey: .name)
        if let imageUrl {
            try container.encode(imageUrl, forKey: .imageUrl)
        }
    }
}

struct CreateCircleInviteBody: Encodable {
    let maxUses: Int?
    let expiresInHours: Int?

    enum CodingKeys: String, CodingKey {
        case maxUses = "max_uses"
        case expiresInHours = "expires_in_hours"
    }
}

struct TransferOwnershipBody: Encodable {
    let userId: UUID

    enum CodingKeys: String, CodingKey {
        case userId = "user_id"
    }
}

struct AddMemberBody: Encodable {
    let userId: UUID

    enum CodingKeys: String, CodingKey {
        case userId = "user_id"
    }
}

struct ShareItemBody: Encodable {
    let itemId: UUID

    enum CodingKeys: String, CodingKey {
        case itemId = "item_id"
    }
}

struct BatchShareBody: Encodable {
    let itemIds: [UUID]

    enum CodingKeys: String, CodingKey {
        case itemIds = "item_ids"
    }
}

struct SetShareRuleBody: Encodable {
    let shareMode: String
    let categoryIds: [UUID]

    enum CodingKeys: String, CodingKey {
        case shareMode = "share_mode"
        case categoryIds = "category_ids"
    }
}

// MARK: - Friend Body Types

struct SendFriendRequestBody: Encodable {
    let username: String
}

// MARK: - Community Wish Body Types

struct ListCommunityWishesQuery {
    let category: String?
    let page: Int
    let limit: Int

    init(category: String? = nil, page: Int = 1, limit: Int = 20) {
        self.category = category
        self.page = page
        self.limit = limit
    }
}

struct CreateCommunityWishBody: Encodable {
    let title: String
    let description: String?
    let category: String
    let isAnonymous: Bool
    let imageUrl: String?
    let links: [String]?

    enum CodingKeys: String, CodingKey {
        case title, description, category, links
        case isAnonymous = "is_anonymous"
        case imageUrl = "image_url"
    }
}

struct UpdateCommunityWishBody: Encodable {
    let title: String?
    let description: String?
    let category: String?
    let imageUrl: String?
    let links: [String]?

    enum CodingKeys: String, CodingKey {
        case title, description, category, links
        case imageUrl = "image_url"
    }
}

struct ReportCommunityWishBody: Encodable {
    let reason: String?
    let details: String?
}

// MARK: - Wish Message Body Types

struct ListWishMessagesQuery {
    let page: Int
    let limit: Int

    init(page: Int = 1, limit: Int = 20) {
        self.page = page
        self.limit = limit
    }
}

struct SendWishMessageBody: Encodable {
    let body: String
}
