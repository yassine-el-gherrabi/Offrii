import Foundation

// MARK: - HTTP Method

enum HTTPMethod: String {
    case GET
    case POST
    case PUT
    case PATCH
    case DELETE
}

// MARK: - API Endpoint

/// Defines every REST endpoint exposed by the Rust/Axum backend.
///
/// Route paths are derived from the backend handler routers:
/// - `/auth/*`         - Authentication
/// - `/items/*`        - Wishlist items
/// - `/categories/*`   - Item categories
/// - `/users/*`        - User profile
/// - `/push-tokens/*`  - APNs push tokens
/// - `/share-links/*`  - Share link management
/// - `/shared/*`       - Public shared view + claim/unclaim
enum APIEndpoint {

    // MARK: Auth

    case register(RegisterBody)
    case login(LoginBody)
    case refresh(RefreshBody)
    case logout
    case changePassword(ChangePasswordBody)
    case forgotPassword(ForgotPasswordBody)
    case resetPassword(ResetPasswordBody)

    // MARK: Items

    case listItems(ListItemsQuery)
    case createItem(CreateItemBody)
    case getItem(id: UUID)
    case updateItem(id: UUID, body: UpdateItemBody)
    case deleteItem(id: UUID)
    case claimItem(id: UUID)
    case unclaimItem(id: UUID)

    // MARK: Categories

    case listCategories
    case createCategory(CreateCategoryBody)

    // MARK: Users

    case getProfile
    case updateProfile(UpdateProfileBody)
    case deleteAccount
    case exportData

    // MARK: Push Tokens

    case registerToken(RegisterPushTokenBody)
    case unregisterToken(token: String)

    // MARK: Share Links

    case createShareLink(CreateShareLinkBody?)
    case listShareLinks
    case deleteShareLink(id: UUID)

    // MARK: Shared (Public)

    case getSharedView(token: String)
    case claimViaShare(token: String, itemId: UUID)
    case unclaimViaShare(token: String, itemId: UUID)

    // MARK: Circles

    case listCircles
    case createCircle(CreateCircleBody)
    case getCircle(id: UUID)
    case updateCircle(id: UUID, body: UpdateCircleBody)
    case deleteCircle(id: UUID)
    case createDirectCircle(userId: UUID)
    case addMemberToCircle(circleId: UUID, body: AddMemberBody)
    case removeMember(circleId: UUID, userId: UUID)
    case shareItemToCircle(circleId: UUID, body: ShareItemBody)
    case listCircleItems(circleId: UUID)
    case unshareItem(circleId: UUID, itemId: UUID)
    case getCircleFeed(circleId: UUID, page: Int, perPage: Int)

    // MARK: Friends

    case searchUsers(query: String)
    case sendFriendRequest(SendFriendRequestBody)
    case listPendingFriendRequests
    case listSentFriendRequests
    case acceptFriendRequest(id: UUID)
    case declineFriendRequest(id: UUID)
    case cancelFriendRequest(id: UUID)
    case listFriends
    case removeFriend(userId: UUID)
}

// MARK: - Endpoint Properties

extension APIEndpoint {

