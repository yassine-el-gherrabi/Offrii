import SwiftUI

// MARK: - HomeView

struct HomeView: View {
    @State private var vm = HomeViewModel()
    @State private var showNotificationCenter = false
    @State private var selectedItemId: UUID?
    @State private var selectedWishId: UUID?
    @State private var showQuickAdd = false
    @Environment(AuthManager.self) private var authManager
    @Environment(AppRouter.self) private var router

    private let gridColumns = [
        GridItem(.flexible(), spacing: OffriiTheme.spacingSM),
        GridItem(.flexible(), spacing: OffriiTheme.spacingSM)
    ]

    // MARK: - Greeting

    private var greeting: String {
        if let name = authManager.currentUser?.displayName, !name.isEmpty {
            return String(format: NSLocalizedString("home.greeting", comment: ""), name)
        }
        return NSLocalizedString("home.greetingDefault", comment: "")
    }

    private var subtitleText: String {
        if vm.stats.claimedItems > 0 {
            return String(
                format: NSLocalizedString("home.subtitle.claimed", comment: ""),
                vm.stats.claimedItems
            )
        }
        if vm.stats.sharedItems > 0 {
            return String(
                format: NSLocalizedString("home.subtitle.shared", comment: ""),
                vm.stats.sharedItems
            )
        }
        if vm.stats.totalItems > 0 {
            return String(
                format: NSLocalizedString("home.subtitle.hasItems", comment: ""),
                vm.stats.totalItems
            )
        }
        return NSLocalizedString("home.subtitle.empty", comment: "")
    }

    private var subtitleIcon: String {
        if vm.stats.claimedItems > 0 { return "gift.fill" }
        if vm.stats.sharedItems > 0 { return "person.2.fill" }
        if vm.stats.totalItems > 0 { return "heart.fill" }
        return "plus.circle"
    }

    // MARK: - Body

    var body: some View {
        ScrollView {
            VStack(spacing: OffriiTheme.spacingLG) {
                if vm.isLoading {
                    homeSkeletonView
                } else {

                // Contextual subtitle under the greeting
                HStack(spacing: 6) {
                    Image(systemName: subtitleIcon)
                        .font(.system(size: 13))
                        .foregroundColor(OffriiTheme.primary)
                    Text(subtitleText)
                        .font(.system(size: 15))
                        .foregroundColor(OffriiTheme.textSecondary)
                }
                .frame(maxWidth: .infinity, alignment: .leading)
                .padding(.top, -OffriiTheme.spacingSM)

                // Section 1: Profile progress (hidden while loading, hidden at 100%)
                if vm.profileProgress.percentage < 100 {
                    ProfileProgressCard(progress: vm.profileProgress) {
                        Task { await vm.load(authManager: authManager) }
                    }
                    .transition(.opacity.combined(with: .move(edge: .top)))
                    .animation(OffriiAnimation.defaultSpring, value: vm.profileProgress.percentage)
                }

                // Section 2: Stats chips
                HomeStatsCard(stats: vm.stats)

                // Section 3: Wishlist grid preview (content first!)
                wishlistGridSection

                // Section 4: Quick actions (secondary, below content)
                HomeQuickActionsSection()

                // Section 5: Activity feed
                if !vm.sanitizedNotifications.isEmpty {
                    HomeActivitySection(notifications: vm.sanitizedNotifications)
                }

                // Section 6: Community spotlight
                CommunitySpotlightSection(wishes: vm.communityWishes, selectedWishId: $selectedWishId)

                } // end else (not loading)
            }
            .padding(.horizontal, OffriiTheme.spacingBase)
            .padding(.top, OffriiTheme.spacingBase)
            .padding(.bottom, OffriiTheme.spacingXXL)
        }
        .background(OffriiTheme.background.ignoresSafeArea())
        .navigationTitle(greeting)
        .navigationBarTitleDisplayMode(.large)
        .toolbar {
            ToolbarItemGroup(placement: .topBarTrailing) {
                Button {
                    showNotificationCenter = true
                } label: {
                    Image(systemName: "bell")
                        .font(.system(size: 18))
                        .foregroundColor(OffriiTheme.primary)
                        .overlay(alignment: .topTrailing) {
                            if vm.unreadNotificationCount > 0 {
                                Text("\(min(vm.unreadNotificationCount, 99))")
                                    .font(.system(size: 9, weight: .bold))
                                    .foregroundColor(.white)
                                    .padding(3)
                                    .background(OffriiTheme.danger)
                                    .clipShape(Circle())
                                    .offset(x: 8, y: -6)
                            }
                        }
                }

                NavigationLink(destination: ProfileView()) {
                    ProfileAvatarButton(
                        initials: ProfileAvatarButton.initials(from: authManager.currentUser?.displayName),
                        avatarUrl: authManager.currentUser?.avatarUrl.flatMap { URL(string: $0) }
                    )
                    .id(authManager.currentUser?.avatarUrl)
                }
            }
        }
        .sheet(isPresented: $showNotificationCenter, onDismiss: {
            Task { await vm.load(authManager: authManager) }
        }) {
            NotificationCenterView()
        }
        .sheet(item: $selectedWishId, onDismiss: {
            Task { await vm.load(authManager: authManager) }
        }) { wishId in
            WishDetailSheet(wishId: wishId)
                .environment(authManager)
                .presentationDetents([.medium, .large])
        }
        .sheet(item: $selectedItemId, onDismiss: {
            Task { await vm.load(authManager: authManager) }
        }) { itemId in
            ItemDetailSheet(itemId: itemId)
                .environment(authManager)
                .presentationDetents([.medium, .large])
        }
        .sheet(isPresented: $showQuickAdd, onDismiss: {
            Task { await vm.load(authManager: authManager) }
        }) {
            QuickAddSheet { name, price, categoryId, priority, imageUrl, links, isPrivate in
                _ = try? await ItemService.shared.createItem(
                    name: name,
                    estimatedPrice: price,
                    priority: priority,
                    categoryId: categoryId,
                    imageUrl: imageUrl,
                    links: links,
                    isPrivate: isPrivate
                )
                return true
            }
        }
        .task { await vm.load(authManager: authManager) }
        .refreshable { await vm.load(authManager: authManager) }
    }

