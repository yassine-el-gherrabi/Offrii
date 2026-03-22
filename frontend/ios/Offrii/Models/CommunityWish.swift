import Foundation
import SwiftUI

// MARK: - Wish Category

enum WishCategory: String, Codable, CaseIterable, Identifiable, CategoryChipItem {
    case education
    case clothing
    case health
    case religion
    case home
    case children
    case other

    var id: String { rawValue }

    var chipLabel: String { label }
    var chipIcon: String { icon }
    var chipColor: Color { color }

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

    var icon: String {
        switch self {
        case .education: return "book.fill"
        case .clothing:  return "tshirt.fill"
        case .health:    return "heart.fill"
        case .religion:  return "hands.sparkles.fill"
        case .home:      return "house.fill"
        case .children:  return "figure.and.child.holdinghands"
        case .other:     return "tag.fill"
        }
    }

    var color: Color {
        switch self {
        case .education: return Color(red: 0.2, green: 0.4, blue: 0.85)
        case .clothing:  return Color(red: 0.7, green: 0.3, blue: 0.6)
        case .health:    return Color(red: 0.85, green: 0.3, blue: 0.35)
        case .religion:  return Color(red: 0.55, green: 0.4, blue: 0.75)
        case .home:      return Color(red: 0.9, green: 0.5, blue: 0.2)
        case .children:  return Color(red: 0.3, green: 0.7, blue: 0.6)
        case .other:     return Color(red: 0.5, green: 0.5, blue: 0.6)
        }
    }

    var gradient: [Color] {
        switch self {
        case .education: return [color, Color(red: 0.4, green: 0.6, blue: 1.0)]
        case .clothing:  return [color, Color(red: 0.9, green: 0.5, blue: 0.8)]
        case .health:    return [color, Color(red: 1.0, green: 0.5, blue: 0.55)]
        case .religion:  return [color, Color(red: 0.75, green: 0.6, blue: 0.95)]
        case .home:      return [color, Color(red: 1.0, green: 0.7, blue: 0.4)]
        case .children:  return [color, Color(red: 0.5, green: 0.9, blue: 0.8)]
        case .other:     return [color, Color(red: 0.7, green: 0.7, blue: 0.8)]
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

struct CommunityWish: Codable, Identifiable, Equatable, Sendable {
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
    let fulfilledAt: Date?
    let createdAt: Date
    let hasReported: Bool?
    let ogImageUrl: String?
    let ogTitle: String?
    let ogSiteName: String?

    enum CodingKeys: String, CodingKey {
        case id, title, description, category, status, links
        case displayName = "display_name"
        case isMine = "is_mine"
        case isMatchedByMe = "is_matched_by_me"
        case hasReported = "has_reported"
        case imageUrl = "image_url"
        case fulfilledAt = "fulfilled_at"
        case createdAt = "created_at"
        case ogImageUrl = "og_image_url"
        case ogTitle = "og_title"
        case ogSiteName = "og_site_name"
    }

    var displayImageUrl: URL? {
        if let imageUrl, let url = URL(string: imageUrl) { return url }
        if let ogImageUrl, let url = URL(string: ogImageUrl) { return url }
        return nil
    }
}

// MARK: - Wish Detail Response

struct WishDetail: Codable, Identifiable, Equatable, Sendable {
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
    let hasReported: Bool?
    let createdAt: Date
    let ogImageUrl: String?
    let ogTitle: String?
    let ogSiteName: String?

    enum CodingKeys: String, CodingKey {
        case id, title, description, category, status, links
        case displayName = "display_name"
        case isMine = "is_mine"
        case isMatchedByMe = "is_matched_by_me"
        case matchedWithDisplayName = "matched_with_display_name"
        case imageUrl = "image_url"
        case matchedAt = "matched_at"
        case fulfilledAt = "fulfilled_at"
        case hasReported = "has_reported"
        case createdAt = "created_at"
        case ogImageUrl = "og_image_url"
        case ogTitle = "og_title"
        case ogSiteName = "og_site_name"
    }
}

// MARK: - My Wish Response (owner view)

struct MyWish: Codable, Identifiable, Equatable, Sendable {
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
    let ogImageUrl: String?
    let ogTitle: String?
    let ogSiteName: String?

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
        case ogImageUrl = "og_image_url"
        case ogTitle = "og_title"
        case ogSiteName = "og_site_name"
    }
}

// MARK: - Wish Message

struct WishMessage: Codable, Identifiable, Equatable, Sendable {
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

struct PaginationMeta: Codable, Equatable, Sendable {
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

struct PaginatedResponse<T: Codable & Equatable & Sendable>: Codable, Sendable {
    let data: [T]
    let pagination: PaginationMeta
}
