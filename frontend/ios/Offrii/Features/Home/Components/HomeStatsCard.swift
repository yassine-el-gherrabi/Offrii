import SwiftUI

// MARK: - HomeStatsCard

struct HomeStatsCard: View {
    let stats: HomeViewModel.Stats

    var body: some View {
        ZStack {
            ScrollView(.horizontal, showsIndicators: false) {
                HStack(spacing: OffriiTheme.spacingSM) {
                    statChip(
                        icon: "heart.fill",
                        value: stats.totalItems,
                        label: NSLocalizedString("home.stats.wishes", comment: "")
                    )
                    statChip(
                        icon: "gift.fill",
                        value: stats.claimedItems,
                        label: NSLocalizedString("home.stats.claimed", comment: "")
                    )
                    statChip(
                        icon: "person.2.fill",
                        value: stats.sharedItems,
                        label: NSLocalizedString("home.stats.shared", comment: "")
                    )
                    statChip(
                        icon: "checkmark.circle.fill",
                        value: stats.purchasedItems,
                        label: NSLocalizedString("home.stats.received", comment: "")
                    )
                }
                .padding(.horizontal, 2)
            }

            // Fade hints on both edges
            HStack {
                LinearGradient(
                    colors: [OffriiTheme.background, OffriiTheme.background.opacity(0)],
                    startPoint: .leading,
                    endPoint: .trailing
                )
                .frame(width: 12)

                Spacer()

                LinearGradient(
                    colors: [OffriiTheme.background.opacity(0), OffriiTheme.background],
                    startPoint: .leading,
                    endPoint: .trailing
                )
                .frame(width: 20)
            }
            .allowsHitTesting(false)
        }
    }

    private func statChip(icon: String, value: Int, label: String) -> some View {
        HStack(spacing: 6) {
            Image(systemName: icon)
                .font(.system(size: 12))
                .foregroundColor(OffriiTheme.primary)

            Text("\(value)")
                .font(.system(size: 15, weight: .bold))
                .foregroundColor(OffriiTheme.primary)

            Text(label.lowercased())
                .font(.system(size: 13))
                .foregroundColor(OffriiTheme.textSecondary)
        }
        .padding(.horizontal, OffriiTheme.spacingMD)
        .padding(.vertical, OffriiTheme.spacingSM)
        .background(OffriiTheme.primary.opacity(0.08))
        .cornerRadius(OffriiTheme.cornerRadiusFull)
    }
}