    // MARK: - Wishlist Grid Section

    @ViewBuilder
    // MARK: - Skeleton

    private var homeSkeletonView: some View {
        VStack(spacing: OffriiTheme.spacingLG) {
            // Subtitle placeholder
            HStack {
                RoundedRectangle(cornerRadius: 4)
                    .fill(OffriiTheme.border.opacity(0.3))
                    .frame(width: 180, height: 14)
                Spacer()
            }
            .shimmer()

            // Stats chips placeholder
            HStack(spacing: OffriiTheme.spacingSM) {
                ForEach(0..<4, id: \.self) { _ in
                    RoundedRectangle(cornerRadius: OffriiTheme.cornerRadiusMD)
                        .fill(OffriiTheme.border.opacity(0.2))
                        .frame(height: 56)
                }
            }
            .shimmer()

            // Grid placeholder
            LazyVGrid(columns: gridColumns, spacing: OffriiTheme.spacingSM) {
                ForEach(0..<4, id: \.self) { _ in
                    RoundedRectangle(cornerRadius: OffriiTheme.cornerRadiusLG)
                        .fill(OffriiTheme.border.opacity(0.15))
                        .frame(height: 160)
                }
            }
            .shimmer()

            // Activity placeholder
            VStack(spacing: OffriiTheme.spacingSM) {
                ForEach(0..<3, id: \.self) { _ in
                    SkeletonRow(height: 56)
                }
            }
        }
    }

    private var wishlistGridSection: some View {
        VStack(alignment: .leading, spacing: OffriiTheme.spacingSM) {
            // Title + "Voir tout"
            HStack {
                Text(NSLocalizedString("home.wishlistPreview.title", comment: ""))
                    .font(OffriiTypography.headline)
                    .foregroundColor(OffriiTheme.text)

                Spacer()

                Button {
                    router.selectedTab = .envies
                } label: {
                    HStack(spacing: OffriiTheme.spacingXXS) {
                        Text(NSLocalizedString("home.wishlistPreview.seeAll", comment: ""))
                            .font(OffriiTypography.subheadline)
                        Image(systemName: "arrow.right")
                            .font(.system(size: 12))
                    }
                    .foregroundColor(OffriiTheme.primary)
                }
            }

            if vm.isLoading && vm.recentItems.isEmpty {
                LazyVGrid(columns: gridColumns, spacing: OffriiTheme.spacingSM) {
                    ForEach(0..<4, id: \.self) { _ in SkeletonGridCard() }
                }
            } else if vm.recentItems.isEmpty {
                LazyVGrid(columns: gridColumns, spacing: OffriiTheme.spacingSM) {
                    ghostAddCard
                }
            } else {
                LazyVGrid(columns: gridColumns, spacing: OffriiTheme.spacingSM) {
                    ForEach(vm.recentItems.prefix(4)) { item in
                        WishlistGridCard(
                            item: item,
                            category: vm.category(for: item.categoryId)
                        ) {
                            selectedItemId = item.id
                        }
                    }

                    if vm.recentItems.count < 4 {
                        ghostAddCard
                    }
                }
            }
        }
    }

    // MARK: - Ghost Add Card

    private var ghostAddCard: some View {
        Button {
            showQuickAdd = true
        } label: {
            VStack(spacing: OffriiTheme.spacingSM) {
                Spacer()
                Image(systemName: "plus")
                    .font(.system(size: 28, weight: .light))
                    .foregroundColor(OffriiTheme.textMuted)
                Text(NSLocalizedString("home.wishlistGrid.addWish", comment: ""))
                    .font(OffriiTypography.caption)
                    .foregroundColor(OffriiTheme.textMuted)
                Spacer()
            }
            .frame(maxWidth: .infinity)
            .frame(height: 186)
            .background(OffriiTheme.card)
            .cornerRadius(OffriiTheme.cornerRadiusLG)
            .overlay(
                RoundedRectangle(cornerRadius: OffriiTheme.cornerRadiusLG)
                    .strokeBorder(style: StrokeStyle(lineWidth: 1.5, dash: [8, 4]))
                    .foregroundColor(OffriiTheme.border)
            )
        }
        .buttonStyle(.plain)
    }
}
