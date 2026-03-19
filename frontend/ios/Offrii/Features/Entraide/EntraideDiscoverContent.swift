import SwiftUI

// MARK: - Discover Content (grid only — no ScrollView, parent handles it)

struct EntraideDiscoverContent: View {
    var viewModel: EntraideViewModel
    @Binding var selectedWishId: UUID?
    @Binding var showCreateSheet: Bool
    @Binding var reportWishId: UUID?
    var searchQuery: String
    @AppStorage("entraide.onboarding.dismissed") private var onboardingDismissed = false
    @State private var recentFulfilled: [CommunityWish] = []

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

                // Recently fulfilled section
                if !recentFulfilled.isEmpty {
                    VStack(alignment: .leading, spacing: OffriiTheme.spacingSM) {
                        HStack(spacing: 6) {
                            Image(systemName: "hands.clap.fill")
                                .font(.system(size: 14))
                                .foregroundColor(OffriiTheme.warning)
                            Text(NSLocalizedString("entraide.recentFulfilled.title", comment: ""))
                                .font(OffriiTypography.subheadline)
                                .fontWeight(.semibold)
                                .foregroundColor(OffriiTheme.text)
                        }

                        ScrollView(.horizontal, showsIndicators: false) {
                            HStack(spacing: 10) {
                                ForEach(recentFulfilled) { wish in
                                    Button {
                                        OffriiHaptics.tap()
                                        selectedWishId = wish.id
                                    } label: {
                                        HStack(spacing: 10) {
                                            Image(systemName: wish.category.icon)
                                                .font(.system(size: 14))
                                                .foregroundColor(.white)
                                                .frame(width: 32, height: 32)
                                                .background(OffriiTheme.warning)
                                                .clipShape(RoundedRectangle(cornerRadius: 8))

                                            VStack(alignment: .leading, spacing: 2) {
                                                Text(wish.title)
                                                    .font(.system(size: 13, weight: .semibold))
                                                    .foregroundColor(OffriiTheme.text)
                                                    .lineLimit(1)

                                                HStack(spacing: 4) {
                                                    Image(systemName: "checkmark.circle.fill")
                                                        .font(.system(size: 10))
                                                        .foregroundColor(OffriiTheme.warning)
                                                    Text(NSLocalizedString("entraide.status.fulfilled", comment: ""))
                                                        .font(.system(size: 11, weight: .medium))
                                                        .foregroundColor(OffriiTheme.warning)

                                                    if let fulfilledAt = wish.fulfilledAt {
                                                        Text("·")
                                                            .foregroundColor(OffriiTheme.textMuted)
                                                        Text(fulfilledAt, style: .relative)
                                                            .foregroundColor(OffriiTheme.textMuted)
                                                    }
                                                }
                                                .font(.system(size: 11))
                                            }
                                        }
                                        .padding(.horizontal, 12)
                                        .padding(.vertical, 10)
                                        .background(OffriiTheme.warning.opacity(0.06))
                                        .cornerRadius(OffriiTheme.cornerRadiusMD)
                                        .overlay(
                                            RoundedRectangle(cornerRadius: OffriiTheme.cornerRadiusMD)
                                                .strokeBorder(OffriiTheme.warning.opacity(0.15), lineWidth: 1)
                                        )
                                    }
                                    .buttonStyle(.plain)
                                }
                            }
                        }
                    }
                    .padding(.bottom, OffriiTheme.spacingSM)
                }

                ForEach(displayedWishes) { wish in
                    EntraideWishCard(wish: wish) {
                        selectedWishId = wish.id
                    }
                    .contextMenu {
                        discoverContextMenu(wish)
                    }
                    .onAppear {
                        Task { await viewModel.loadMoreIfNeeded(currentWish: wish) }
                    }
                }
            }
            .padding(.horizontal, OffriiTheme.spacingBase)
            .padding(.vertical, OffriiTheme.spacingSM)
            .task {
                recentFulfilled = (try? await CommunityWishService.shared.listRecentFulfilled()) ?? []
            }
        }
    }

    // MARK: - Context Menu (Discover)

    @ViewBuilder
    private func discoverContextMenu(_ wish: CommunityWish) -> some View {
        if !wish.isMine && wish.status == .open && !wish.isMatchedByMe {
            Button {
                Task {
                    if await viewModel.offerWish(id: wish.id) {
                        await viewModel.loadWishes()
                    }
                }
            } label: {
                Label(
                    NSLocalizedString("entraide.action.offer", comment: ""),
                    systemImage: "hand.raised.fill"
                )
            }
        }

        if !wish.isMine {
            Button(role: .destructive) {
                reportWishId = wish.id
            } label: {
                Label(
                    NSLocalizedString("entraide.action.report", comment: ""),
                    systemImage: "exclamationmark.triangle"
                )
            }
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
