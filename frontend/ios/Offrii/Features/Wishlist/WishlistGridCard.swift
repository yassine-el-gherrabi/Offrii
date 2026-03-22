import NukeUI
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

                    // Badges
                    badgeOverlay

                    // Shared circles avatar stack (bottom-left)
                    if !item.sharedCircles.isEmpty {
                        let maxVisible = item.sharedCircles.count <= 4 ? item.sharedCircles.count : 3
                        let overflow = item.sharedCircles.count - maxVisible
                        HStack(spacing: -6) {
                            ForEach(item.sharedCircles.prefix(maxVisible)) { circle in
                                if let url = circle.imageURL {
                                    LazyImage(url: url) { state in
                                        if let image = state.image {
                                            image
                                                .resizable()
                                                .aspectRatio(contentMode: .fill)
                                                .frame(width: 20, height: 20)
                                                .clipShape(Circle())
                                                .overlay(Circle().strokeBorder(.white, lineWidth: 1.5))
                                        } else {
                                            circleInitial(circle)
                                        }
                                    }
                                } else {
                                    circleInitial(circle)
                                }
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
            LazyImage(url: url) { state in
                if let image = state.image {
                    image
                        .resizable()
                        .aspectRatio(contentMode: .fill)
                        .frame(minWidth: 0, maxWidth: .infinity)
                        .frame(height: 130)
                        .clipped()
                } else if state.error != nil {
                    placeholderView
                } else {
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

    private func circleInitial(_ circle: SharedCircleInfo) -> some View {
        Text(circle.initial)
            .font(.system(size: 8, weight: .bold))
            .foregroundColor(.white)
            .frame(width: 20, height: 20)
            .background(OffriiTheme.primary)
            .clipShape(Circle())
            .overlay(Circle().strokeBorder(.white, lineWidth: 1.5))
    }

    // MARK: - Badge Overlay

    private var badgeOverlay: some View {
        ZStack {
            // Top-left: Private lock (icon only)
            if item.isPrivate {
                glassBadge {
                    Image(systemName: "lock.fill")
                        .font(.system(size: 10, weight: .semibold))
                }
                .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
                .padding(OffriiTheme.spacingSM)
            }

            // Top-right: Priority flames (2-3 only, 1 = default = no badge)
            if item.priority >= 2 {
                glassBadge {
                    HStack(spacing: -1) {
                        ForEach(0..<item.priority, id: \.self) { _ in
                            Image(systemName: "flame.fill")
                                .font(.system(size: 10))
                        }
                    }
                }
                .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topTrailing)
                .padding(OffriiTheme.spacingSM)
            }

            // Bottom-right: Link indicator (small, discreet)
            if let links = item.links, !links.isEmpty {
                glassBadge {
                    Image(systemName: "link")
                        .font(.system(size: 9, weight: .semibold))
                }
                .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .bottomTrailing)
                .padding(OffriiTheme.spacingSM)
            }
        }
    }

    /// Badge style: white background + corail icons
    private func glassBadge<Content: View>(@ViewBuilder content: () -> Content) -> some View {
        HStack(spacing: 3) {
            content()
        }
        .foregroundColor(OffriiTheme.primary)
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
