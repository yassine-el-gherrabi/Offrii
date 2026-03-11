import SwiftUI

// MARK: - WishCard

struct WishCard: View {
    let wish: CommunityWish
    var onOffer: (() -> Void)?

    var body: some View {
        VStack(alignment: .leading, spacing: OffriiTheme.spacingSM) {
            // Top row: category chip + status badge
            HStack {
                WishCategoryChip(category: wish.category)
                Spacer()
                WishStatusBadge(status: wish.status)
            }

            // Title
            Text(wish.title)
                .font(OffriiTypography.headline)
                .foregroundColor(OffriiTheme.text)
                .lineLimit(2)

            // Description (truncated)
            if let description = wish.description, !description.isEmpty {
                Text(description)
                    .font(OffriiTypography.subheadline)
                    .foregroundColor(OffriiTheme.textSecondary)
                    .lineLimit(2)
            }

            // Bottom row: author + date
            HStack {
                // Author
                HStack(spacing: OffriiTheme.spacingXS) {
                    Image(systemName: "person.fill")
                        .font(.system(size: 10))
                        .foregroundColor(OffriiTheme.textMuted)
                    Text(wish.displayName ?? NSLocalizedString("entraide.anonymous", comment: ""))
                        .font(OffriiTypography.caption)
                        .foregroundColor(OffriiTheme.textMuted)
                }

                Spacer()

                // Relative date
                Text(wish.createdAt, style: .relative)
                    .font(OffriiTypography.caption)
                    .foregroundColor(OffriiTheme.textMuted)
            }

            // CTA button for open wishes that aren't mine
            if wish.status == .open && !wish.isMine {
                Button {
                    onOffer?()
                } label: {
                    Text("entraide.offer")
                        .font(OffriiTypography.footnote)
                        .fontWeight(.semibold)
                        .foregroundColor(.white)
                        .frame(maxWidth: .infinity)
                        .padding(.vertical, OffriiTheme.spacingSM)
                        .background(OffriiTheme.primary)
                        .cornerRadius(OffriiTheme.cornerRadiusSM)
                }
                .buttonStyle(.plain)
            }
        }
        .padding(OffriiTheme.spacingBase)
        .background(OffriiTheme.card)
        .cornerRadius(OffriiTheme.cornerRadiusLG)
        .shadow(
            color: OffriiTheme.cardShadowColor,
            radius: OffriiTheme.cardShadowRadius,
            x: 0,
            y: OffriiTheme.cardShadowY
        )
    }
}