    /// The relative path for this endpoint (appended to the base URL).
    var path: String {
        switch self {
        // Auth
        case .register:                         return "/auth/register"
        case .login:                            return "/auth/login"
        case .refresh:                          return "/auth/refresh"
        case .logout:                           return "/auth/logout"
        case .changePassword:                   return "/auth/change-password"
        case .forgotPassword:                   return "/auth/forgot-password"
        case .resetPassword:                    return "/auth/reset-password"

        // Items
        case .listItems:                        return "/items"
        case .createItem:                       return "/items"
        case .getItem(let id):                  return "/items/\(id)"
        case .updateItem(let id, _):            return "/items/\(id)"
        case .deleteItem(let id):               return "/items/\(id)"
        case .claimItem(let id):                return "/items/\(id)/claim"
        case .unclaimItem(let id):              return "/items/\(id)/claim"

        // Categories
        case .listCategories:                   return "/categories"
        case .createCategory:                   return "/categories"

        // Users
        case .getProfile:                       return "/users/me"
        case .updateProfile:                    return "/users/me"
        case .deleteAccount:                    return "/users/me"
        case .exportData:                       return "/users/me/export"

        // Push Tokens
        case .registerToken:                    return "/push-tokens"
        case .unregisterToken(let token):       return "/push-tokens/\(token)"

        // Share Links
        case .createShareLink:                  return "/share-links"
        case .listShareLinks:                   return "/share-links"
        case .deleteShareLink(let id):          return "/share-links/\(id)"

        // Shared
        case .getSharedView(let token):                 return "/shared/\(token)"
        case .claimViaShare(let token, let itemId):     return "/shared/\(token)/items/\(itemId)/claim"
        case .unclaimViaShare(let token, let itemId):   return "/shared/\(token)/items/\(itemId)/claim"

        // Circles
        case .listCircles:                              return "/circles"
        case .createCircle:                             return "/circles"
        case .getCircle(let id):                        return "/circles/\(id)"
        case .updateCircle(let id, _):                  return "/circles/\(id)"
        case .deleteCircle(let id):                     return "/circles/\(id)"
        case .createDirectCircle(let userId):           return "/circles/direct/\(userId)"
        case .addMemberToCircle(let circleId, _):       return "/circles/\(circleId)/members"
        case .removeMember(let circleId, let userId):   return "/circles/\(circleId)/members/\(userId)"
        case .shareItemToCircle(let circleId, _):       return "/circles/\(circleId)/items"
        case .listCircleItems(let circleId):            return "/circles/\(circleId)/items"
        case .unshareItem(let circleId, let itemId):    return "/circles/\(circleId)/items/\(itemId)"
        case .getCircleFeed(let circleId, _, _):        return "/circles/\(circleId)/feed"

        // Friends
        case .searchUsers:                              return "/users/search"
        case .sendFriendRequest:                        return "/me/friend-requests"
        case .listPendingFriendRequests:                return "/me/friend-requests"
        case .listSentFriendRequests:                   return "/me/friend-requests/sent"
        case .acceptFriendRequest(let id):              return "/me/friend-requests/\(id)/accept"
        case .declineFriendRequest(let id):             return "/me/friend-requests/\(id)"
        case .cancelFriendRequest(let id):              return "/me/friend-requests/\(id)/cancel"
        case .listFriends:                              return "/me/friends"
        case .removeFriend(let userId):                 return "/me/friends/\(userId)"
        }
    }

    /// The HTTP method for this endpoint.
    var method: HTTPMethod {
        switch self {
        // Auth
        case .register:         return .POST
        case .login:            return .POST
        case .refresh:          return .POST
        case .logout:           return .POST
        case .changePassword:   return .POST
        case .forgotPassword:   return .POST
        case .resetPassword:    return .POST

        // Items
        case .listItems:        return .GET
        case .createItem:       return .POST
        case .getItem:          return .GET
        case .updateItem:       return .PUT
        case .deleteItem:       return .DELETE
        case .claimItem:        return .POST
        case .unclaimItem:      return .DELETE

        // Categories
        case .listCategories:   return .GET
        case .createCategory:   return .POST

        // Users
        case .getProfile:       return .GET
        case .updateProfile:    return .PATCH
        case .deleteAccount:    return .DELETE
        case .exportData:       return .GET

        // Push Tokens
        case .registerToken:    return .POST
        case .unregisterToken:  return .DELETE

        // Share Links
        case .createShareLink:  return .POST
        case .listShareLinks:   return .GET
        case .deleteShareLink:  return .DELETE

        // Shared
        case .getSharedView:    return .GET
        case .claimViaShare:    return .POST
        case .unclaimViaShare:  return .DELETE

        // Circles
        case .listCircles:          return .GET
        case .createCircle:         return .POST
        case .getCircle:            return .GET
        case .updateCircle:         return .PATCH
        case .deleteCircle:         return .DELETE
        case .createDirectCircle:   return .POST
        case .addMemberToCircle:    return .POST
        case .removeMember:         return .DELETE
        case .shareItemToCircle:    return .POST
        case .listCircleItems:      return .GET
        case .unshareItem:          return .DELETE
        case .getCircleFeed:        return .GET

        // Friends
        case .searchUsers:              return .GET
        case .sendFriendRequest:        return .POST
        case .listPendingFriendRequests: return .GET
        case .listSentFriendRequests:   return .GET
        case .acceptFriendRequest:      return .POST
        case .declineFriendRequest:     return .DELETE
        case .cancelFriendRequest:      return .DELETE
        case .listFriends:              return .GET
        case .removeFriend:             return .DELETE
        }
    }

