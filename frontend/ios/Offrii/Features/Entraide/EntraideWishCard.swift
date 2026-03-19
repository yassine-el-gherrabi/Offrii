import NukeUI
import SwiftUI

// MARK: - Entraide Wish Card (same visual pattern as WishlistGridCard)

struct EntraideWishCard: View {
    let wish: CommunityWish
    var onTap: (() -> Void)?

    // Same saturation level as CategoryStyle.gradient in Envies
    private var categoryGradient: [Color] {
        switch wish.category {
        case .education: return [Color(red: 0.2, green: 0.4, blue: 0.85), Color(red: 0.4, green: 0.6, blue: 1.0)]
        case .clothing:  return [Color(red: 0.7, green: 0.3, blue: 0.6), Color(red: 0.9, green: 0.5, blue: 0.8)]
        case .health:    return [Color(red: 0.85, green: 0.3, blue: 0.35), Color(red: 1.0, green: 0.5, blue: 0.55)]
        case .religion:  return [Color(red: 0.55, green: 0.4, blue: 0.75), Color(red: 0.75, green: 0.6, blue: 0.95)]
        case .home:      return [Color(red: 0.9, green: 0.5, blue: 0.2), Color(red: 1.0, green: 0.7, blue: 0.4)]
        case .children:  return [Color(red: 0.3, green: 0.7, blue: 0.6), Color(red: 0.5, green: 0.9, blue: 0.8)]
        case .other:     return [Color(red: 0.5, green: 0.5, blue: 0.6), Color(red: 0.7, green: 0.7, blue: 0.8)]
        }
    }

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

    private var statusColor: Color {
        switch wish.status {
        case .open:      return OffriiTheme.success
        case .matched:   return OffriiTheme.warning
        case .fulfilled: return OffriiTheme.primary
        default:         return OffriiTheme.textMuted
        }
    }

    private var statusLabel: String {
        switch wish.status {
        case .open:      return NSLocalizedString("entraide.status.open", comment: "")
        case .matched:   return NSLocalizedString("entraide.status.matched", comment: "")
        case .fulfilled: return NSLocalizedString("entraide.status.fulfilled", comment: "")
        case .closed:    return NSLocalizedString("entraide.status.closed", comment: "")
        case .pending:   return NSLocalizedString("entraide.status.pending", comment: "")
        case .review:    return NSLocalizedString("entraide.status.review", comment: "")
        case .flagged:   return NSLocalizedString("entraide.status.flagged", comment: "")
        case .rejected:  return NSLocalizedString("entraide.status.rejected", comment: "")
        }
    }

    var body: some View {
        Button {
            OffriiHaptics.tap()
            onTap?()
        } label: {
            VStack(alignment: .leading, spacing: 0) {
                ZStack {
                    imageZone

                    // Badges top-right (same as WishlistGridCard.otherBadges)
                    VStack(alignment: .trailing, spacing: 4) {
                        // Status badge
                        glassBadge(color: statusColor) {
                            HStack(spacing: 3) {
                                Circle().fill(statusColor).frame(width: 6, height: 6)
                                Text(statusLabel)
                            }
                        }

                        // Links badge
                        if let links = wish.links, !links.isEmpty {
                            glassBadge(color: OffriiTheme.primary) {
                                Image(systemName: "link")
                            }
                        }
                    }
                    .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topTrailing)
                    .padding(OffriiTheme.spacingSM)

                    // Anonymous badge top-left (same position as "Privé" in Envies)
                    if wish.displayName == nil {
                        VStack {
                            glassBadge(color: OffriiTheme.primary) {
                                Image(systemName: "person.fill.questionmark")
                                Text(NSLocalizedString("entraide.anonymous", comment: ""))
                            }
                        }
                        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
                        .padding(OffriiTheme.spacingSM)
                    }
                }

                // Text zone (same as WishlistGridCard)
                VStack(alignment: .leading, spacing: 2) {
                    Text(wish.title)
                        .font(.system(size: 14, weight: .semibold))
                        .foregroundColor(OffriiTheme.text)
                        .lineLimit(2)
                        .multilineTextAlignment(.leading)

                    HStack(spacing: 4) {
                        Text(wish.category.label)
                        Text("·")
                        Text(wish.createdAt, style: .relative)
                    }
                    .font(.system(size: 12))
                    .foregroundColor(OffriiTheme.textMuted)
                    .lineLimit(1)
                }
                .padding(.horizontal, OffriiTheme.spacingSM)
                .padding(.vertical, OffriiTheme.spacingSM)
                .frame(height: 56, alignment: .top)
            }
            .background(OffriiTheme.card)
            .cornerRadius(OffriiTheme.cornerRadiusLG)
            .shadow(color: OffriiTheme.cardShadowColor, radius: 6, x: 0, y: 2)
        }
        .buttonStyle(.plain)
    }

    // MARK: - Image Zone

    @ViewBuilder
    private var imageZone: some View {
        if let url = wish.displayImageUrl {
            LazyImage(url: url) { state in
                if let image = state.image {
                    image
                        .resizable()
                        .aspectRatio(contentMode: .fill)
                        .frame(minWidth: 0, maxWidth: .infinity)
                        .frame(height: 130)
                        .clipped()
                } else {
                    placeholderView
                }
            }
            .frame(height: 130)
            .clipped()
        } else {
            placeholderView
        }
    }

    private var placeholderView: some View {
        LinearGradient(
            colors: categoryGradient,
            startPoint: .topLeading,
            endPoint: .bottomTrailing
        )
        .frame(height: 130)
        .overlay(
            Image(systemName: categoryIcon)
                .font(.system(size: 32, weight: .light))
                .foregroundColor(.white.opacity(0.7))
        )
    }

    // MARK: - Glass Badge (same style as WishlistGridCard.glassBadge)

    private func glassBadge<Content: View>(
        color: Color = OffriiTheme.primary,
        @ViewBuilder content: () -> Content
    ) -> some View {
        HStack(spacing: 3) {
            content()
        }
        .foregroundColor(color)
        .font(.system(size: 9, weight: .semibold))
        .padding(.horizontal, 7)
        .padding(.vertical, 4)
        .background(
            RoundedRectangle(cornerRadius: 6)
                .fill(.white)
        )
        .clipShape(RoundedRectangle(cornerRadius: 6))
        .shadow(color: .black.opacity(0.08), radius: 4, x: 0, y: 1)
    }
}
