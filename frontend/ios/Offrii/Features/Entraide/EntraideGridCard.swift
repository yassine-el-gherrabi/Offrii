import SwiftUI

// MARK: - EntraideGridCard

struct EntraideGridCard: View {
    let wish: CommunityWish
    var onTap: (() -> Void)?

    private var imageURL: URL? {
        guard let urlStr = wish.imageUrl else { return nil }
        return URL(string: urlStr)
    }

    private var statusVariant: StatusDotVariant {
        switch wish.status {
        case .open:      return .open
        case .matched:   return .matched
        case .fulfilled: return .fulfilled
        case .closed:    return .closed
        default:         return .closed
        }
    }

    private var subtitleText: String {
        let category = wish.category.chipLabel
        let time = RelativeDateTimeFormatter().localizedString(for: wish.createdAt, relativeTo: Date())
        return "\(category) · \(time)"
    }

    var body: some View {
        OffriiGridCard(
            imageURL: imageURL,
            placeholderIcon: categoryIcon(wish.category),
            placeholderGradient: [wish.category.backgroundColor, wish.category.textColor.opacity(0.3)],
            title: wish.title,
            subtitle: subtitleText,
            badges: [.status(statusVariant)],
            onTap: onTap
        )
    }

    private func categoryIcon(_ cat: WishCategory) -> String {
        switch cat {
        case .education: return "book.fill"
        case .clothing:  return "tshirt.fill"
        case .health:    return "heart.fill"
        case .religion:  return "hands.sparkles.fill"
        case .home:      return "house.fill"
        case .children:  return "figure.and.child.holdinghands"
        case .other:     return "gift.fill"
        }
    }
}
