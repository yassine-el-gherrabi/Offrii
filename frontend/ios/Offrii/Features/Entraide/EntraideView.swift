import SwiftUI

// MARK: - EntraideView

struct EntraideView: View {
    @Environment(AuthManager.self) private var authManager
    @State private var viewModel = EntraideViewModel()
    @State private var myNeedsViewModel = EntraideMyNeedsViewModel()
    @State private var selectedSegment = 0
    @State private var showCreateSheet = false
    @State private var selectedWishId: UUID?
    @State private var messagesWishId: UUID?
    @State private var reportWishId: UUID?
    @State private var searchQuery = ""
    @State private var sortField = "created_at"
    @State private var sortOrder = "desc"

    private var sortLabel: String {
        switch sortField {
        case "created_at": return NSLocalizedString("entraide.sort.date", comment: "")
        case "title":      return NSLocalizedString("entraide.sort.name", comment: "")
        default:           return NSLocalizedString("entraide.sort.date", comment: "")
        }
    }

    private var segmentLabel: String {
        switch selectedSegment {
        case 0:  return NSLocalizedString("entraide.segment.discover", comment: "")
        case 1:  return NSLocalizedString("entraide.segment.myNeeds", comment: "")
        default: return NSLocalizedString("entraide.segment.myOffers", comment: "")
        }
    }

    private var isCurrentSegmentLoading: Bool {
        switch selectedSegment {
        case 0:  return viewModel.isLoading
        case 1:  return myNeedsViewModel.isLoading
        default: return viewModel.isLoading
        }
    }

    private var displayCount: Int {
        switch selectedSegment {
        case 0:  return viewModel.filteredWishes.count
        case 1:  return myNeedsViewModel.wishes.count
        default: return viewModel.myOfferWishes.count
        }
    }

    var body: some View {
        ZStack(alignment: .bottomTrailing) {
            ScrollView {
                LazyVStack(spacing: 0, pinnedViews: .sectionHeaders) {
                    Section {
                        switch selectedSegment {
                        case 0:
                            EntraideDiscoverContent(
                                viewModel: viewModel,
                                selectedWishId: $selectedWishId,
                                showCreateSheet: $showCreateSheet,
                                searchQuery: searchQuery
                            )
                        case 1:
                            EntraideMyNeedsContent(
                                viewModel: myNeedsViewModel,
                                selectedWishId: $selectedWishId,
                                showCreateSheet: $showCreateSheet
                            )
                        case 2:
                            EntraideMyOffersContent(
                                viewModel: viewModel,
                                selectedWishId: $selectedWishId
                            )
                        default:
                            EmptyView()
                        }
                    } header: {
                        VStack(spacing: 0) {
                            Text(NSLocalizedString("entraide.subtitle", comment: ""))
                                .font(.system(size: 13, weight: .regular).italic())
                                .foregroundColor(OffriiTheme.textSecondary)
                                .frame(maxWidth: .infinity, alignment: .leading)
                                .padding(.horizontal, OffriiTheme.spacingBase)
                                .padding(.bottom, 2)

                            categoryChipsBar
                            statsBar
                        }
                        .background(OffriiTheme.background)
                    }
                }
            }
            .refreshable {
                switch selectedSegment {
                case 0:  await viewModel.loadWishes()
                case 1:  await myNeedsViewModel.loadMyWishes()
                default: await viewModel.loadMyOffers()
                }
            }

            // FAB
            if selectedSegment == 0 || selectedSegment == 1 {
                OffriiFloatingActionButton(icon: "plus") {
                    showCreateSheet = true
                }
                .padding(.trailing, OffriiTheme.spacingLG)
                .padding(.bottom, OffriiTheme.spacingLG)
            }
        }
        .background(OffriiTheme.background.ignoresSafeArea())
        .navigationTitle(NSLocalizedString("entraide.title", comment: ""))
        .navigationBarTitleDisplayMode(.large)
        .searchable(
            text: $searchQuery,
            placement: .navigationBarDrawer(displayMode: .automatic),
            prompt: NSLocalizedString("entraide.search.placeholder", comment: "")
        )
        .toolbar {
            ToolbarItemGroup(placement: .topBarTrailing) {
                NavigationLink(destination: ProfileView()) {
                    ProfileAvatarButton(
                        initials: ProfileAvatarButton.initials(
                            from: authManager.currentUser?.displayName
                        ),
                        avatarUrl: authManager.currentUser?.avatarUrl
                            .flatMap { URL(string: $0) }
                    )
                }
            }
        }
        .sheet(isPresented: $showCreateSheet, onDismiss: {
            Task {
                await viewModel.loadWishes()
                await viewModel.loadMyOffers()
                await myNeedsViewModel.loadMyWishes()
            }
        }) {
            CreateWishSheet()
                .presentationDetents([.large])
        }
        .sheet(item: $selectedWishId, onDismiss: {
            Task {
                await viewModel.loadWishes()
                await viewModel.loadMyOffers()
                await myNeedsViewModel.loadMyWishes()
            }
        }) { wishId in
            WishDetailSheet(
                wishId: wishId,
                onOpenMessages: { messagesWishId = wishId },
                onReport: { reportWishId = wishId }
            )
            .environment(authManager)
            .presentationDetents([.medium, .large])
        }
        .sheet(item: $messagesWishId) { wishId in
            WishMessagesSheet(wishId: wishId)
                .presentationDetents([.large])
        }
        .sheet(item: $reportWishId) { wishId in
            ReportWishSheet(wishId: wishId)
                .presentationDetents([.medium])
        }
        .task {
            await viewModel.loadWishes()
            await viewModel.loadMyOffers()
            await myNeedsViewModel.loadMyWishes()
        }
    }

