import Foundation

// MARK: - Wish Category

enum WishCategory: String, Codable, CaseIterable, Identifiable {
    case education
    case clothing
    case health
    case religion
    case home
    case children
    case other

    var id: String { rawValue }

    var label: String {
        switch self {
        case .education: return NSLocalizedString("entraide.category.education", comment: "")
        case .clothing:  return NSLocalizedString("entraide.category.clothing", comment: "")
        case .health:    return NSLocalizedString("entraide.category.health", comment: "")
        case .religion:  return NSLocalizedString("entraide.category.religion", comment: "")
        case .home:      return NSLocalizedString("entraide.category.home", comment: "")
        case .children:  return NSLocalizedString("entraide.category.children", comment: "")
        case .other:     return NSLocalizedString("entraide.category.other", comment: "")
        }
    }

    var emoji: String {
        switch self {
        case .education: return "📚"
        case .clothing:  return "🧥"
        case .health:    return "🏥"
        case .religion:  return "🕌"
        case .home:      return "🏠"
        case .children:  return "👶"
        case .other:     return "📦"
        }
    }
}

// MARK: - Wish Status

enum WishStatus: String, Codable {
    case pending
    case flagged
    case rejected
    case open
    case matched
    case fulfilled
    case closed
    case review
}

// MARK: - Report Reason

enum WishReportReason: String, Codable, CaseIterable, Identifiable {
    case inappropriate
    case spam
    case scam
    case other

    var id: String { rawValue }

    var label: String {
        switch self {
        case .inappropriate: return NSLocalizedString("entraide.report.inappropriate", comment: "")
        case .spam:          return NSLocalizedString("entraide.report.spam", comment: "")
        case .scam:          return NSLocalizedString("entraide.report.scam", comment: "")
        case .other:         return NSLocalizedString("entraide.report.other", comment: "")
        }
    }
}

// MARK: - Wish Response (public feed item)

struct CommunityWish: Codable, Identifiable, Equatable {
    let id: UUID
    let displayName: String?
    let title: String
    let description: String?
    let category: WishCategory
    let status: WishStatus
    let isMine: Bool
    let isMatchedByMe: Bool
    let imageUrl: String?
    let links: [String]?
    let createdAt: Date

    enum CodingKeys: String, CodingKey {
        case id, title, description, category, status, links
        case displayName = "display_name"
        case isMine = "is_mine"
        case isMatchedByMe = "is_matched_by_me"
        case imageUrl = "image_url"
        case createdAt = "created_at"
    }
}

// MARK: - Wish Detail Response

struct WishDetail: Codable, Identifiable, Equatable {
    let id: UUID
    let displayName: String?
    let title: String
    let description: String?
    let category: WishCategory
    let status: WishStatus
    let isMine: Bool
    let isMatchedByMe: Bool
    let matchedWithDisplayName: String?
    let imageUrl: String?
    let links: [String]?
    let matchedAt: Date?
    let fulfilledAt: Date?
    let createdAt: Date

    enum CodingKeys: String, CodingKey {
        case id, title, description, category, status, links
        case displayName = "display_name"
        case isMine = "is_mine"
        case isMatchedByMe = "is_matched_by_me"
        case matchedWithDisplayName = "matched_with_display_name"
        case imageUrl = "image_url"
        case matchedAt = "matched_at"
        case fulfilledAt = "fulfilled_at"
        case createdAt = "created_at"
    }
}

// MARK: - My Wish Response (owner view)

struct MyWish: Codable, Identifiable, Equatable {
    let id: UUID
    let title: String
    let description: String?
    let category: WishCategory
    let status: WishStatus
    let isAnonymous: Bool
    let matchedWithDisplayName: String?
    let reportCount: Int
    let reopenCount: Int
    let moderationNote: String?
    let imageUrl: String?
    let links: [String]?
    let createdAt: Date
    let matchedAt: Date?
    let fulfilledAt: Date?
    let closedAt: Date?

    enum CodingKeys: String, CodingKey {
        case id, title, description, category, status, links
        case isAnonymous = "is_anonymous"
        case matchedWithDisplayName = "matched_with_display_name"
        case reportCount = "report_count"
        case reopenCount = "reopen_count"
        case moderationNote = "moderation_note"
        case imageUrl = "image_url"
        case createdAt = "created_at"
        case matchedAt = "matched_at"
        case fulfilledAt = "fulfilled_at"
        case closedAt = "closed_at"
    }
}

// MARK: - Wish Message

struct WishMessage: Codable, Identifiable, Equatable {
    let id: UUID
    let senderDisplayName: String
    let isMine: Bool
    let body: String
    let createdAt: Date

    enum CodingKeys: String, CodingKey {
        case id, body
        case senderDisplayName = "sender_display_name"
        case isMine = "is_mine"
        case createdAt = "created_at"
    }
}

// MARK: - Pagination Meta

struct PaginationMeta: Codable, Equatable {
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

// MARK: - Paginated Response (generic)

struct PaginatedResponse<T: Codable>: Codable where T: Equatable {
    let data: [T]
    let pagination: PaginationMeta
}
