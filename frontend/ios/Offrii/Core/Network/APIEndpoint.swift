// swiftlint:disable file_length
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
    case verifyResetCode(VerifyResetCodeBody)
    case resetPassword(ResetPasswordBody)
    case googleAuth(GoogleAuthBody)
    case appleAuth(AppleAuthBody)
    case verifyEmail(VerifyEmailBody)
    case resendVerification

    // MARK: Items

    case listItems(ListItemsQuery)
    case createItem(CreateItemBody)
    case getItem(id: UUID)
    case updateItem(id: UUID, body: UpdateItemBody)
    case deleteItem(id: UUID)
    case claimItem(id: UUID)
    case unclaimItem(id: UUID)
    case uploadImage
    case batchDeleteItems(BatchDeleteItemsBody)
    case ownerUnclaimWeb(id: UUID)

    // MARK: Categories

    case listCategories

    // MARK: Users

    case getProfile
    case updateProfile(UpdateProfileBody)
    case deleteAccount
    case exportData
    case requestEmailChange(ChangeEmailBody)

    // MARK: Push Tokens

    case registerToken(RegisterPushTokenBody)
    case unregisterToken(token: String)

    // MARK: Share Links

    case createShareLink(CreateShareLinkBody?)
    case listShareLinks
    case deleteShareLink(id: UUID)
    case updateShareLink(id: UUID, body: UpdateShareLinkBody)

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
    case batchShareItems(circleId: UUID, body: BatchShareBody)
    case listCircleItems(circleId: UUID)
    case getCircleItem(circleId: UUID, itemId: UUID)
    case unshareItem(circleId: UUID, itemId: UUID)
    case getShareRule(circleId: UUID)
    case setShareRule(circleId: UUID, body: SetShareRuleBody)
    case getCircleFeed(circleId: UUID, page: Int, perPage: Int)
    case transferCircleOwnership(circleId: UUID, body: TransferOwnershipBody)
    case listReservations
    case listMyShareRules
    case createCircleInvite(circleId: UUID, body: CreateCircleInviteBody?)
    case listCircleInvites(circleId: UUID)
    case revokeCircleInvite(circleId: UUID, inviteId: UUID)
    case joinCircleViaInvite(token: String)

    // MARK: Notifications
    case listNotifications(page: Int, limit: Int)
    case markNotificationRead(id: UUID)
    case markAllNotificationsRead
    case deleteNotification(id: UUID)
    case unreadNotificationCount

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

    // MARK: Community Wishes

    case listCommunityWishes(ListCommunityWishesQuery)
    case createCommunityWish(CreateCommunityWishBody)
    case getCommunityWish(id: UUID)
    case updateCommunityWish(id: UUID, body: UpdateCommunityWishBody)
    case closeCommunityWish(id: UUID)
    case deleteCommunityWish(id: UUID)
    case reopenCommunityWish(id: UUID)
    case offerCommunityWish(id: UUID)
    case withdrawOfferCommunityWish(id: UUID)
    case rejectOfferCommunityWish(id: UUID)
    case confirmCommunityWish(id: UUID)
    case reportCommunityWish(id: UUID, body: ReportCommunityWishBody)
    case blockCommunityWish(id: UUID)
    case unblockCommunityWish(id: UUID)
    case listMyCommunityWishes
    case listMyCommunityOffers
    case listRecentFulfilled

    // MARK: Wish Messages

    case listWishMessages(wishId: UUID, query: ListWishMessagesQuery)
    case sendWishMessage(wishId: UUID, body: SendWishMessageBody)
}

// MARK: - Endpoint Properties

extension APIEndpoint {

