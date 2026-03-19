import NukeUI
import SwiftUI

// MARK: - Entraide Wish Card (full-width, text-first)

struct EntraideWishCard: View {
    let wish: CommunityWish
    var onTap: (() -> Void)?

    var body: some View {
        Button {
            OffriiHaptics.tap()
            onTap?()
        } label: {
            HStack(alignment: .top, spacing: OffriiTheme.spacingMD) {
                // Category icon
                Image(systemName: wish.category.icon)
                    .font(.system(size: 18))
                    .foregroundColor(wish.category.color)
                    .frame(width: 36, height: 36)
                    .background(wish.category.color.opacity(0.12))
                    .clipShape(RoundedRectangle(cornerRadius: 8))

                // Text content
                VStack(alignment: .leading, spacing: 3) {
                    Text(wish.title)
                        .font(.system(size: 15, weight: .semibold))
                        .foregroundColor(OffriiTheme.text)
                        .lineLimit(2)
                        .multilineTextAlignment(.leading)

                    if let desc = wish.description, !desc.isEmpty {
                        Text(desc)
                            .font(.system(size: 13))
                            .foregroundColor(OffriiTheme.textSecondary)
                            .lineLimit(2)
                    }

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

                    if wish.status != .open {
                        statusBadge
                    } else if wish.createdAt < Date().addingTimeInterval(-48 * 3600) {
                        HStack(spacing: 4) {
                            Image(systemName: "clock")
                                .font(.system(size: 10))
                            Text(NSLocalizedString("entraide.aging.waiting", comment: ""))
                                .font(.system(size: 11))
                        }
                        .foregroundColor(OffriiTheme.warning)
                        .padding(.top, 2)
                    }
                }

                Spacer(minLength: 0)

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
