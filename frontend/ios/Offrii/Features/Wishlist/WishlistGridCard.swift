import SwiftUI

// MARK: - WishlistGridCard

struct WishlistGridCard: View {
    let item: Item
    let category: CategoryResponse?
    var isPurchasedTab: Bool = false
    var onTap: (() -> Void)?

    private var style: CategoryStyle {
        CategoryStyle(icon: category?.icon)
    }

    private var priceText: String? {
        guard let price = item.estimatedPrice else { return nil }
        let formatter = NumberFormatter()
        formatter.numberStyle = .currency
        formatter.currencyCode = "EUR"
        return formatter.string(from: price as NSDecimalNumber)
    }

    var body: some View {
        Button {
            OffriiHaptics.tap()
            onTap?()
        } label: {
            VStack(alignment: .leading, spacing: 0) {
                // Image zone
                ZStack {
                    imageZone

                    // Claimed: full overlay like web
                    if item.isClaimed {
                        Color.black.opacity(0.35)
                        Text(NSLocalizedString("wishlist.reserved", comment: ""))
                            .font(.system(size: 13, weight: .bold))
                            .tracking(2)
                            .textCase(.uppercase)
                            .foregroundColor(.white)
                    }

                    // Other badges (top-right)
                    VStack(alignment: .trailing, spacing: 4) {
                        otherBadges
                    }
                    .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topTrailing)
                    .padding(OffriiTheme.spacingSM)

                    // Shared circles avatar stack (bottom-left)
                    if !item.sharedCircles.isEmpty {
                        let maxVisible = item.sharedCircles.count <= 4 ? item.sharedCircles.count : 3
                        let overflow = item.sharedCircles.count - maxVisible
                        HStack(spacing: -6) {
                            ForEach(item.sharedCircles.prefix(maxVisible)) { circle in
                                Text(circle.initial)
                                    .font(.system(size: 8, weight: .bold))
                                    .foregroundColor(.white)
                                    .frame(width: 20, height: 20)
                                    .background(circle.isDirect == true ? OffriiTheme.textSecondary : OffriiTheme.primary)
                                    .clipShape(Circle())
                                    .overlay(
                                        Circle().strokeBorder(.white, lineWidth: 1.5)
                                    )
                            }
                            if overflow > 0 {
                                Text("+\(overflow)")
                                    .font(.system(size: 7, weight: .bold))
                                    .foregroundColor(.white)
                                    .frame(width: 20, height: 20)
                                    .background(OffriiTheme.textMuted)
                                    .clipShape(Circle())
                                    .overlay(
                                        Circle().strokeBorder(.white, lineWidth: 1.5)
                                    )
                            }
                        }
                        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .bottomLeading)
                        .padding(OffriiTheme.spacingSM)
                    }
                }

                // Text zone
                HStack(alignment: .top, spacing: 4) {
                    // Priority dots
                    priorityDots

                    VStack(alignment: .leading, spacing: 2) {
                        Text(item.name)
                            .font(.system(size: 14, weight: .semibold))
                            .foregroundColor(OffriiTheme.text)
                            .lineLimit(2)
                            .multilineTextAlignment(.leading)

                        Text(priceText ?? category?.name ?? " ")
                            .font(.system(size: 12, weight: .regular))
                            .foregroundColor(OffriiTheme.textMuted)
                            .lineLimit(1)
                    }
                }
                .padding(.horizontal, OffriiTheme.spacingSM)
                .padding(.vertical, OffriiTheme.spacingSM)
                .frame(height: 56, alignment: .top)
            }
            .background(OffriiTheme.card)
            .cornerRadius(OffriiTheme.cornerRadiusLG)
            .shadow(color: OffriiTheme.cardShadowColor, radius: 6, x: 0, y: 2)
            .opacity(isPurchasedTab ? 0.7 : 1.0)
        }
        .buttonStyle(.plain)
    }

    // MARK: - Image Zone

    @ViewBuilder
    private var imageZone: some View {
        if let url = item.displayImageUrl {
            AsyncImage(url: url) { phase in
                switch phase {
                case .success(let image):
                    image
                        .resizable()
                        .aspectRatio(contentMode: .fill)
                        .frame(minWidth: 0, maxWidth: .infinity)
                        .frame(height: 130)
                        .clipped()
                case .failure:
                    placeholderView
                default:
                    placeholderView
                        .shimmer()
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
            colors: style.gradient,
            startPoint: .topLeading,
            endPoint: .bottomTrailing
        )
        .frame(height: 130)
        .overlay(
            Image(systemName: style.sfSymbol)
                .font(.system(size: 32, weight: .light))
                .foregroundColor(.white.opacity(0.7))
        )
    }

    // MARK: - Priority Dots

    @ViewBuilder
    private var priorityDots: some View {
        if item.priority >= 2 {
            HStack(spacing: 2) {
                ForEach(0..<item.priority, id: \.self) { _ in
                    Circle()
                        .fill(item.priority == 3 ? OffriiTheme.danger : OffriiTheme.accent)
                        .frame(width: 5, height: 5)
                }
            }
            .padding(.top, 4)
        }
    }

    // MARK: - Badge Overlay (non-claimed badges)

    @ViewBuilder
    private var otherBadges: some View {
            // Purchased checkmark
            if isPurchasedTab {
                glassBadge {
                    Image(systemName: "checkmark")
                        .font(.system(size: 10, weight: .bold))
                }
            }

            // Private
            if item.isPrivate {
                glassBadge {
                    Image(systemName: "lock.fill")
                        .font(.system(size: 9, weight: .semibold))
                    Text(NSLocalizedString("wishlist.private", comment: ""))
                        .font(.system(size: 9, weight: .semibold))
                }
            }

            // (Shared circles now shown as avatar stack at bottom-left of image)

            // Has links
            if let links = item.links, !links.isEmpty {
                glassBadge {
                    Image(systemName: "link")
                        .font(.system(size: 9, weight: .semibold))
                }
            }
    }

    /// Unified glass morphism badge style
    private func glassBadge<Content: View>(@ViewBuilder content: () -> Content) -> some View {
        HStack(spacing: 3) {
            content()
        }
        .foregroundColor(.white)
        .font(.system(size: 9, weight: .semibold))
        .padding(.horizontal, 7)
        .padding(.vertical, 4)
        .background(
            RoundedRectangle(cornerRadius: 6)
                .fill(.black.opacity(0.35))
                .background(
                    RoundedRectangle(cornerRadius: 6)
                        .fill(.ultraThinMaterial)
                )
        )
        .clipShape(RoundedRectangle(cornerRadius: 6))
        .overlay(
            RoundedRectangle(cornerRadius: 6)
                .strokeBorder(.white.opacity(0.2), lineWidth: 0.5)
        )
        .shadow(color: .black.opacity(0.15), radius: 2, x: 0, y: 1)
    }
}
