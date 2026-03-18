import SwiftUI

// MARK: - Circle Filter

enum CircleFilter: String, CaseIterable {
    case all
    case groups
    case friends
    case reservations

    var localizedTitle: String {
        switch self {
        case .all:
            return NSLocalizedString("circles.filter.all", comment: "")
        case .groups:
            return NSLocalizedString("circles.filter.groups", comment: "")
        case .friends:
            return NSLocalizedString("circles.filter.friends", comment: "")
        case .reservations:
            return NSLocalizedString("circles.filter.reservations", comment: "")
        }
    }
}

// MARK: - CirclesListView

struct CirclesListView: View {
    @Environment(AuthManager.self) private var authManager
    @Environment(AppRouter.self) private var router
    @Environment(OnboardingTipManager.self) private var tipManager
    @State private var viewModel = CirclesViewModel()
    @State private var selectedFilter: CircleFilter = .all
    @State private var showCreateCircle = false
    @State private var showAddFriend = false
    @State private var showQuickCreate = false
    @State private var showInviteContacts = false
    @State private var showNotificationCenter = false
    @State private var unreadCount = 0
    @State private var circleToDelete: OffriiCircle?
    @State private var friendToRemove: FriendResponse?
    @State private var directCircleToRemove: OffriiCircle?
    @State private var showAcceptToast = false
    @State private var acceptedName = ""

    private var displayedCircles: [OffriiCircle] {
        let searched = viewModel.filteredCircles
        switch selectedFilter {
        case .all:
            return searched
        case .groups:
            return searched.filter { !$0.isDirect }
        case .friends:
            return searched.filter { $0.isDirect }
        case .reservations:
            return [] // Not used — reservations have their own content
        }
    }

