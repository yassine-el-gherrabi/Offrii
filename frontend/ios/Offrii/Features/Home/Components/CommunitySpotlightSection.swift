import SwiftUI

// MARK: - CommunitySpotlightSection

struct CommunitySpotlightSection: View {
    let wishes: [CommunityWish]
    @Binding var selectedWishId: UUID?

    var body: some View {
        VStack(alignment: .leading, spacing: OffriiTheme.spacingSM) {
            // Section title + "See all"
            HStack {
                Text(NSLocalizedString("home.community.title", comment: ""))
                    .font(OffriiTypography.headline)
                    .foregroundColor(OffriiTheme.text)

                Spacer()

                NavigationLink {
                    EntraideView()
                } label: {
                    HStack(spacing: OffriiTheme.spacingXXS) {
                        Text(NSLocalizedString("home.community.seeAll", comment: ""))
                            .font(OffriiTypography.subheadline)
                        Image(systemName: "arrow.right")
                            .font(.system(size: 12))
                    }
                    .foregroundColor(OffriiTheme.primary)
                }
            }

            if wishes.isEmpty {
                OffriiEmptyState(
                    icon: "hand.raised.fill",
                    title: NSLocalizedString("entraide.empty", comment: ""),
                    subtitle: NSLocalizedString("entraide.emptySubtitle", comment: "")
                )
                .padding(.vertical, OffriiTheme.spacingBase)
            } else {
                VStack(spacing: OffriiTheme.spacingSM) {
                    ForEach(wishes) { wish in
                        EntraideWishCard(wish: wish) {
                            selectedWishId = wish.id
                        }
                    }
                }
            }
        }
    }
}