    /// The relative path for this endpoint (appended to the base URL).
    var path: String {
        switch self {
        // Auth
        case .register:                         return "/v1/auth/register"
        case .login:                            return "/v1/auth/login"
        case .refresh:                          return "/v1/auth/refresh"
        case .logout:                           return "/v1/auth/logout"
        case .changePassword:                   return "/v1/auth/change-password"
        case .forgotPassword:                   return "/v1/auth/forgot-password"
        case .verifyResetCode:                  return "/v1/auth/verify-reset-code"
        case .resetPassword:                    return "/v1/auth/reset-password"
        case .googleAuth:                       return "/v1/auth/google"
        case .appleAuth:                        return "/v1/auth/apple"
        case .verifyEmail:                      return "/v1/auth/verify-email"
        case .resendVerification:               return "/v1/auth/resend-verification"

        // Items
        case .listItems:                        return "/v1/items"
        case .createItem:                       return "/v1/items"
        case .getItem(let id):                  return "/v1/items/\(id)"
        case .updateItem(let id, _):            return "/v1/items/\(id)"
        case .deleteItem(let id):               return "/v1/items/\(id)"
        case .claimItem(let id):                return "/v1/items/\(id)/claim"
        case .unclaimItem(let id):              return "/v1/items/\(id)/claim"
        case .uploadImage:                      return "/v1/upload/image"
        case .batchDeleteItems:                 return "/v1/items/batch-delete"
        case .ownerUnclaimWeb(let id):          return "/v1/items/\(id)/web-claim"

        // Categories
        case .listCategories:                   return "/v1/categories"

        // Users (Refactor 2: /users/me -> /me/profile)
        case .getProfile:                       return "/v1/me/profile"
        case .updateProfile:                    return "/v1/me/profile"
        case .deleteAccount:                    return "/v1/me/profile"
        case .exportData:                       return "/v1/me/export"
        case .requestEmailChange:               return "/v1/me/email"

        // Push Tokens
        case .registerToken:                    return "/v1/push-tokens"
        case .unregisterToken(let token):       return "/v1/push-tokens/\(token)"

        // Share Links
        case .createShareLink:                  return "/v1/share-links"
        case .listShareLinks:                   return "/v1/share-links"
        case .deleteShareLink(let id):          return "/v1/share-links/\(id)"
        case .updateShareLink(let id, _):       return "/v1/share-links/\(id)"

        // Shared
        case .getSharedView(let token):                 return "/v1/shared/\(token)"
        case .claimViaShare(let token, let itemId):     return "/v1/shared/\(token)/items/\(itemId)/claim"
        case .unclaimViaShare(let token, let itemId):   return "/v1/shared/\(token)/items/\(itemId)/claim"

        // Circles
        case .listCircles:                              return "/v1/circles"
        case .createCircle:                             return "/v1/circles"
        case .getCircle(let id):                        return "/v1/circles/\(id)"
        case .updateCircle(let id, _):                  return "/v1/circles/\(id)"
        case .deleteCircle(let id):                     return "/v1/circles/\(id)"
        case .createDirectCircle(let userId):           return "/v1/circles/direct/\(userId)"
        case .addMemberToCircle(let circleId, _):       return "/v1/circles/\(circleId)/members"
        case .removeMember(let circleId, let userId):   return "/v1/circles/\(circleId)/members/\(userId)"
        case .shareItemToCircle(let circleId, _):       return "/v1/circles/\(circleId)/items"
        case .batchShareItems(let circleId, _):        return "/v1/circles/\(circleId)/items/batch"
        case .listCircleItems(let circleId):            return "/v1/circles/\(circleId)/items"
        case .getCircleItem(let circleId, let itemId):  return "/v1/circles/\(circleId)/items/\(itemId)"
        case .unshareItem(let circleId, let itemId):    return "/v1/circles/\(circleId)/items/\(itemId)"
        case .getShareRule(let circleId):                return "/v1/circles/\(circleId)/share-rule"
        case .setShareRule(let circleId, _):              return "/v1/circles/\(circleId)/share-rule"
        case .getCircleFeed(let circleId, _, _):        return "/v1/circles/\(circleId)/feed"
        case .transferCircleOwnership(let circleId, _): return "/v1/circles/\(circleId)/transfer"
        case .listReservations:                         return "/v1/circles/my-reservations"
        case .listMyShareRules:                         return "/v1/circles/my-share-rules"
        case .listNotifications:                        return "/v1/me/notifications"
        case .markNotificationRead(let id):             return "/v1/me/notifications/\(id)/read"
        case .markAllNotificationsRead:                 return "/v1/me/notifications/read"
        case .deleteNotification(let id):               return "/v1/me/notifications/\(id)"
        case .unreadNotificationCount:                  return "/v1/me/notifications/unread-count"
        case .createCircleInvite(let circleId, _):      return "/v1/circles/\(circleId)/invite"
        case .listCircleInvites(let circleId):          return "/v1/circles/\(circleId)/invites"
        case .revokeCircleInvite(let cid, let iid):     return "/v1/circles/\(cid)/invites/\(iid)"
        case .joinCircleViaInvite(let token):           return "/v1/circles/join/\(token)"

        // Friends
        case .searchUsers:                              return "/v1/users/search"
        case .sendFriendRequest:                        return "/v1/me/friend-requests"
        case .listPendingFriendRequests:                return "/v1/me/friend-requests"
        case .listSentFriendRequests:                   return "/v1/me/friend-requests/sent"
        case .acceptFriendRequest(let id):              return "/v1/me/friend-requests/\(id)/accept"
        case .declineFriendRequest(let id):             return "/v1/me/friend-requests/\(id)"
        case .cancelFriendRequest(let id):              return "/v1/me/friend-requests/\(id)/cancel"
        case .listFriends:                              return "/v1/me/friends"
        case .removeFriend(let userId):                 return "/v1/me/friends/\(userId)"

        // Community Wishes
        case .listCommunityWishes:                      return "/v1/community/wishes"
        case .createCommunityWish:                      return "/v1/community/wishes"
        case .listMyCommunityWishes:                    return "/v1/community/wishes/mine"
        case .listMyCommunityOffers:                   return "/v1/community/wishes/my-offers"
        case .listRecentFulfilled:                     return "/v1/community/wishes/recent-fulfilled"
        case .getCommunityWish(let id):                 return "/v1/community/wishes/\(id)"
        case .updateCommunityWish(let id, _):           return "/v1/community/wishes/\(id)"
        case .closeCommunityWish(let id):               return "/v1/community/wishes/\(id)/close"
        case .deleteCommunityWish(let id):              return "/v1/community/wishes/\(id)"
        case .reopenCommunityWish(let id):              return "/v1/community/wishes/\(id)/reopen"
        case .offerCommunityWish(let id):               return "/v1/community/wishes/\(id)/offer"
        case .withdrawOfferCommunityWish(let id):       return "/v1/community/wishes/\(id)/offer"
        case .rejectOfferCommunityWish(let id):         return "/v1/community/wishes/\(id)/reject"
        case .confirmCommunityWish(let id):             return "/v1/community/wishes/\(id)/confirm"
        case .reportCommunityWish(let id, _):           return "/v1/community/wishes/\(id)/report"
        case .blockCommunityWish(let id):              return "/v1/community/wishes/\(id)/block"
        case .unblockCommunityWish(let id):            return "/v1/community/wishes/\(id)/block"

        // Wish Messages
        case .listWishMessages(let wishId, _):          return "/v1/community/wishes/\(wishId)/messages"
        case .sendWishMessage(let wishId, _):           return "/v1/community/wishes/\(wishId)/messages"
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
        case .verifyResetCode:  return .POST
        case .resetPassword:    return .POST
        case .googleAuth:       return .POST
        case .appleAuth:        return .POST
        case .verifyEmail:      return .POST
        case .resendVerification: return .POST

        // Items
        case .listItems:        return .GET
        case .createItem:       return .POST
        case .getItem:          return .GET
        case .updateItem:       return .PATCH
        case .deleteItem:       return .DELETE
        case .claimItem:        return .POST
        case .unclaimItem:      return .DELETE
        case .uploadImage:      return .POST
        case .batchDeleteItems: return .POST
        case .ownerUnclaimWeb:  return .DELETE

        // Categories
        case .listCategories:   return .GET

        // Users
        case .getProfile:       return .GET
        case .updateProfile:    return .PATCH
        case .deleteAccount:    return .DELETE
        case .exportData:       return .GET
        case .requestEmailChange: return .POST

        // Push Tokens
        case .registerToken:    return .POST
        case .unregisterToken:  return .DELETE

        // Share Links
        case .createShareLink:  return .POST
        case .listShareLinks:   return .GET
        case .deleteShareLink:  return .DELETE
        case .updateShareLink:  return .PATCH

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
        case .batchShareItems:      return .POST
        case .listCircleItems:      return .GET
        case .getCircleItem:        return .GET
        case .unshareItem:          return .DELETE
        case .getShareRule:         return .GET
        case .setShareRule:         return .PUT
        case .getCircleFeed:        return .GET
        case .transferCircleOwnership: return .POST
        case .listReservations:     return .GET
        case .listMyShareRules:     return .GET
        case .listNotifications:    return .GET
        case .markNotificationRead: return .POST
        case .markAllNotificationsRead: return .POST
        case .deleteNotification:   return .DELETE
        case .unreadNotificationCount: return .GET
        case .createCircleInvite:   return .POST
        case .listCircleInvites:    return .GET
        case .revokeCircleInvite:   return .DELETE
        case .joinCircleViaInvite:  return .POST

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

        // Community Wishes
        case .listCommunityWishes:              return .GET
        case .createCommunityWish:              return .POST
        case .listMyCommunityWishes:            return .GET
        case .listMyCommunityOffers:           return .GET
        case .listRecentFulfilled:             return .GET
        case .getCommunityWish:                 return .GET
        case .updateCommunityWish:              return .PATCH
        case .closeCommunityWish:               return .POST
        case .deleteCommunityWish:              return .DELETE
        case .reopenCommunityWish:              return .POST
        case .offerCommunityWish:               return .POST
        case .withdrawOfferCommunityWish:       return .DELETE
        case .rejectOfferCommunityWish:         return .POST
        case .confirmCommunityWish:             return .POST
        case .reportCommunityWish:              return .POST
        case .blockCommunityWish:               return .POST
        case .unblockCommunityWish:             return .DELETE

        // Wish Messages
        case .listWishMessages:                 return .GET
        case .sendWishMessage:                  return .POST
        }
    }