    var body: some View {
        ZStack(alignment: .bottomTrailing) {
            VStack(spacing: 0) {
                filterChips

                if selectedFilter == .reservations {
                    ReservationsListView()
                } else if selectedFilter == .friends {
                    friendsContent
                } else if viewModel.isLoadingCircles && viewModel.circles.isEmpty {
                    ScrollView {
                        LazyVStack(spacing: OffriiTheme.spacingSM) {
                            ForEach(0..<5, id: \.self) { _ in
                                SkeletonRow(height: 86)
                            }
                        }
                        .padding(.horizontal, OffriiTheme.spacingBase)
                        .padding(.top, OffriiTheme.spacingBase)
                    }
                } else if viewModel.circles.isEmpty && viewModel.pendingCount == 0 {
                    Spacer()
                    OffriiEmptyState(
                        icon: "person.2.fill",
                        title: NSLocalizedString("circles.empty", comment: ""),
                        subtitle: NSLocalizedString("circles.emptySubtitle", comment: ""),
                        ctaTitle: NSLocalizedString("circles.create", comment: ""),
                        ctaAction: { showCreateCircle = true }
                    )
                    Spacer()
                } else {
                    ScrollView {
                        LazyVStack(spacing: OffriiTheme.spacingSM) {
                            if !viewModel.circles.isEmpty {
                                searchBar
                                    .padding(.horizontal, OffriiTheme.spacingXS)
                            }

                            ForEach(displayedCircles) { circle in
                                NavigationLink(value: circle.id) {
                                    CircleCardRow(circle: circle)
                                }
                                .buttonStyle(.plain)
                                .contextMenu {
                                    if circle.isDirect {
                                        Button(role: .destructive) {
                                            directCircleToRemove = circle
                                        } label: {
                                            Label(
                                                NSLocalizedString("friends.removeConfirm.title", comment: ""),
                                                systemImage: "person.badge.minus"
                                            )
                                        }
                                    } else if circle.ownerId == authManager.currentUser?.id {
                                        Button(role: .destructive) {
                                            circleToDelete = circle
                                        } label: {
                                            Label(
                                                NSLocalizedString("circles.context.deleteCircle", comment: ""),
                                                systemImage: "trash"
                                            )
                                        }
                                    } else {
                                        Button(role: .destructive) {
                                            circleToDelete = circle
                                        } label: {
                                            Label(
                                                NSLocalizedString("circles.context.leaveCircle", comment: ""),
                                                systemImage: "rectangle.portrait.and.arrow.right"
                                            )
                                        }
                                    }
                                }
                            }
                        }
                        .padding(.horizontal, OffriiTheme.spacingBase)
                        .padding(.vertical, OffriiTheme.spacingSM)
                    }
                }
            }

            OffriiFloatingActionButton(icon: selectedFilter == .friends ? "person.badge.plus" : "plus") {
                switch selectedFilter {
                case .friends:
                    showAddFriend = true
                case .groups:
                    showCreateCircle = true
                default:
                    showQuickCreate = true
                }
            }
            .padding(.trailing, OffriiTheme.spacingLG)
            .padding(.bottom, OffriiTheme.spacingLG)
        }
        .background(OffriiTheme.background.ignoresSafeArea())
        .navigationTitle(NSLocalizedString("circles.title", comment: ""))
        .navigationBarTitleDisplayMode(.large)
        .toolbar {
            ToolbarItemGroup(placement: .topBarTrailing) {
                Button {
                    showNotificationCenter = true
                } label: {
                    ZStack(alignment: .topTrailing) {
                        Image(systemName: "bell")
                            .font(.system(size: 18))
                            .foregroundColor(OffriiTheme.primary)

                        if unreadCount > 0 {
                            Text("\(unreadCount)")
                                .font(.system(size: 9, weight: .bold))
                                .foregroundColor(.white)
                                .padding(3)
                                .background(OffriiTheme.danger)
                                .clipShape(Circle())
                                .offset(x: 6, y: -6)
                        }
                    }
                }

                NavigationLink(destination: ProfileView()) {
                    ProfileAvatarButton(
                        initials: ProfileAvatarButton.initials(
                            from: authManager.currentUser?.displayName
                        ),
                        avatarUrl: authManager.currentUser?.avatarUrl.flatMap { URL(string: $0) }
                    )
                }
            }
        }
        .navigationDestination(for: UUID.self) { circleId in
            CircleDetailView(circleId: circleId)
                .environment(authManager)
        }
        .sheet(isPresented: $showCreateCircle) {
            CreateCircleSheet { _ in
                Task { await viewModel.loadCircles() }
            }
        }
        .sheet(isPresented: $showAddFriend) {
            AddFriendSheet {
                Task {
                    await viewModel.loadFriends()
                    await viewModel.loadSentRequests()
                }
            }
        }
        .sheet(isPresented: $showInviteContacts) {
            InviteContactsSheet()
                .presentationDetents([.large])
        }
        .sheet(isPresented: $showNotificationCenter, onDismiss: {
            Task { await loadUnreadCount() }
        }) {
            NotificationCenterView()
                .presentationDetents([.medium, .large])
        }
        .sheet(isPresented: $showQuickCreate) {
            CirclesQuickActionSheet(
                onCreateCircle: {
                    showQuickCreate = false
                    DispatchQueue.main.asyncAfter(deadline: .now() + 0.3) {
                        showCreateCircle = true
                    }
                },
                onAddFriend: {
                    showQuickCreate = false
                    DispatchQueue.main.asyncAfter(deadline: .now() + 0.3) {
                        showAddFriend = true
                    }
                }
            )
            .presentationDetents([.height(220)])
        }
        .task {
            await viewModel.loadAll()
            await loadUnreadCount()
            tipManager.showIfNeeded(.circlesCreate)
        }
        .onAppear {
            // Handle pending navigation from push notification or notification center
            if router.showFriends {
                selectedFilter = .friends
                router.showFriends = false
            }
            Task {
                await viewModel.loadCircles()
                await loadUnreadCount()
            }
        }
        .refreshable {
            await viewModel.loadAll()
        }
        .onChange(of: router.showFriends) { _, show in
            guard show else { return }
            selectedFilter = .friends
            router.showFriends = false
        }
        .alert(
            NSLocalizedString(
                circleToDelete?.isDirect == true
                    ? "circles.leaveCircle.title" : "circles.deleteCircle.title",
                comment: ""
            ),
            isPresented: Binding(
                get: { circleToDelete != nil },
                set: { if !$0 { circleToDelete = nil } }
            )
        ) {
            Button(NSLocalizedString("common.delete", comment: ""), role: .destructive) {
                if let circle = circleToDelete {
                    Task { await viewModel.deleteCircle(circle) }
                }
                circleToDelete = nil
            }
            Button(NSLocalizedString("common.cancel", comment: ""), role: .cancel) {
                circleToDelete = nil
            }
        } message: {
            Text(NSLocalizedString(
                circleToDelete?.isDirect == true
                    ? "circles.leaveCircle.message" : "circles.deleteCircle.message",
                comment: ""
            ))
        }
        .alert(
            NSLocalizedString("friends.removeConfirm.title", comment: ""),
            isPresented: Binding(
                get: { friendToRemove != nil },
                set: { if !$0 { friendToRemove = nil } }
            )
        ) {
            Button(NSLocalizedString("friends.remove", comment: ""), role: .destructive) {
                if let friend = friendToRemove {
                    Task {
                        await viewModel.removeFriend(friend)
                        OffriiHaptics.success()
                    }
                }
                friendToRemove = nil
            }
            Button(NSLocalizedString("common.cancel", comment: ""), role: .cancel) {
                friendToRemove = nil
            }
        } message: {
            if let friend = friendToRemove {
                Text(String(
                    format: NSLocalizedString("friends.removeConfirm.message", comment: ""),
                    friend.displayName ?? friend.username
                ))
            }
        }
        .alert(
            NSLocalizedString("friends.removeConfirm.title", comment: ""),
            isPresented: Binding(
                get: { directCircleToRemove != nil },
                set: { if !$0 { directCircleToRemove = nil } }
            )
        ) {
            Button(NSLocalizedString("friends.remove", comment: ""), role: .destructive) {
                if let circle = directCircleToRemove {
                    Task {
                        // Match friend by UUID from circle.memberIds
                        let friend = viewModel.friends.first { friend in
                            circle.memberIds.contains(friend.userId)
                        }
                        if let friend {
                            await viewModel.removeFriend(friend)
                        }
                        await viewModel.loadAll()
                        OffriiHaptics.success()
                    }
                }
                directCircleToRemove = nil
            }
            Button(NSLocalizedString("common.cancel", comment: ""), role: .cancel) {
                directCircleToRemove = nil
            }
        } message: {
            if let circle = directCircleToRemove {
                Text(String(
                    format: NSLocalizedString("friends.removeConfirm.message", comment: ""),
                    circle.name ?? circle.memberNames.first ?? ""
                ))
            }
        }
        .offriiToast(
            isPresented: $showAcceptToast,
            message: String(format: NSLocalizedString("friends.accepted.toast", comment: ""), acceptedName),
            style: .success
        )
    }

