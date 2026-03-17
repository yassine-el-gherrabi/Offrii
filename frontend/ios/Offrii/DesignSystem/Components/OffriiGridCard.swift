import NukeUI
import SwiftUI

// MARK: - Grid Card Badge

enum GridCardBadge {
    case priority(Int)
    case reserved
    case status(StatusDotVariant)
}

// MARK: - OffriiGridCard

struct OffriiGridCard: View {
    let imageURL: URL?
    var placeholderIcon: String = "gift.fill"
    var placeholderGradient: [Color] = [OffriiTheme.primary.opacity(0.3), OffriiTheme.accent.opacity(0.2)]
    let title: String
    var subtitle: String?
    var badges: [GridCardBadge] = []
    var onTap: (() -> Void)?

    var body: some View {
        Button {
            OffriiHaptics.tap()
            onTap?()
        } label: {
            VStack(alignment: .leading, spacing: 0) {
                // Image zone
                ZStack(alignment: .topTrailing) {
                    imageZone
                    badgeOverlay
                }

                // Text zone
                VStack(alignment: .leading, spacing: OffriiTheme.spacingXXS) {
                    Text(title)
                        .font(.system(size: 15, weight: .semibold))
                        .foregroundColor(OffriiTheme.text)
                        .lineLimit(2)
                        .multilineTextAlignment(.leading)

                    if let subtitle {
                        Text(subtitle)
                            .font(.system(size: 13, weight: .regular))
                            .foregroundColor(OffriiTheme.textMuted)
                            .lineLimit(1)
                    }
                }
                .padding(.horizontal, OffriiTheme.spacingSM)
                .padding(.vertical, OffriiTheme.spacingSM)
            }
            .background(OffriiTheme.card)
            .cornerRadius(OffriiTheme.cornerRadiusLG)
            .shadow(
                color: OffriiTheme.cardShadowColor,
                radius: 6,
                x: 0,
                y: 2
            )
        }
        .buttonStyle(.plain)
    }

    // MARK: - Image Zone

    @ViewBuilder
    private var imageZone: some View {
        if let url = imageURL {
            LazyImage(url: url) { state in
                if let image = state.image {
                    image
                        .resizable()
                        .aspectRatio(contentMode: .fill)
                } else {
                    placeholderView
                }
            }
            .frame(height: 120)
            .clipped()
        } else {
            placeholderView
        }
    }

    private var placeholderView: some View {
        LinearGradient(
            colors: placeholderGradient,
            startPoint: .topLeading,
            endPoint: .bottomTrailing
        )
        .frame(height: 120)
        .overlay(
            Image(systemName: placeholderIcon)
                .font(.system(size: 32, weight: .light))
                .foregroundColor(.white.opacity(0.7))
        )
    }

    // MARK: - Badge Overlay

    @ViewBuilder
    private var badgeOverlay: some View {
        VStack(alignment: .trailing, spacing: 4) {
            ForEach(Array(badges.enumerated()), id: \.offset) { _, badge in
                switch badge {
                case .priority(let level):
                    if level >= 2 {
                        StatusDot(variant: .priority(level), size: 10)
                            .padding(6)
                            .background(.ultraThinMaterial)
                            .cornerRadius(OffriiTheme.cornerRadiusXS)
                    }
                case .reserved:
                    HStack(spacing: 3) {
                        Image(systemName: "lock.fill")
                            .font(.system(size: 8, weight: .bold))
                        Text(NSLocalizedString("wishlist.reserved", comment: ""))
                            .font(.system(size: 9, weight: .semibold))
                    }
                    .foregroundColor(OffriiTheme.accent)
                    .padding(.horizontal, 6)
                    .padding(.vertical, 3)
                    .background(.ultraThinMaterial)
                    .cornerRadius(OffriiTheme.cornerRadiusXS)
                case .status(let variant):
                    StatusDot(variant: variant, showLabel: true, size: 8)
                        .padding(.horizontal, 6)
                        .padding(.vertical, 3)
                        .background(.ultraThinMaterial)
                        .cornerRadius(OffriiTheme.cornerRadiusXS)
                }
            }
        }
        .padding(OffriiTheme.spacingSM)
    }
}

// MARK: - Skeleton Grid Card

struct SkeletonGridCard: View {
    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            RoundedRectangle(cornerRadius: 0)
                .fill(OffriiTheme.border.opacity(0.2))
                .frame(height: 120)

            VStack(alignment: .leading, spacing: OffriiTheme.spacingXXS) {
                RoundedRectangle(cornerRadius: OffriiTheme.cornerRadiusXS)
                    .fill(OffriiTheme.border.opacity(0.3))
                    .frame(height: 14)

                RoundedRectangle(cornerRadius: OffriiTheme.cornerRadiusXS)
                    .fill(OffriiTheme.border.opacity(0.2))
                    .frame(width: 60, height: 12)
            }
            .padding(.horizontal, OffriiTheme.spacingSM)
            .padding(.vertical, OffriiTheme.spacingSM)
        }
        .background(OffriiTheme.card)
        .cornerRadius(OffriiTheme.cornerRadiusLG)
        .shimmer()
    }
}