    // MARK: - Category Chips (same as WishlistView pattern)

    private var categoryChipsBar: some View {
        ScrollView(.horizontal, showsIndicators: false) {
            HStack(spacing: OffriiTheme.spacingSM) {
                let allSelected = viewModel.selectedCategory == nil

                Button {
                    Task { await viewModel.selectCategory(nil) }
                } label: {
                    HStack(spacing: 4) {
                        Image(systemName: "sparkles")
                            .font(.system(size: 11))
                        Text(NSLocalizedString("entraide.category.all", comment: ""))
                            .font(.system(size: 13, weight: allSelected ? .semibold : .regular))
                    }
                    .foregroundColor(allSelected ? .white : OffriiTheme.textSecondary)
                    .padding(.horizontal, OffriiTheme.spacingMD)
                    .padding(.vertical, OffriiTheme.spacingSM)
                    .background(allSelected ? OffriiTheme.primary : .white)
                    .cornerRadius(OffriiTheme.cornerRadiusXL)
                    .overlay(
                        RoundedRectangle(cornerRadius: OffriiTheme.cornerRadiusXL)
                            .strokeBorder(allSelected ? .clear : OffriiTheme.border, lineWidth: 1)
                    )
                }
                .buttonStyle(.plain)

                ForEach(WishCategory.allCases) { category in
                    let isSelected = viewModel.selectedCategory == category
                    let color = entraideCategoryColor(category)

                    HStack(spacing: 4) {
                        Image(systemName: entraideCategoryIcon(category))
                            .font(.system(size: 11))
                        Text(category.label)
                            .font(.system(size: 13, weight: isSelected ? .semibold : .regular))
                    }
                    .foregroundColor(isSelected ? .white : OffriiTheme.textSecondary)
                    .padding(.horizontal, OffriiTheme.spacingMD)
                    .padding(.vertical, OffriiTheme.spacingSM)
                    .background(isSelected ? color : .white)
                    .cornerRadius(OffriiTheme.cornerRadiusXL)
                    .overlay(
                        RoundedRectangle(cornerRadius: OffriiTheme.cornerRadiusXL)
                            .strokeBorder(isSelected ? .clear : OffriiTheme.border, lineWidth: 1)
                    )
                    .onTapGesture {
                        Task { await viewModel.selectCategory(category) }
                    }
                    .animation(OffriiAnimation.snappy, value: isSelected)
                }
            }
            .padding(.horizontal, OffriiTheme.spacingBase)
            .padding(.vertical, OffriiTheme.spacingXS)
        }
    }

