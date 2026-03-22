// swiftlint:disable file_length
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
    @State private var searchQuery = ""
    @State private var circleToDelete: OffriiCircle?
    @State private var circleToLeave: OffriiCircle?
    @State private var friendToRemove: FriendResponse?
    @State private var directCircleToRemove: OffriiCircle?
    @State private var showAcceptToast = false
    @State private var acceptedName = ""
    @State private var circleToTransfer: OffriiCircle?
    @State private var transferMembers: [CircleMember] = []

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
                                            circleToLeave = circle
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
        .searchable(
            text: $searchQuery,
            placement: .navigationBarDrawer(displayMode: .automatic),
            prompt: NSLocalizedString("circles.search.placeholder", comment: "")
        )
        .onChange(of: searchQuery) { _, newValue in
            viewModel.circleSearchQuery = newValue
            viewModel.friendSearchQuery = newValue
        }
        .toolbar {
            ToolbarItemGroup(placement: .topBarTrailing) {
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
        .sheet(isPresented: Binding(
            get: { circleToTransfer != nil },
            set: { if !$0 { circleToTransfer = nil } }
        )) {
            if let circle = circleToTransfer {
                TransferOwnershipPicker(
                    members: transferMembers,
                    onSelect: { member in
                        Task {
                            try? await CircleService.shared.transferOwnership(
                                circleId: circle.id, userId: member.userId
                            )
                            OffriiHaptics.success()
                            await viewModel.loadCircles()
                        }
                        circleToTransfer = nil
                    }
                )
                .presentationDetents([.medium])
            }
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
        .confirmationDialog(
            NSLocalizedString("circles.deleteCircle.title", comment: ""),
            isPresented: Binding(
                get: { circleToDelete != nil },
                set: { if !$0 { circleToDelete = nil } }
            ),
            titleVisibility: .visible
        ) {
            if let circle = circleToDelete, circle.memberCount > 1 {
                Button(NSLocalizedString("circles.deleteCircle.transferFirst", comment: "")) {
                    let id = circle.id
                    circleToDelete = nil
                    Task {
                        if let detail = try? await CircleService.shared.getCircle(id: id) {
                            transferMembers = detail.members.filter { $0.userId != authManager.currentUser?.id }
                            circleToTransfer = circle
                        }
                    }
                }
            }
            Button(NSLocalizedString("circles.deleteCircle.confirm", comment: ""), role: .destructive) {
                if let circle = circleToDelete {
                    Task { await viewModel.deleteCircle(circle) }
                }
                circleToDelete = nil
            }
            Button(NSLocalizedString("common.cancel", comment: ""), role: .cancel) {
                circleToDelete = nil
            }
        } message: {
            Text(NSLocalizedString("circles.deleteCircle.message", comment: ""))
        }
        .alert(
            NSLocalizedString("circles.leaveCircle.title", comment: ""),
            isPresented: Binding(
                get: { circleToLeave != nil },
                set: { if !$0 { circleToLeave = nil } }
            )
        ) {
            Button(NSLocalizedString("circles.context.leaveCircle", comment: ""), role: .destructive) {
                if let circle = circleToLeave,
                   let myId = authManager.currentUser?.id {
                    Task {
                        try? await CircleService.shared.removeMember(
                            circleId: circle.id, userId: myId
                        )
                        await viewModel.loadCircles()
                        OffriiHaptics.success()
                    }
                }
                circleToLeave = nil
            }
            Button(NSLocalizedString("common.cancel", comment: ""), role: .cancel) {
                circleToLeave = nil
            }
        } message: {
            Text(NSLocalizedString("circles.leaveCircle.message", comment: ""))
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

}