    /// Whether this endpoint requires an `Authorization: Bearer` header.
    var requiresAuth: Bool {
        switch self {
        case .register, .login, .refresh, .forgotPassword, .verifyResetCode, .resetPassword,
             .googleAuth, .appleAuth:
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
        case .listNotifications(let page, let limit):
            return [
                URLQueryItem(name: "page", value: String(page)),
                URLQueryItem(name: "limit", value: String(limit)),
            ]
        case .listCommunityWishes(let query):
            var items: [URLQueryItem] = []
            if let category = query.category {
                items.append(.init(name: "category", value: category))
            }
            items.append(.init(name: "page", value: String(query.page)))
            items.append(.init(name: "limit", value: String(query.limit)))
            return items.isEmpty ? nil : items
        case .listWishMessages(_, let query):
            return [
                URLQueryItem(name: "page", value: String(query.page)),
                URLQueryItem(name: "limit", value: String(query.limit)),
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
        case .verifyResetCode(let body):    return body
        case .resetPassword(let body):      return body
        case .googleAuth(let body):         return body
        case .appleAuth(let body):          return body
        case .verifyEmail(let body):        return body
        case .createItem(let body):         return body
        case .updateItem(_, let body):      return body
        case .updateProfile(let body):      return body
        case .requestEmailChange(let body): return body
        case .registerToken(let body):      return body
        case .createShareLink(let body):    return body
        case .updateShareLink(_, let body): return body
        case .createCircle(let body):       return body
        case .updateCircle(_, let body):    return body
        case .addMemberToCircle(_, let body): return body
        case .shareItemToCircle(_, let body): return body
        case .batchShareItems(_, let body): return body
        case .setShareRule(_, let body): return body
        case .transferCircleOwnership(_, let body): return body
        case .createCircleInvite(_, let body):      return body
        case .sendFriendRequest(let body):          return body
        case .createCommunityWish(let body):        return body
        case .updateCommunityWish(_, let body):     return body
        case .reportCommunityWish(_, let body):     return body
        case .sendWishMessage(_, let body):         return body
        case .batchDeleteItems(let body):          return body
        default:                                    return nil
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