    // MARK: - Stats Bar (same pattern as WishlistView)

    private var statsBar: some View {
        HStack {
            HStack(spacing: 4) {
                if isCurrentSegmentLoading {
                    RoundedRectangle(cornerRadius: 3)
                        .fill(OffriiTheme.textMuted.opacity(0.2))
                        .frame(width: 20, height: 14)
                        .shimmer()
                } else {
                    Text("\(displayCount)")
                        .font(.system(size: 13, weight: .medium))
                        .foregroundColor(OffriiTheme.text)
                }
                Text(displayCount == 1
                    ? NSLocalizedString("entraide.countSingular", comment: "")
                    : NSLocalizedString("entraide.countPlural", comment: ""))
                    .font(.system(size: 13))
                    .foregroundColor(OffriiTheme.textMuted)

                Text("·").foregroundColor(OffriiTheme.textMuted)

                Menu {
                    Button {
                        if sortField == "created_at" {
                            sortOrder = sortOrder == "desc" ? "asc" : "desc"
                        } else {
                            sortField = "created_at"
                            sortOrder = "desc"
                        }
                        applySort()
                    } label: {
                        HStack {
                            Text(NSLocalizedString("entraide.sort.date", comment: ""))
                            if sortField == "created_at" {
                                Image(systemName: sortOrder == "desc" ? "chevron.down" : "chevron.up")
                            }
                        }
                    }

                    Button {
                        if sortField == "title" {
                            sortOrder = sortOrder == "asc" ? "desc" : "asc"
                        } else {
                            sortField = "title"
                            sortOrder = "asc"
                        }
                        applySort()
                    } label: {
                        HStack {
                            Text(NSLocalizedString("entraide.sort.name", comment: ""))
                            if sortField == "title" {
                                Image(systemName: sortOrder == "asc" ? "chevron.down" : "chevron.up")
                            }
                        }
                    }
                } label: {
                    HStack(spacing: 2) {
                        Text(sortLabel)
                            .font(.system(size: 13, weight: .medium))
                        Image(systemName: sortOrder == "desc" ? "chevron.down" : "chevron.up")
                            .font(.system(size: 10, weight: .semibold))
                    }
                    .foregroundColor(OffriiTheme.primary)
                }
            }

            Spacer()

            Picker("", selection: $selectedSegment) {
                Text(NSLocalizedString("entraide.segment.discover", comment: "")).tag(0)
                Text(NSLocalizedString("entraide.segment.myNeeds", comment: "")).tag(1)
                Text(NSLocalizedString("entraide.segment.myOffers", comment: "")).tag(2)
            }
            .pickerStyle(.segmented)
            .frame(width: 260)
        }
        .padding(.horizontal, OffriiTheme.spacingBase)
        .padding(.vertical, OffriiTheme.spacingXS)
    }

    // MARK: - Sort

    private func applySort() {
        viewModel.sortField = sortField
        viewModel.sortOrder = sortOrder
    }

    // MARK: - Category Helpers

    private func entraideCategoryColor(_ cat: WishCategory) -> Color {
        switch cat {
        case .education: return Color(red: 0.2, green: 0.4, blue: 0.85)
        case .clothing:  return Color(red: 0.7, green: 0.3, blue: 0.6)
        case .health:    return Color(red: 0.85, green: 0.3, blue: 0.35)
        case .religion:  return Color(red: 0.55, green: 0.4, blue: 0.75)
        case .home:      return Color(red: 0.9, green: 0.5, blue: 0.2)
        case .children:  return Color(red: 0.3, green: 0.7, blue: 0.6)
        case .other:     return Color(red: 0.5, green: 0.5, blue: 0.6)
        }
    }

    private func entraideCategoryIcon(_ cat: WishCategory) -> String {
        switch cat {
        case .education: return "book.fill"
        case .clothing:  return "tshirt.fill"
        case .health:    return "heart.fill"
        case .religion:  return "hands.sparkles.fill"
        case .home:      return "house.fill"
        case .children:  return "figure.and.child.holdinghands"
        case .other:     return "tag.fill"
        }
    }
}
