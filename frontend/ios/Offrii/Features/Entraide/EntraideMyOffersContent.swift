import SwiftUI

// MARK: - My Offers Content

struct EntraideMyOffersContent: View {
    var viewModel: EntraideViewModel
    @Binding var selectedWishId: UUID?
    @Binding var messagesWishId: UUID?
    var selectedCategory: WishCategory?
    @State private var wishToWithdraw: UUID?

    private var displayedOffers: [CommunityWish] {
        guard let category = selectedCategory else { return viewModel.myOfferWishes }
        return viewModel.myOfferWishes.filter { $0.category == category }
    }

    var body: some View {
        if viewModel.isLoadingOffers && viewModel.myOfferWishes.isEmpty {
            LazyVStack(spacing: OffriiTheme.spacingSM) {
                ForEach(0..<4, id: \.self) { _ in
                    SkeletonRow()
                }
            }
            .padding(.horizontal, OffriiTheme.spacingBase)
            .padding(.vertical, OffriiTheme.spacingSM)
        } else if displayedOffers.isEmpty {
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
                ForEach(displayedOffers) { wish in
                    EntraideWishCard(wish: wish) {
                        selectedWishId = wish.id
                    }
                    .contextMenu {
                        offersContextMenu(wish)
                    }
                }
            }
            .padding(.horizontal, OffriiTheme.spacingBase)
            .padding(.vertical, OffriiTheme.spacingSM)
            .alert(
                NSLocalizedString("entraide.withdraw.confirmTitle", comment: ""),
                isPresented: Binding(
                    get: { wishToWithdraw != nil },
                    set: { if !$0 { wishToWithdraw = nil } }
                )
            ) {
                Button(NSLocalizedString("common.cancel", comment: ""), role: .cancel) {
                    wishToWithdraw = nil
                }
                Button(NSLocalizedString("entraide.action.withdraw", comment: ""), role: .destructive) {
                    if let id = wishToWithdraw {
                        Task {
                            try? await CommunityWishService.shared.withdrawOffer(id: id)
                            OffriiHaptics.success()
                            await viewModel.loadMyOffers()
                            await viewModel.loadWishes()
                        }
                    }
                    wishToWithdraw = nil
                }
            } message: {
                Text(NSLocalizedString("entraide.withdraw.confirmMessage", comment: ""))
            }
        }
    }

    // MARK: - Context Menu (My Offers)

    @ViewBuilder
    private func offersContextMenu(_ wish: CommunityWish) -> some View {
        if wish.status == .matched {
            Button {
                messagesWishId = wish.id
            } label: {
                Label(
                    NSLocalizedString("entraide.action.messages", comment: ""),
                    systemImage: "bubble.left"
                )
            }
        }

        Button {
            selectedWishId = wish.id
        } label: {
            Label(
                NSLocalizedString("entraide.action.viewDetail", comment: ""),
                systemImage: "eye"
            )
        }

        if wish.status == .matched {
            Divider()
            Button(role: .destructive) {
                wishToWithdraw = wish.id
            } label: {
                Label(
                    NSLocalizedString("entraide.action.withdraw", comment: ""),
                    systemImage: "arrow.uturn.backward"
                )
            }
        }
    }
}
