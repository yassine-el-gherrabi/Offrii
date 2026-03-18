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
    @State private var showInviteContacts = false
    @State private var showNotificationCenter = false
    @State private var unreadCount = 0
    @State private var circleToDelete: OffriiCircle?
    @State private var friendToRemove: FriendResponse?
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
                        subtitle: NSLocalizedString("circles.emptySubtitle", comment: "")
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
                                            // Find the friend for this direct circle
                                            friendToRemove = viewModel.friends.first { friend in
                                                circle.memberNames.contains(friend.username)
                                            }
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
                if selectedFilter == .friends {
                    showAddFriend = true
                } else {
                    showCreateCircle = true
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
