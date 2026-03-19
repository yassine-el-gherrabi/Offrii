import SwiftUI

// MARK: - Discover Content (grid only — no ScrollView, parent handles it)

struct EntraideDiscoverContent: View {
    var viewModel: EntraideViewModel
    @Binding var selectedWishId: UUID?
    @Binding var showCreateSheet: Bool
    var searchQuery: String
    @AppStorage("entraide.onboarding.dismissed") private var onboardingDismissed = false

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
                // Onboarding card (first visit only)
                if !onboardingDismissed {
                    VStack(alignment: .leading, spacing: OffriiTheme.spacingSM) {
                        HStack {
                            Image(systemName: "hand.wave.fill")
                                .font(.system(size: 20))
                                .foregroundColor(OffriiTheme.primary)
                            Text(NSLocalizedString("entraide.onboarding.title", comment: ""))
                                .font(OffriiTypography.headline)
                                .foregroundColor(OffriiTheme.text)
                            Spacer()
                            Button {
                                withAnimation { onboardingDismissed = true }
                            } label: {
                                Image(systemName: "xmark")
                                    .font(.system(size: 12, weight: .semibold))
                                    .foregroundColor(OffriiTheme.textMuted)
                            }
                        }
                        Text(NSLocalizedString("entraide.onboarding.body", comment: ""))
                            .font(OffriiTypography.caption)
                            .foregroundColor(OffriiTheme.textSecondary)
                    }
                    .padding(OffriiTheme.spacingBase)
                    .background(OffriiTheme.primary.opacity(0.05))
                    .cornerRadius(OffriiTheme.cornerRadiusLG)
                    .overlay(
                        RoundedRectangle(cornerRadius: OffriiTheme.cornerRadiusLG)
                            .strokeBorder(OffriiTheme.primary.opacity(0.2), lineWidth: 1)
                    )
                }

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
