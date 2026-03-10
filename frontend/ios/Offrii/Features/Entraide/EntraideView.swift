import SwiftUI

// MARK: - EntraideView

struct EntraideView: View {
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
            OffriiTheme.cardSurface.ignoresSafeArea()

            VStack(spacing: 0) {
                // Header
                headerView

                // Category chips
                categoryChips
                    .padding(.vertical, OffriiTheme.spacingSM)

                // Feed
                feedContent
            }

            // FAB
            VStack {
                Spacer()
                HStack {
                    Spacer()
                    fabButton
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
        }
    }

    // MARK: - Header

    private var headerView: some View {
        ZStack {
            OffriiTheme.primary
                .ignoresSafeArea(edges: .top)

            DecorativeSquares(preset: .header)

            HStack(alignment: .bottom) {
                VStack(alignment: .leading, spacing: OffriiTheme.spacingXS) {
                    Text("entraide.title")
                        .font(OffriiTypography.largeTitle)
                        .foregroundColor(.white)

                    Text("entraide.subtitle")
                        .font(OffriiTypography.subheadline)
                        .foregroundColor(.white.opacity(0.8))
                }

                Spacer()

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
            .padding(.horizontal, OffriiTheme.spacingLG)
            .padding(.bottom, OffriiTheme.spacingLG)
            .padding(.top, OffriiTheme.spacingXL)
        }
        .frame(minHeight: 140)
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
                            .padding(.horizontal, OffriiTheme.spacingMD)
                            .padding(.vertical, OffriiTheme.spacingSM)
                            .background(isSelected ? OffriiTheme.primary : OffriiTheme.card)
                            .cornerRadius(OffriiTheme.cornerRadiusXL)
                    }
                    .buttonStyle(.plain)
                    .animation(OffriiTheme.defaultAnimation, value: isSelected)
                }
            }
            .padding(.horizontal, OffriiTheme.spacingMD)
        }
    }

    // MARK: - Feed Content

    @ViewBuilder
    private var feedContent: some View {
        if viewModel.isLoading {
            Spacer()
            ProgressView()
                .frame(maxWidth: .infinity)
            Spacer()
        } else if viewModel.wishes.isEmpty {
            Spacer()
            emptyState
            Spacer()
        } else {
            ScrollView {
                LazyVStack(spacing: OffriiTheme.spacingMD) {
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
                        ProgressView()
                            .padding(OffriiTheme.spacingMD)
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

    // MARK: - Empty State

    private var emptyState: some View {
        VStack(spacing: OffriiTheme.spacingMD) {
            Image(systemName: "hand.raised.fill")
                .font(.system(size: 48))
                .foregroundColor(OffriiTheme.textMuted)

            Text("entraide.empty")
                .font(OffriiTypography.title3)
                .foregroundColor(OffriiTheme.text)

            Text("entraide.emptySubtitle")
                .font(OffriiTypography.subheadline)
                .foregroundColor(OffriiTheme.textSecondary)
                .multilineTextAlignment(.center)
                .padding(.horizontal, OffriiTheme.spacingXL)
        }
    }

    // MARK: - FAB

    private var fabButton: some View {
        Button {
            showCreateSheet = true
        } label: {
            HStack(spacing: OffriiTheme.spacingSM) {
                Image(systemName: "plus")
                    .font(.system(size: 16, weight: .bold))
                Text("entraide.fab.publish")
                    .font(OffriiTypography.footnote)
                    .fontWeight(.semibold)
            }
            .foregroundColor(.white)
            .padding(.horizontal, OffriiTheme.spacingMD)
            .padding(.vertical, OffriiTheme.spacingSM + 4)
            .background(OffriiTheme.primary)
            .cornerRadius(OffriiTheme.cornerRadiusXL)
            .shadow(color: OffriiTheme.primary.opacity(0.4), radius: 8, x: 0, y: 4)
        }
        .buttonStyle(.plain)
        .accessibilityLabel(Text("entraide.fab.publish"))
    }
}