    // MARK: - Filter Chips

    @ViewBuilder
    private var filterChips: some View {
        ScrollView(.horizontal, showsIndicators: false) {
            HStack(spacing: OffriiTheme.spacingSM) {
                ForEach(CircleFilter.allCases, id: \.self) { filter in
                    OffriiChip(
                        title: filter.localizedTitle,
                        isSelected: selectedFilter == filter,
                        badgeCount: filter == .friends ? viewModel.pendingCount : 0,
                        action: { selectedFilter = filter }
                    )
                }
            }
            .padding(.horizontal, OffriiTheme.spacingBase)
        }
        .padding(.vertical, OffriiTheme.spacingSM)
    }

    // MARK: - Search Bar

    @ViewBuilder
    private var searchBar: some View {
        HStack(spacing: OffriiTheme.spacingSM) {
            Image(systemName: "magnifyingglass")
                .foregroundColor(OffriiTheme.textMuted)

            TextField(
                NSLocalizedString("circles.search.placeholder", comment: ""),
                text: Bindable(viewModel).circleSearchQuery
            )
            .font(OffriiTypography.body)
            .autocapitalization(.none)
            .autocorrectionDisabled()

            if !viewModel.circleSearchQuery.isEmpty {
                Button {
                    viewModel.circleSearchQuery = ""
                } label: {
                    Image(systemName: "xmark.circle.fill")
                        .foregroundColor(OffriiTheme.textMuted)
                }
            }
        }
        .padding(OffriiTheme.spacingSM)
        .background(OffriiTheme.card)
        .cornerRadius(OffriiTheme.cornerRadiusSM)
        .overlay(
            RoundedRectangle(cornerRadius: OffriiTheme.cornerRadiusSM)
                .stroke(OffriiTheme.border, lineWidth: 1)
        )
    }