    /// Whether this endpoint requires an `Authorization: Bearer` header.
    var requiresAuth: Bool {
        switch self {
        case .register, .login, .refresh, .forgotPassword, .resetPassword:
            return false
        case .getSharedView:
            return false
        default:
            return true
        }
    }

    /// URL query items for GET requests that accept query parameters.
    var queryItems: [URLQueryItem]? {
        switch self {
        case .listItems(let query):
            var items: [URLQueryItem] = []
            if let status = query.status {
                items.append(.init(name: "status", value: status))
            }
            if let categoryId = query.categoryId {
                items.append(.init(name: "category_id", value: categoryId.uuidString))
            }
            if let sort = query.sort {
                items.append(.init(name: "sort", value: sort))
            }
            if let order = query.order {
                items.append(.init(name: "order", value: order))
            }
            if let page = query.page {
                items.append(.init(name: "page", value: String(page)))
            }
            if let perPage = query.perPage {
                items.append(.init(name: "per_page", value: String(perPage)))
            }
            return items.isEmpty ? nil : items
        case .searchUsers(let query):
            return [URLQueryItem(name: "q", value: query)]
        case .getCircleFeed(_, let page, let perPage):
            return [
                URLQueryItem(name: "page", value: String(page)),
                URLQueryItem(name: "per_page", value: String(perPage)),
            ]
        default:
            return nil
        }
    }

    /// The JSON body for this endpoint, if any.
    var body: (any Encodable)? {
        switch self {
        case .register(let body):           return body
        case .login(let body):              return body
        case .refresh(let body):            return body
        case .changePassword(let body):     return body
        case .forgotPassword(let body):     return body
        case .resetPassword(let body):      return body
        case .createItem(let body):         return body
        case .updateItem(_, let body):      return body
        case .createCategory(let body):     return body
        case .updateProfile(let body):      return body
        case .registerToken(let body):      return body
        case .createShareLink(let body):    return body
        case .createCircle(let body):       return body
        case .updateCircle(_, let body):    return body
        case .addMemberToCircle(_, let body): return body
        case .shareItemToCircle(_, let body): return body
        case .sendFriendRequest(let body):  return body
        default:                            return nil
        }
    }

    // MARK: - Full URL

    /// The base URL read from `Info.plist` via the `API_BASE_URL` key.
    /// Set this value through an xcconfig file or build settings.
    static var baseURL: String {
        guard let url = Bundle.main.infoDictionary?["API_BASE_URL"] as? String,
              !url.isEmpty else {
            #if DEBUG
            return "http://localhost:3000"
            #else
            fatalError("API_BASE_URL is not configured in Info.plist")
            #endif
        }
        return url
    }

    /// Constructs the full `URL` by combining the base URL, path, and query parameters.
    var url: URL? {
        let urlString = Self.baseURL + path

        guard var components = URLComponents(string: urlString) else {
            return nil
        }

        if let queryItems {
            components.queryItems = queryItems
        }

        return components.url
    }
}

// MARK: - Request Body Types

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

struct CreateCategoryBody: Encodable {
    let name: String
    let icon: String?
}

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

struct RegisterPushTokenBody: Encodable {
    let token: String
    let platform: String
}

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
