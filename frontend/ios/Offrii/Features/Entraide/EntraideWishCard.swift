import NukeUI
import SwiftUI

// MARK: - Entraide Wish Card (full-width, text-first)

struct EntraideWishCard: View {
    let wish: CommunityWish
    var onTap: (() -> Void)?

    private var categoryIcon: String {
        switch wish.category {
        case .education: return "book.fill"
        case .clothing:  return "tshirt.fill"
        case .health:    return "heart.fill"
        case .religion:  return "hands.sparkles.fill"
        case .home:      return "house.fill"
        case .children:  return "figure.and.child.holdinghands"
        case .other:     return "tag.fill"
        }
    }

    private var categoryColor: Color {
        switch wish.category {
        case .education: return Color(red: 0.2, green: 0.4, blue: 0.85)
        case .clothing:  return Color(red: 0.7, green: 0.3, blue: 0.6)
        case .health:    return Color(red: 0.85, green: 0.3, blue: 0.35)
        case .religion:  return Color(red: 0.55, green: 0.4, blue: 0.75)
        case .home:      return Color(red: 0.9, green: 0.5, blue: 0.2)
        case .children:  return Color(red: 0.3, green: 0.7, blue: 0.6)
        case .other:     return Color(red: 0.5, green: 0.5, blue: 0.6)
        }
    }

    var body: some View {
        Button {
            OffriiHaptics.tap()
            onTap?()
        } label: {
            HStack(alignment: .top, spacing: OffriiTheme.spacingMD) {
                // Category icon
                Image(systemName: categoryIcon)
                    .font(.system(size: 18))
                    .foregroundColor(categoryColor)
                    .frame(width: 36, height: 36)
                    .background(categoryColor.opacity(0.12))
                    .clipShape(RoundedRectangle(cornerRadius: 8))

                // Text content
                VStack(alignment: .leading, spacing: 3) {
                    // Title
                    Text(wish.title)
                        .font(.system(size: 15, weight: .semibold))
                        .foregroundColor(OffriiTheme.text)
                        .lineLimit(2)
                        .multilineTextAlignment(.leading)

                    // Description
                    if let desc = wish.description, !desc.isEmpty {
                        Text(desc)
                            .font(.system(size: 13))
                            .foregroundColor(OffriiTheme.textSecondary)
                            .lineLimit(2)
                    }

                    // Metadata: Author · Category · Time
                    HStack(spacing: 4) {
                        if let name = wish.displayName {
                            Text(name)
                            Text("·")
                        }
                        Text(wish.category.label)
                        Text("·")
                        Text(wish.createdAt, style: .relative)
                    }
                    .font(.system(size: 12))
                    .foregroundColor(OffriiTheme.textMuted)
                    .lineLimit(1)

                    // Status badge (only for non-open)
                    if wish.status != .open {
                        statusBadge
                    }
                }

                Spacer(minLength: 0)

                // Optional photo (trailing, small)
                if let url = wish.displayImageUrl {
                    LazyImage(url: url) { state in
                        if let image = state.image {
                            image
                                .resizable()
                                .aspectRatio(contentMode: .fill)
                                .frame(width: 60, height: 60)
                                .clipShape(RoundedRectangle(cornerRadius: 8))
                        }
                    }
                    .frame(width: 60, height: 60)
                }
            }
            .padding(OffriiTheme.spacingBase)
            .background(OffriiTheme.card)
            .cornerRadius(OffriiTheme.cornerRadiusLG)
            .shadow(color: OffriiTheme.cardShadowColor, radius: 4, x: 0, y: 2)
        }
        .buttonStyle(.plain)
    }

    // MARK: - Status Badge

    private var statusBadge: some View {
        let (color, label) = statusInfo
        return HStack(spacing: 4) {
            Circle().fill(color).frame(width: 6, height: 6)
            Text(label)
                .font(.system(size: 11, weight: .medium))
        }
        .foregroundColor(color)
        .padding(.top, 2)
    }

    private var statusInfo: (Color, String) {
        switch wish.status {
        case .matched:   return (OffriiTheme.warning, NSLocalizedString("entraide.status.matched", comment: ""))
        case .fulfilled: return (OffriiTheme.primary, NSLocalizedString("entraide.status.fulfilled", comment: ""))
        case .closed:    return (OffriiTheme.textMuted, NSLocalizedString("entraide.status.closed", comment: ""))
        default:         return (OffriiTheme.textMuted, "")
        }
    }
}