    // MARK: - Friends Content (Amis filter) — extracted to FriendsListContent.swift

    @ViewBuilder
    private var friendsContent: some View {
        FriendsListContent(
            viewModel: viewModel,
            showAddFriend: $showAddFriend,
            friendToRemove: $friendToRemove,
            showAcceptToast: $showAcceptToast,
            acceptedName: $acceptedName
        )
    }

    private func loadUnreadCount() async {
        unreadCount = (try? await NotificationCenterService.shared.unreadCount()) ?? 0
    }
}

// MARK: - Quick Action Sheet (Tous filter FAB)

private struct CirclesQuickActionSheet: View {
    @Environment(\.dismiss) private var dismiss
    let onCreateCircle: () -> Void
    let onAddFriend: () -> Void

    var body: some View {
        VStack(spacing: OffriiTheme.spacingBase) {
            actionRow(
                icon: "person.2.fill",
                iconColor: OffriiTheme.accent,
                title: NSLocalizedString("create.createCircle", comment: ""),
                subtitle: NSLocalizedString("create.createCircleSubtitle", comment: "")
            ) {
                onCreateCircle()
            }

            actionRow(
                icon: "person.badge.plus",
                iconColor: OffriiTheme.accent,
                title: NSLocalizedString("create.addFriend", comment: ""),
                subtitle: NSLocalizedString("create.addFriendSubtitle", comment: "")
            ) {
                onAddFriend()
            }
        }
        .padding(.horizontal, OffriiTheme.spacingLG)
        .padding(.top, OffriiTheme.spacingLG)
    }

    private func actionRow(
        icon: String,
        iconColor: Color,
        title: String,
        subtitle: String,
        action: @escaping () -> Void
    ) -> some View {
        Button(action: action) {
            HStack(spacing: OffriiTheme.spacingBase) {
                RoundedRectangle(cornerRadius: OffriiTheme.cornerRadiusMD)
                    .fill(iconColor.opacity(0.12))
                    .frame(width: 48, height: 48)
                    .overlay(
                        Image(systemName: icon)
                            .font(.system(size: 20))
                            .foregroundColor(iconColor)
                    )

                VStack(alignment: .leading, spacing: OffriiTheme.spacingXXS) {
                    Text(title)
                        .font(OffriiTypography.headline)
                        .foregroundColor(OffriiTheme.text)
                    Text(subtitle)
                        .font(OffriiTypography.subheadline)
                        .foregroundColor(OffriiTheme.textSecondary)
                }

                Spacer()

                Image(systemName: "chevron.right")
                    .font(.system(size: 12, weight: .semibold))
                    .foregroundColor(OffriiTheme.textMuted)
            }
            .padding(OffriiTheme.spacingBase)
            .background(OffriiTheme.card)
            .cornerRadius(OffriiTheme.cornerRadiusLG)
            .shadow(
                color: OffriiTheme.cardShadowColor,
                radius: OffriiTheme.cardShadowRadius,
                x: 0,
                y: OffriiTheme.cardShadowY
            )
        }
        .buttonStyle(.plain)
    }
}
