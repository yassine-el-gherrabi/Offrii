import SwiftUI

// MARK: - Discover Content (grid only — no ScrollView, parent handles it)

struct EntraideDiscoverContent: View {
    var viewModel: EntraideViewModel
    @Binding var selectedWishId: UUID?
    @Binding var showCreateSheet: Bool
    var searchQuery: String

    private var displayedWishes: [CommunityWish] {
        if searchQuery.isEmpty {
            return viewModel.filteredWishes
        }
        return viewModel.filteredWishes.filter {
            $0.title.localizedCaseInsensitiveContains(searchQuery)
                || ($0.description?.localizedCaseInsensitiveContains(searchQuery) ?? false)
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
                    subtitle: NSLocalizedString("entraide.emptySubtitle", comment: ""),
                    ctaTitle: NSLocalizedString("entraide.fab.publish", comment: ""),
                    ctaAction: { showCreateSheet = true }
                )
                Spacer()
            }
        } else {
            LazyVStack(spacing: OffriiTheme.spacingSM) {
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
        LazyVStack(spacing: OffriiTheme.spacingSM) {
            ForEach(0..<5, id: \.self) { _ in
                SkeletonRow()
            }
        }
        .padding(.horizontal, OffriiTheme.spacingBase)
        .padding(.vertical, OffriiTheme.spacingSM)
    }
}
