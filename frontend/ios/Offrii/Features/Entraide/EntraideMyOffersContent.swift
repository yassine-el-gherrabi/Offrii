import SwiftUI

// MARK: - My Offers Content

struct EntraideMyOffersContent: View {
    var viewModel: EntraideViewModel
    @Binding var selectedWishId: UUID?

    private let gridColumns = [
        GridItem(.flexible(), spacing: OffriiTheme.spacingSM),
        GridItem(.flexible(), spacing: OffriiTheme.spacingSM)
    ]

    var body: some View {
        if viewModel.isLoading && viewModel.wishes.isEmpty {
            ScrollView {
                LazyVGrid(columns: gridColumns, spacing: OffriiTheme.spacingSM) {
                    ForEach(0..<4, id: \.self) { _ in
                        SkeletonGridCard()
                    }
                }
                .padding(.horizontal, OffriiTheme.spacingBase)
                .padding(.vertical, OffriiTheme.spacingSM)
            }
        } else if viewModel.myOffers.isEmpty {
            Spacer()
            OffriiEmptyState(
                icon: "hand.raised",
                title: NSLocalizedString("entraide.myOffers.empty", comment: ""),
                subtitle: NSLocalizedString("entraide.myOffers.emptySubtitle", comment: "")
            )
            Spacer()
        } else {
            ScrollView {
                LazyVGrid(columns: gridColumns, spacing: OffriiTheme.spacingSM) {
                    ForEach(viewModel.myOffers) { wish in
                        EntraideWishCard(wish: wish) {
                            selectedWishId = wish.id
                        }
                    }
                }
                .padding(.horizontal, OffriiTheme.spacingBase)
                .padding(.vertical, OffriiTheme.spacingSM)
            }
            .refreshable {
                await viewModel.loadWishes()
            }
        }
    }
}
