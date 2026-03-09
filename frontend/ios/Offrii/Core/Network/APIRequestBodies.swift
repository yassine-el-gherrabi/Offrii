import Foundation

// MARK: - Auth Body Types

struct RegisterBody: Encodable {
    let email: String
    let password: String
    let displayName: String?

    enum CodingKeys: String, CodingKey {
        case email
        case password
        case displayName = "display_name"
    }
}

struct LoginBody: Encodable {
    let email: String
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

// MARK: - Item Body Types

struct CreateItemBody: Encodable {
    let name: String
    let description: String?
    let url: String?
    let estimatedPrice: Decimal?
    let priority: Int16?
    let categoryId: UUID?

    enum CodingKeys: String, CodingKey {
        case name
        case description
        case url
        case estimatedPrice = "estimated_price"
        case priority
        case categoryId = "category_id"
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

    enum CodingKeys: String, CodingKey {
        case name
        case description
        case url
        case estimatedPrice = "estimated_price"
        case priority
        case categoryId = "category_id"
        case status
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

// MARK: - Category Body Types

struct CreateCategoryBody: Encodable {
    let name: String
    let icon: String?
}

// MARK: - User Body Types

struct UpdateProfileBody: Encodable {
    let displayName: String?
    let username: String?
    let reminderFreq: String?
    let reminderTime: String?
    let timezone: String?
    let locale: String?

    enum CodingKeys: String, CodingKey {
        case displayName = "display_name"
        case username
        case reminderFreq = "reminder_freq"
        case reminderTime = "reminder_time"
        case timezone
        case locale
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

    enum CodingKeys: String, CodingKey {
        case expiresAt = "expires_at"
    }
}

// MARK: - Circle Body Types

struct CreateCircleBody: Encodable {
    let name: String
}

struct UpdateCircleBody: Encodable {
    let name: String
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

// MARK: - Friend Body Types

struct SendFriendRequestBody: Encodable {
    let username: String
}
