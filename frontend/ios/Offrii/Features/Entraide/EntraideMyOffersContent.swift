import SwiftUI

// MARK: - My Offers Content

struct EntraideMyOffersContent: View {
    var viewModel: EntraideViewModel
    @Binding var selectedWishId: UUID?

    var body: some View {
        if viewModel.isLoadingOffers && viewModel.myOfferWishes.isEmpty {
            LazyVStack(spacing: OffriiTheme.spacingSM) {
                ForEach(0..<4, id: \.self) { _ in
                    SkeletonRow()
                }
            }
            .padding(.horizontal, OffriiTheme.spacingBase)
            .padding(.vertical, OffriiTheme.spacingSM)
        } else if viewModel.myOfferWishes.isEmpty {
            VStack(spacing: OffriiTheme.spacingBase) {
                Spacer().frame(height: 40)
                OffriiEmptyState(
                    icon: "hand.raised",
                    title: NSLocalizedString("entraide.myOffers.empty", comment: ""),
                    subtitle: NSLocalizedString("entraide.myOffers.emptySubtitle", comment: "")
                )
                Spacer()
            }
        } else {
            LazyVStack(spacing: OffriiTheme.spacingSM) {
                ForEach(viewModel.myOfferWishes) { wish in
                    EntraideWishCard(wish: wish) {
                        selectedWishId = wish.id
                    }
                }
            }
            .padding(.horizontal, OffriiTheme.spacingBase)
            .padding(.vertical, OffriiTheme.spacingSM)
        }
    }
}
