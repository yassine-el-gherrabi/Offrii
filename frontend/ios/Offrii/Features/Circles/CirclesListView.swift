import SwiftUI

// MARK: - Circle Filter

enum CircleFilter: String, CaseIterable {
    case all
    case groups
    case friends

    var localizedTitle: String {
        switch self {
        case .all:
            return NSLocalizedString("circles.filter.all", comment: "")
        case .groups:
            return NSLocalizedString("circles.filter.groups", comment: "")
        case .friends:
            return NSLocalizedString("circles.filter.friends", comment: "")
        }
    }
}

// MARK: - CirclesListView

struct CirclesListView: View {
    @Environment(AuthManager.self) private var authManager
    @Environment(OnboardingTipManager.self) private var tipManager
    @State private var viewModel = CirclesViewModel()
    @State private var selectedFilter: CircleFilter = .all
    @State private var showCreateCircle = false
    @State private var showAddFriend = false
    @State private var showInviteContacts = false
    @State private var showPendingSheet = false
    @State private var circleToDelete: OffriiCircle?

    private var displayedCircles: [OffriiCircle] {
        let searched = viewModel.filteredCircles
        switch selectedFilter {
        case .all:
            return searched
        case .groups:
            return searched.filter { !$0.isDirect }
        case .friends:
            return searched.filter { $0.isDirect }
        }
    }

    var body: some View {
        ZStack(alignment: .bottomTrailing) {
            VStack(spacing: 0) {
                filterChips

                if viewModel.isLoadingCircles && viewModel.circles.isEmpty {
                    ScrollView {
                        LazyVStack(spacing: OffriiTheme.spacingSM) {
                            ForEach(0..<5, id: \.self) { _ in
                                SkeletonRow(height: 86)
                            }
                        }
                        .padding(.horizontal, OffriiTheme.spacingBase)
                        .padding(.top, OffriiTheme.spacingBase)
                    }
                } else if viewModel.circles.isEmpty {
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
                            searchBar
                                .padding(.horizontal, OffriiTheme.spacingXS)

                            if viewModel.pendingCount > 0 {
                                pendingSection
                            }

                            ForEach(displayedCircles) { circle in
                                NavigationLink(value: circle.id) {
                                    CircleCardRow(circle: circle)
                                }
                                .buttonStyle(.plain)
                                .contextMenu {
                                    Button(role: .destructive) {
                                        circleToDelete = circle
                                    } label: {
                                        Label(
                                            NSLocalizedString(
                                                circle.isDirect ? "circles.context.leave" : "circles.context.delete",
                                                comment: ""
                                            ),
                                            systemImage: circle.isDirect
                                                ? "rectangle.portrait.and.arrow.right" : "trash"
                                        )
                                    }
                                }
                            }
                        }
                        .padding(.horizontal, OffriiTheme.spacingBase)
                        .padding(.vertical, OffriiTheme.spacingSM)
                    }
                }
            }

