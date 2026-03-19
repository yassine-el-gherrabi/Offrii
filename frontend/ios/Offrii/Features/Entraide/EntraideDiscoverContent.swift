import SwiftUI

// MARK: - Discover Content (grid only — no ScrollView, parent handles it)

struct EntraideDiscoverContent: View {
    var viewModel: EntraideViewModel
    @Binding var selectedWishId: UUID?
    var searchQuery: String

    private let gridColumns = [
        GridItem(.flexible(), spacing: OffriiTheme.spacingSM),
        GridItem(.flexible(), spacing: OffriiTheme.spacingSM)
    ]

    private var displayedWishes: [CommunityWish] {
        if searchQuery.isEmpty {
            return viewModel.filteredWishes
        }
        return viewModel.filteredWishes.filter {
            $0.title.localizedCaseInsensitiveContains(searchQuery)
        }
    }

    var body: some View {
        if viewModel.isLoading && viewModel.wishes.isEmpty {
            skeletonGrid
        } else if displayedWishes.isEmpty {
            VStack(spacing: OffriiTheme.spacingBase) {
                Spacer().frame(height: 40)
                OffriiEmptyState(
                    icon: "heart.circle",
                    title: NSLocalizedString("entraide.empty", comment: ""),
                    subtitle: NSLocalizedString("entraide.emptySubtitle", comment: "")
                )
                Spacer()
            }
        } else {
            LazyVGrid(columns: gridColumns, spacing: OffriiTheme.spacingSM) {
                ForEach(displayedWishes) { wish in
                    EntraideWishCard(wish: wish) {
                        selectedWishId = wish.id
                    }
                    .onAppear {
                        Task { await viewModel.loadMoreIfNeeded(currentWish: wish) }
                    }
                }
            }
            .padding(.horizontal, OffriiTheme.spacingBase)
            .padding(.vertical, OffriiTheme.spacingSM)
        }
    }

    private var skeletonGrid: some View {
        LazyVGrid(columns: gridColumns, spacing: OffriiTheme.spacingSM) {
            ForEach(0..<6, id: \.self) { _ in
                SkeletonGridCard()
            }
        }
        .padding(.horizontal, OffriiTheme.spacingBase)
        .padding(.vertical, OffriiTheme.spacingSM)
    }
}
