import SwiftUI

// MARK: - MyOffersSection

struct MyOffersSection: View {
    @State private var viewModel = EntraideViewModel()
    @State private var selectedWishId: UUID?

    private let gridColumns = [
        GridItem(.flexible(), spacing: OffriiTheme.spacingSM),
        GridItem(.flexible(), spacing: OffriiTheme.spacingSM)
    ]

    /// Filter loaded wishes to only those where isMatchedByMe == true
    private var myOffers: [CommunityWish] {
        viewModel.wishes.filter { $0.isMatchedByMe }
    }

    var body: some View {
        Group {
            if viewModel.isLoading {
                ScrollView {
                    LazyVGrid(columns: gridColumns, spacing: OffriiTheme.spacingSM) {
                        ForEach(0..<4, id: \.self) { _ in
                            SkeletonGridCard()
                        }
                    }
                    .padding(.horizontal, OffriiTheme.spacingBase)
                    .padding(.vertical, OffriiTheme.spacingSM)
                }
            } else if myOffers.isEmpty {
                Spacer()
                OffriiEmptyState(
                    icon: "hand.raised.fill",
                    title: NSLocalizedString("entraide.myOffers.empty", comment: ""),
                    subtitle: NSLocalizedString("entraide.myOffers.emptySubtitle", comment: "")
                )
                Spacer()
            } else {
                ScrollView {
                    LazyVGrid(columns: gridColumns, spacing: OffriiTheme.spacingSM) {
                        ForEach(myOffers) { wish in
                            EntraideGridCard(wish: wish) {
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
        .sheet(item: $selectedWishId) { wishId in
            WishDetailSheet(wishId: wishId)
                .presentationDetents([.medium, .large])
        }
        .task {
            await viewModel.loadWishes()
        }
    }
}
