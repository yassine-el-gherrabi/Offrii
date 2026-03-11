import SwiftUI

// MARK: - EntraideView

struct EntraideView: View {
    @Environment(OnboardingTipManager.self) private var tipManager
    @State private var viewModel = EntraideViewModel()
    @State private var showCreateSheet = false

    // Category filter items for chips
    private var categoryChipItems: [(label: String, category: WishCategory?)] {
        var items: [(label: String, category: WishCategory?)] = [
            (NSLocalizedString("entraide.category.all", comment: ""), nil),
        ]
        for cat in WishCategory.allCases {
            items.append((cat.chipLabel, cat))
        }
        return items
    }

    var body: some View {
        ZStack {
            OffriiTheme.background.ignoresSafeArea()

            VStack(spacing: 0) {
                // Header
                SectionHeader(
                    title: NSLocalizedString("entraide.title", comment: ""),
                    subtitle: NSLocalizedString("entraide.subtitle", comment: ""),
                    variant: .entraide
                ) {
                    NavigationLink {
                        MyWishesView()
                    } label: {
                        Image(systemName: "list.bullet")
                            .font(.system(size: 18, weight: .semibold))
                            .foregroundColor(.white)
                            .frame(width: 36, height: 36)
                            .background(Color.white.opacity(0.2))
                            .cornerRadius(OffriiTheme.cornerRadiusSM)
                    }
                }

                // Category chips
                categoryChips
                    .padding(.vertical, OffriiTheme.spacingSM)
                    .overlay(alignment: .bottom) {
                        if tipManager.activeTip == .entraideBrowse {
                            OffriiTooltip(
                                message: OnboardingTipManager.message(for: .entraideBrowse),
                                arrow: .top
                            ) {
                                tipManager.dismiss(.entraideBrowse)
                            }
                            .offset(y: 50)
                        }
                    }

                // Feed
                feedContent
            }

            // FAB
            VStack {
                Spacer()
                HStack {
                    Spacer()
                    OffriiFloatingActionButton(
                        icon: "plus",
                        label: NSLocalizedString("entraide.fab.publish", comment: "")
                    ) {
                        showCreateSheet = true
                    }
                    .padding(.trailing, OffriiTheme.spacingLG)
                    .padding(.bottom, OffriiTheme.spacingSM)
                }
            }
        }
        .navigationBarHidden(true)
        .sheet(isPresented: $showCreateSheet) {
            CreateWishSheet {
                Task { await viewModel.loadWishes() }
            }
            .presentationDetents([.large])
        }
        .task {
            await viewModel.loadWishes()
            tipManager.showIfNeeded(.entraideBrowse)
        }
    }

    // MARK: - Category Chips

    private var categoryChips: some View {
        ScrollView(.horizontal, showsIndicators: false) {
            HStack(spacing: OffriiTheme.spacingSM) {
                ForEach(Array(categoryChipItems.enumerated()), id: \.offset) { _, item in
                    let isSelected = viewModel.selectedCategory == item.category
                    Button {
                        viewModel.selectCategory(item.category)
                    } label: {
                        Text(item.label)
                            .font(OffriiTypography.footnote)
                            .fontWeight(isSelected ? .semibold : .regular)
                            .foregroundColor(isSelected ? .white : OffriiTheme.textSecondary)
                            .padding(.horizontal, OffriiTheme.spacingBase)
                            .padding(.vertical, OffriiTheme.spacingSM)
                            .background(isSelected ? OffriiTheme.primary : OffriiTheme.card)
                            .cornerRadius(OffriiTheme.cornerRadiusXL)
                    }
                    .buttonStyle(.plain)
                    .animation(OffriiAnimation.defaultSpring, value: isSelected)
                }
            }
            .padding(.horizontal, OffriiTheme.spacingBase)
        }
    }

    // MARK: - Feed Content

    @ViewBuilder
    private var feedContent: some View {
        if viewModel.isLoading {
            ScrollView {
                LazyVStack(spacing: OffriiTheme.spacingBase) {
                    ForEach(0..<4, id: \.self) { _ in
                        SkeletonCard()
                    }
                }
                .padding(.horizontal, OffriiTheme.spacingLG)
                .padding(.vertical, OffriiTheme.spacingSM)
            }
        } else if viewModel.wishes.isEmpty {
            Spacer()
            OffriiEmptyState(
                icon: "hand.raised.fill",
                title: NSLocalizedString("entraide.empty", comment: ""),
                subtitle: NSLocalizedString("entraide.emptySubtitle", comment: "")
            )
            Spacer()
        } else {
            ScrollView {
                LazyVStack(spacing: OffriiTheme.spacingBase) {
                    ForEach(viewModel.wishes) { wish in
                        NavigationLink {
                            WishDetailView(wishId: wish.id)
                        } label: {
                            WishCard(wish: wish) {
                                Task { _ = await viewModel.offerWish(id: wish.id) }
                            }
                        }
                        .buttonStyle(.plain)
                        .onAppear {
                            Task { await viewModel.loadMoreIfNeeded(currentWish: wish) }
                        }
                    }

                    if viewModel.isLoadingMore {
                        SkeletonCard()
                            .padding(.horizontal, OffriiTheme.spacingBase)
                    }
                }
                .padding(.horizontal, OffriiTheme.spacingLG)
                .padding(.vertical, OffriiTheme.spacingSM)
            }
            .refreshable {
                await viewModel.loadWishes()
            }
        }
    }
}
