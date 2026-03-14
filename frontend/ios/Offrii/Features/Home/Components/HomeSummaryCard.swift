import SwiftUI

// MARK: - HomeSummaryCard

struct HomeSummaryCard: View {
    let stats: HomeViewModel.Stats

    var body: some View {
        HStack(spacing: OffriiTheme.spacingLG) {
            statItem(icon: "heart.fill", value: "\(stats.totalItems)", label: NSLocalizedString("tab.envies", comment: ""))
            statItem(icon: "checkmark.circle.fill", value: "\(stats.claimedItems)", label: NSLocalizedString("wishlist.reserved", comment: ""))
            statItem(icon: "person.2.fill", value: "\(stats.circleCount)", label: NSLocalizedString("tab.cercles", comment: ""))
            statItem(icon: "person.fill", value: "\(stats.friendCount)", label: NSLocalizedString("profile.friends", comment: ""))
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

    private func statItem(icon: String, value: String, label: String) -> some View {
        VStack(spacing: OffriiTheme.spacingXS) {
            Image(systemName: icon)
                .font(.system(size: 16))
                .foregroundColor(OffriiTheme.primary)

            Text(value)
                .font(OffriiTypography.headline)
                .foregroundColor(OffriiTheme.text)

            Text(label)
                .font(OffriiTypography.caption2)
                .foregroundColor(OffriiTheme.textMuted)
                .lineLimit(1)
        }
        .frame(maxWidth: .infinity)
    }
}