            OffriiFloatingActionButton(icon: "plus") {
                showCreateCircle = true
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
                    if viewModel.pendingCount > 0 {
                        showPendingSheet = true
                    }
                } label: {
                    ZStack(alignment: .topTrailing) {
                        Image(systemName: "bell")
                            .font(.system(size: 18))
                            .foregroundColor(OffriiTheme.primary)

                        if viewModel.pendingCount > 0 {
                            Text("\(viewModel.pendingCount)")
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
        .sheet(isPresented: $showPendingSheet) {
            PendingInvitationsSheet(viewModel: viewModel)
                .presentationDetents([.medium, .large])
        }
        .task {
            await viewModel.loadAll()
            tipManager.showIfNeeded(.circlesCreate)
        }
        .onAppear {
            Task { await viewModel.loadCircles() }
        }
        .refreshable {
            await viewModel.loadAll()
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

    // MARK: - Pending Invitations Section

    @ViewBuilder
    private var pendingSection: some View {
        VStack(alignment: .leading, spacing: OffriiTheme.spacingSM) {
            HStack {
                Text(NSLocalizedString("circles.invitations", comment: ""))
                    .font(OffriiTypography.headline)
                    .foregroundColor(OffriiTheme.text)

                Text("\(viewModel.pendingCount)")
                    .font(OffriiTypography.caption)
                    .fontWeight(.bold)
                    .foregroundColor(.white)
                    .padding(.horizontal, 6)
                    .padding(.vertical, 2)
                    .background(OffriiTheme.danger)
                    .clipShape(Capsule())

                Spacer()

                if viewModel.pendingRequests.count > 2 {
                    Button {
                        showPendingSheet = true
                    } label: {
                        Text(NSLocalizedString("circles.invitations.viewAll", comment: ""))
                            .font(OffriiTypography.footnote)
                            .foregroundColor(OffriiTheme.primary)
                    }
                }
            }

            ForEach(Array(viewModel.pendingRequests.prefix(2))) { request in
                pendingRequestRow(request)
            }
        }
        .padding(OffriiTheme.spacingBase)
        .background(OffriiTheme.card)
        .cornerRadius(OffriiTheme.cornerRadiusLG)
        .shadow(color: OffriiTheme.cardShadowColor, radius: 6, x: 0, y: 2)
    }

    @ViewBuilder
    private func pendingRequestRow(_ request: FriendRequestResponse) -> some View {
        HStack(spacing: OffriiTheme.spacingSM) {
            AvatarView(
                request.fromDisplayName ?? request.fromUsername,
                size: .small
            )

            VStack(alignment: .leading, spacing: 2) {
                Text(request.fromDisplayName ?? request.fromUsername)
                    .font(OffriiTypography.body)
                    .foregroundColor(OffriiTheme.text)
                Text("@\(request.fromUsername)")
                    .font(OffriiTypography.caption)
                    .foregroundColor(OffriiTheme.textMuted)
            }

            Spacer()

            HStack(spacing: OffriiTheme.spacingSM) {
                Button {
                    Task { await viewModel.acceptRequest(request) }
                } label: {
                    Text(NSLocalizedString("friends.accept", comment: ""))
                        .font(OffriiTypography.footnote)
                        .fontWeight(.semibold)
                        .foregroundColor(.white)
                        .padding(.horizontal, OffriiTheme.spacingSM)
                        .padding(.vertical, OffriiTheme.spacingXS)
                        .background(OffriiTheme.primary)
                        .cornerRadius(OffriiTheme.cornerRadiusXL)
                }

                Button {
                    Task { await viewModel.declineRequest(request) }
                } label: {
                    Text(NSLocalizedString("friends.decline", comment: ""))
                        .font(OffriiTypography.footnote)
                        .foregroundColor(OffriiTheme.textSecondary)
                        .padding(.horizontal, OffriiTheme.spacingSM)
                        .padding(.vertical, OffriiTheme.spacingXS)
                        .background(OffriiTheme.textMuted.opacity(0.15))
                        .cornerRadius(OffriiTheme.cornerRadiusXL)
                }
            }
        }
    }
}

// MARK: - Pending Invitations Sheet

struct PendingInvitationsSheet: View {
    var viewModel: CirclesViewModel
    @Environment(\.dismiss) private var dismiss

    var body: some View {
        NavigationStack {
            ScrollView {
                LazyVStack(spacing: 0) {
                    ForEach(viewModel.pendingRequests) { request in
                        HStack(spacing: OffriiTheme.spacingSM) {
                            AvatarView(
                                request.fromDisplayName ?? request.fromUsername,
                                size: .small
                            )

                            VStack(alignment: .leading, spacing: 2) {
                                Text(request.fromDisplayName ?? request.fromUsername)
                                    .font(OffriiTypography.body)
                                    .foregroundColor(OffriiTheme.text)
                                Text("@\(request.fromUsername)")
                                    .font(OffriiTypography.caption)
                                    .foregroundColor(OffriiTheme.textMuted)
                            }

                            Spacer()

                            HStack(spacing: OffriiTheme.spacingSM) {
                                Button {
                                    Task { await viewModel.acceptRequest(request) }
                                } label: {
                                    Text(NSLocalizedString("friends.accept", comment: ""))
                                        .font(OffriiTypography.footnote)
                                        .fontWeight(.semibold)
                                        .foregroundColor(.white)
                                        .padding(.horizontal, OffriiTheme.spacingSM)
                                        .padding(.vertical, OffriiTheme.spacingXS)
                                        .background(OffriiTheme.primary)
                                        .cornerRadius(OffriiTheme.cornerRadiusXL)
                                }

                                Button {
                                    Task { await viewModel.declineRequest(request) }
                                } label: {
                                    Text(NSLocalizedString("friends.decline", comment: ""))
                                        .font(OffriiTypography.footnote)
                                        .foregroundColor(OffriiTheme.textSecondary)
                                        .padding(.horizontal, OffriiTheme.spacingSM)
                                        .padding(.vertical, OffriiTheme.spacingXS)
                                        .background(OffriiTheme.textMuted.opacity(0.15))
                                        .cornerRadius(OffriiTheme.cornerRadiusXL)
                                }
                            }
                        }
                        .padding(.horizontal, OffriiTheme.spacingLG)
                        .padding(.vertical, OffriiTheme.spacingSM)

                        Divider()
                            .padding(.leading, 56)
                            .padding(.horizontal, OffriiTheme.spacingLG)
                    }

                    if viewModel.sentRequests.isEmpty == false {
                        Text(NSLocalizedString("friends.sent", comment: ""))
                            .font(OffriiTypography.headline)
                            .foregroundColor(OffriiTheme.text)
                            .frame(maxWidth: .infinity, alignment: .leading)
                            .padding(.horizontal, OffriiTheme.spacingLG)
                            .padding(.top, OffriiTheme.spacingLG)
                            .padding(.bottom, OffriiTheme.spacingSM)

                        ForEach(viewModel.sentRequests) { request in
                            HStack(spacing: OffriiTheme.spacingSM) {
                                AvatarView(
                                    request.toDisplayName ?? request.toUsername,
                                    size: .small
                                )

                                VStack(alignment: .leading, spacing: 2) {
                                    Text(request.toDisplayName ?? request.toUsername)
                                        .font(OffriiTypography.body)
                                        .foregroundColor(OffriiTheme.text)
                                    Text("@\(request.toUsername)")
                                        .font(OffriiTypography.caption)
                                        .foregroundColor(OffriiTheme.textMuted)
                                }

                                Spacer()

                                Button {
                                    Task { await viewModel.cancelRequest(request) }
                                } label: {
                                    Text(NSLocalizedString("friends.cancel", comment: ""))
                                        .font(OffriiTypography.footnote)
                                        .foregroundColor(OffriiTheme.danger)
                                        .padding(.horizontal, OffriiTheme.spacingSM)
                                        .padding(.vertical, OffriiTheme.spacingXS)
                                        .background(OffriiTheme.danger.opacity(0.1))
                                        .cornerRadius(OffriiTheme.cornerRadiusXL)
                                }
                            }
                            .padding(.horizontal, OffriiTheme.spacingLG)
                            .padding(.vertical, OffriiTheme.spacingXS)
                        }
                    }
                }
                .padding(.top, OffriiTheme.spacingSM)
            }
            .background(OffriiTheme.background.ignoresSafeArea())
            .navigationTitle(NSLocalizedString("circles.invitations", comment: ""))
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button(NSLocalizedString("common.ok", comment: "")) {
                        dismiss()
                    }
                }
            }
        }
    }
}
