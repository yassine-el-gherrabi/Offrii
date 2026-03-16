import SwiftUI

// MARK: - EntraideView

struct EntraideView: View {
    @Environment(AuthManager.self) private var authManager
    @Environment(OnboardingTipManager.self) private var tipManager
    @State private var viewModel = EntraideViewModel()
    @State private var showCreateSheet = false
    @State private var selectedSegment: EntraideSegment = .discover
    @State private var selectedWishId: UUID?

    private let gridColumns = [
        GridItem(.flexible(), spacing: OffriiTheme.spacingSM),
        GridItem(.flexible(), spacing: OffriiTheme.spacingSM)
    ]

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
                    NavigationLink(destination: ProfileView()) {
                        ProfileAvatarButton(
                            initials: ProfileAvatarButton.initials(from: authManager.currentUser?.displayName),
                            avatarUrl: authManager.currentUser?.avatarUrl.flatMap { URL(string: $0) }
                        )
                    }
                }

                // 3-segment picker
                Picker("", selection: $selectedSegment) {
                    ForEach(EntraideSegment.allCases, id: \.rawValue) { segment in
                        Text(segment.label).tag(segment)
                    }
                }
                .pickerStyle(.segmented)
                .padding(.horizontal, OffriiTheme.spacingLG)
                .padding(.vertical, OffriiTheme.spacingSM)

                // Content based on selected segment
                switch selectedSegment {
                case .discover:
                    discoverContent
                case .myNeeds:
                    myNeedsContent
                case .myOffers:
                    MyOffersSection()
                }
            }

            // FAB (only on discover segment)
            if selectedSegment == .discover {
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
        }
        .navigationBarHidden(true)
        .sheet(isPresented: $showCreateSheet) {
            CreateWishSheet {
                Task { await viewModel.loadWishes() }
            }
            .presentationDetents([.large])
        }
        .sheet(item: $selectedWishId) { wishId in
            WishDetailSheet(wishId: wishId)
                .presentationDetents([.medium, .large])
        }
        .task {
            await viewModel.loadWishes()
            tipManager.showIfNeeded(.entraideBrowse)
        }
    }

    // MARK: - Discover Content

    @ViewBuilder
    private var discoverContent: some View {
        VStack(spacing: 0) {
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

            // Feed grid
            if viewModel.isLoading {
                ScrollView {
                    LazyVGrid(columns: gridColumns, spacing: OffriiTheme.spacingSM) {
                        ForEach(0..<6, id: \.self) { _ in
                            SkeletonGridCard()
                        }
                    }
                    .padding(.horizontal, OffriiTheme.spacingBase)
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
                    LazyVGrid(columns: gridColumns, spacing: OffriiTheme.spacingSM) {
                        ForEach(viewModel.wishes) { wish in
                            EntraideGridCard(wish: wish) {
                                selectedWishId = wish.id
                            }
                            .onAppear {
                                Task { await viewModel.loadMoreIfNeeded(currentWish: wish) }
                            }
                        }

                        if viewModel.isLoadingMore {
                            SkeletonGridCard()
                            SkeletonGridCard()
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

    // MARK: - My Needs Content (inline MyWishesView)

    @ViewBuilder
    private var myNeedsContent: some View {
        MyWishesView()
    }

    // MARK: - Category Chips

    private var categoryChips: some View {
        ScrollView(.horizontal, showsIndicators: false) {
            HStack(spacing: OffriiTheme.spacingSM) {
                ForEach(Array(categoryChipItems.enumerated()), id: \.offset) { _, item in
                    let isSelected = viewModel.selectedCategory == item.category

                    OffriiChip(
                        title: item.label,
                        isSelected: isSelected,
                        backgroundColor: item.category?.backgroundColor,
                        textColor: item.category?.textColor
                    ) {
                        viewModel.selectCategory(item.category)
                    }
                }
            }
            .padding(.horizontal, OffriiTheme.spacingBase)
        }
    }
}
