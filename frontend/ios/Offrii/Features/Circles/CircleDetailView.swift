import NukeUI
import SwiftUI

// swiftlint:disable file_length type_body_length

struct CircleDetailView: View {
    let circleId: UUID
    @Environment(AuthManager.self) private var authManager
    @Environment(OnboardingTipManager.self) private var tipManager
    @Environment(\.dismiss) private var dismiss
    @State private var viewModel = CircleDetailViewModel()
    @State private var showInvite = false
    @State private var showEdit = false
    @State private var showLeaveAlert = false
    @State private var selectedItemId: UUID?
    @State private var memberToRemove: CircleMember?
    @State private var memberToTransfer: CircleMember?

    private var currentUserId: UUID? { authManager.currentUser?.id }
    private var isOwner: Bool { viewModel.detail?.ownerId == currentUserId }
    private var isDirect: Bool { viewModel.detail?.isDirect == true }

    private let gridColumns = [
        GridItem(.flexible(), spacing: OffriiTheme.spacingSM),
        GridItem(.flexible(), spacing: OffriiTheme.spacingSM)
    ]

    var body: some View {
        ZStack {
            OffriiTheme.background.ignoresSafeArea()

            if viewModel.isLoading && viewModel.detail == nil {
                SkeletonList(count: 4)
                    .padding(.top, OffriiTheme.spacingBase)
            } else if let detail = viewModel.detail {
                if isDirect {
                    directCircleContent(detail)
                } else {
                    groupCircleContent(detail)
                }
            } else if let error = viewModel.error {
                errorView(error)
            }
        }
        .navigationTitle(navigationTitle)
        .navigationBarTitleDisplayMode(.inline)
        .toolbar { toolbarContent }
        .sheet(isPresented: $showInvite) {
            if let detail = viewModel.detail {
                InviteFriendsSheet(
                    circleId: circleId,
                    existingMemberIds: Set(detail.members.map(\.userId)),
                    onInvited: { Task { await reload() } }
                )
            }
        }
        .sheet(isPresented: $showEdit) {
            if let name = viewModel.detail?.name {
                EditCircleSheet(
                    circleId: circleId,
                    currentName: name,
                    currentImageUrl: viewModel.detail?.imageUrl
                ) {
                    Task { await reload() }
                }
                .presentationDetents([.medium])
            }
        }
        .sheet(item: $selectedItemId, onDismiss: {
            Task { await reload() }
        }) { itemId in
            ItemDetailSheet(itemId: itemId, circleId: circleId)
                .environment(authManager)
                .presentationDetents([.medium, .large])
        }
        .alert(
            NSLocalizedString("circles.members.leaveConfirm.title", comment: ""),
            isPresented: $showLeaveAlert
        ) {
            Button(
                NSLocalizedString("circles.members.leave", comment: ""),
                role: .destructive
            ) {
                Task {
                    if await viewModel.leaveCircle(circleId: circleId) {
                        dismiss()
                    }
                }
            }
            Button(NSLocalizedString("common.cancel", comment: ""), role: .cancel) {}
        } message: {
            Text(NSLocalizedString(
                "circles.members.leaveConfirm.message",
                comment: ""
            ))
        }
        .alert(
            NSLocalizedString("circles.removeMember.title", comment: ""),
            isPresented: Binding(
                get: { memberToRemove != nil },
                set: { if !$0 { memberToRemove = nil } }
            )
        ) {
            Button(NSLocalizedString("friends.remove", comment: ""), role: .destructive) {
                if let member = memberToRemove {
                    Task {
                        await viewModel.removeMember(circleId: circleId, userId: member.userId)
                    }
                }
                memberToRemove = nil
            }
            Button(NSLocalizedString("common.cancel", comment: ""), role: .cancel) {
                memberToRemove = nil
            }
        } message: {
            if let member = memberToRemove {
                Text(String(format: NSLocalizedString("circles.removeMember.message", comment: ""), member.displayName ?? ""))
            }
        }
        .alert(
            NSLocalizedString("circles.transferOwnership.title", comment: ""),
            isPresented: Binding(
                get: { memberToTransfer != nil },
                set: { if !$0 { memberToTransfer = nil } }
            )
        ) {
            Button(NSLocalizedString("circles.transferOwnership.action", comment: ""), role: .destructive) {
                if let member = memberToTransfer {
                    Task {
                        await viewModel.transferOwnership(circleId: circleId, userId: member.userId)
                    }
                }
                memberToTransfer = nil
            }
            Button(NSLocalizedString("common.cancel", comment: ""), role: .cancel) {
                memberToTransfer = nil
            }
        } message: {
            if let member = memberToTransfer {
                Text(String(format: NSLocalizedString("circles.transferOwnership.message", comment: ""), member.displayName ?? ""))
            }
        }
        .refreshable {
            await reload()
        }
        .task {
            viewModel.currentUserId = currentUserId
            await reload()
            if !isDirect {
                tipManager.showIfNeeded(.circlesShare)
            }
        }
    }

    // MARK: - Navigation Title

    private var navigationTitle: String {
        if isDirect, let friend = viewModel.friendMember {
            return friend.displayName ?? friend.username
        }
        return viewModel.detail?.name
            ?? NSLocalizedString("circles.unnamed", comment: "")
    }

    // MARK: - Toolbar

    @ToolbarContentBuilder
    private var toolbarContent: some ToolbarContent {
        ToolbarItem(placement: .primaryAction) {
            if isDirect {
                EmptyView()
            } else {
                HStack(spacing: OffriiTheme.spacingSM) {
                    Button {
                        showEdit = true
                    } label: {
                        Image(systemName: "pencil")
                            .font(.system(size: 16))
                    }

                    Button {
                        showInvite = true
                    } label: {
                        Image(systemName: "person.badge.plus")
                            .font(.system(size: 16))
                    }
                    .overlay(alignment: .bottom) {
                        if tipManager.activeTip == .circlesShare {
                            OffriiTooltip(
                                message: OnboardingTipManager.message(
                                    for: .circlesShare
                                ),
                                arrow: .top
                            ) {
                                tipManager.dismiss(.circlesShare)
                            }
                            .frame(width: 220)
                            .offset(y: 50)
                        }
                    }
                }
            }
        }
    }

    // MARK: - Error View

    @ViewBuilder
    private func errorView(_ error: String) -> some View {
        VStack(spacing: OffriiTheme.spacingBase) {
            Text(error)
                .font(OffriiTypography.body)
                .foregroundColor(OffriiTheme.textSecondary)
            Button(NSLocalizedString("common.retry", comment: "")) {
                Task { await reload() }
            }
            .foregroundColor(OffriiTheme.primary)
        }
    }

    // MARK: - Direct (1:1) Content

    @ViewBuilder
    private func directCircleContent(_ detail: CircleDetailResponse) -> some View {
        VStack(spacing: 0) {
            // Friend header
            directHeader(detail)

            // 3-tab picker: Their wishes / My wishes / Activity
            Picker("", selection: $viewModel.selectedTab) {
                Text(NSLocalizedString("circles.detail.theirWishes", comment: ""))
                    .tag(CircleDetailViewModel.DetailTab.items)
                Text(NSLocalizedString("circles.detail.myWishes", comment: ""))
                    .tag(CircleDetailViewModel.DetailTab.myItems)
                Text(NSLocalizedString("circles.detail.activity", comment: ""))
                    .tag(CircleDetailViewModel.DetailTab.activity)
            }
            .pickerStyle(.segmented)
            .padding(.horizontal, OffriiTheme.spacingLG)
            .padding(.vertical, OffriiTheme.spacingSM)

            switch viewModel.selectedTab {
            case .items:
                itemGridContent(items: viewModel.theirItems, showClaimButtons: true)
            case .myItems:
                itemGridContent(items: viewModel.myItems, showClaimButtons: false)
            case .activity:
                CircleActivityFeed(
                    events: viewModel.feed,
                    currentUserId: currentUserId
                )
            case .members:
                // Not shown for direct circles, but handle gracefully
                EmptyView()
            }
        }
    }

    // MARK: - Direct Header

    @ViewBuilder
    private func directHeader(_ detail: CircleDetailResponse) -> some View {
        let friend = viewModel.friendMember

        VStack(spacing: OffriiTheme.spacingSM) {
            AvatarView(
                friend?.displayName ?? friend?.username ?? detail.name,
                size: .large
            )

            Text(friend?.displayName ?? friend?.username ?? (detail.name ?? ""))
                .font(OffriiTypography.titleSmall)
                .foregroundColor(OffriiTheme.text)

            if let username = friend?.username {
                Text("@\(username)")
                    .font(OffriiTypography.subheadline)
                    .foregroundColor(OffriiTheme.textSecondary)
            }

            Text(String(
                format: NSLocalizedString("circles.detail.friendSince", comment: ""),
                formattedDate(detail.createdAt)
            ))
            .font(OffriiTypography.caption)
            .foregroundColor(OffriiTheme.textMuted)
        }
        .frame(maxWidth: .infinity)
        .padding(.vertical, OffriiTheme.spacingLG)
    }

    // MARK: - Group Header

    @ViewBuilder
    private func groupHeader(_ detail: CircleDetailResponse) -> some View {
        VStack(spacing: OffriiTheme.spacingSM) {
            if let imageUrl = detail.imageUrl, let url = URL(string: imageUrl) {
                LazyImage(url: url) { state in
                    if let image = state.image {
                        image
                            .resizable()
                            .aspectRatio(contentMode: .fill)
                            .frame(width: 72, height: 72)
                            .clipShape(Circle())
                    } else {
                        groupAvatarFallback(detail)
                    }
                }
            } else {
                groupAvatarFallback(detail)
            }

            Text(String(
                format: NSLocalizedString("circles.memberCount", comment: ""),
                detail.members.count
            ))
            .font(OffriiTypography.caption)
            .foregroundColor(OffriiTheme.textMuted)
        }
        .frame(maxWidth: .infinity)
        .padding(.vertical, OffriiTheme.spacingBase)
    }

    private func groupAvatarFallback(_ detail: CircleDetailResponse) -> some View {
        AvatarView(detail.name, size: .large)
    }

    // MARK: - Group Content

    @ViewBuilder
    private func groupCircleContent(_ detail: CircleDetailResponse) -> some View {
        VStack(spacing: 0) {
            // Circle avatar header
            groupHeader(detail)

            // Member carousel filter
            MemberCarousel(
                members: detail.members,
                selectedMemberId: $viewModel.selectedMemberFilter,
                currentUserId: currentUserId
            )

            // 4-tab segmented control
            Picker("", selection: $viewModel.selectedTab) {
                Text(NSLocalizedString("circles.detail.items", comment: ""))
                    .tag(CircleDetailViewModel.DetailTab.items)
                Text(NSLocalizedString("circles.detail.myWishes", comment: ""))
                    .tag(CircleDetailViewModel.DetailTab.myItems)
                Text(NSLocalizedString("circles.detail.members", comment: ""))
                    .tag(CircleDetailViewModel.DetailTab.members)
                Text(NSLocalizedString("circles.detail.activity", comment: ""))
                    .tag(CircleDetailViewModel.DetailTab.activity)
            }
            .pickerStyle(.segmented)
            .padding(.horizontal, OffriiTheme.spacingLG)
            .padding(.vertical, OffriiTheme.spacingSM)

            switch viewModel.selectedTab {
            case .items:
                itemGridContent(items: viewModel.filteredItems, showClaimButtons: true)
            case .myItems:
                itemGridContent(items: viewModel.myItems, showClaimButtons: false)
            case .members:
                membersTabContent(detail)
            case .activity:
                CircleActivityFeed(
                    events: viewModel.feed,
                    currentUserId: currentUserId
                )
            }
        }
    }

    // MARK: - Shared Item Grid

    @ViewBuilder
    // swiftlint:disable:next function_body_length
    private func itemGridContent(
        items: [CircleItemResponse],
        showClaimButtons: Bool
    ) -> some View {
        if items.isEmpty {
            Spacer()
            OffriiEmptyState(
                icon: "tray",
                title: NSLocalizedString("circles.detail.noItems", comment: ""),
                subtitle: NSLocalizedString(
                    "circles.detail.noItemsSubtitle",
                    comment: ""
                )
            )
            Spacer()
        } else {
            ScrollView {
                LazyVGrid(columns: gridColumns, spacing: OffriiTheme.spacingSM) {
                    ForEach(items) { item in
                        let isMyItem = item.sharedBy == currentUserId

                        Button {
                            OffriiHaptics.tap()
                            selectedItemId = item.id
                        } label: {
                            circleItemCard(item, showClaimButtons: showClaimButtons)
                        }
                        .buttonStyle(.plain)
                        .contextMenu {
                            if !isMyItem {
                                if item.isClaimed {
                                    if item.claimedBy?.userId == currentUserId {
                                        Button {
                                            Task {
                                                await viewModel.unclaimItem(itemId: item.id)
                                                await viewModel.loadItems(circleId: circleId)
                                                await viewModel.loadFeed(circleId: circleId)
                                            }
                                        } label: {
                                            Label(
                                                NSLocalizedString("circles.detail.claimed", comment: ""),
                                                systemImage: "xmark.circle"
                                            )
                                        }
                                    }
                                } else {
                                    Button {
                                        Task {
                                            await viewModel.claimItem(itemId: item.id)
                                            await viewModel.loadItems(circleId: circleId)
                                                await viewModel.loadFeed(circleId: circleId)
                                        }
                                    } label: {
                                        Label(
                                            NSLocalizedString("circles.detail.handleIt", comment: ""),
                                            systemImage: "gift"
                                        )
                                    }
                                }
                            }

                            Button {
                                UIPasteboard.general.string = item.name
                            } label: {
                                Label(
                                    NSLocalizedString("common.copy", comment: ""),
                                    systemImage: "doc.on.doc"
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

    // MARK: - Members Tab

    @ViewBuilder
    // swiftlint:disable:next function_body_length
    private func membersTabContent(_ detail: CircleDetailResponse) -> some View {
        ScrollView {
            LazyVStack(spacing: 0) {
                ForEach(detail.members) { member in
                    let isSelf = member.userId == currentUserId
                    let isMemberOwner = member.role == "owner"

                    HStack(spacing: OffriiTheme.spacingSM) {
                        AvatarView(
                            member.displayName ?? member.username,
                            size: .small
                        )

                        VStack(alignment: .leading, spacing: 2) {
                            HStack(spacing: OffriiTheme.spacingXS) {
                                Text(member.displayName ?? member.username)
                                    .font(OffriiTypography.body)
                                    .foregroundColor(OffriiTheme.text)

                                if isSelf {
                                    Text(NSLocalizedString(
                                        "circles.members.you",
                                        comment: ""
                                    ))
                                    .font(OffriiTypography.caption)
                                    .foregroundColor(OffriiTheme.textMuted)
                                }
                            }

                            Text("@\(member.username)")
                                .font(OffriiTypography.caption)
                                .foregroundColor(OffriiTheme.textMuted)
                        }

                        Spacer()

                        if isMemberOwner {
                            Text(NSLocalizedString(
                                "circles.members.owner",
                                comment: ""
                            ))
                            .font(OffriiTypography.caption)
                            .fontWeight(.medium)
                            .foregroundColor(OffriiTheme.accent)
                            .padding(.horizontal, OffriiTheme.spacingSM)
                            .padding(.vertical, OffriiTheme.spacingXXS)
                            .background(OffriiTheme.accent.opacity(0.1))
                            .cornerRadius(OffriiTheme.cornerRadiusFull)
                        }

                        if isOwner && !isSelf && !isMemberOwner {
                            Button {
                                memberToRemove = member
                            } label: {
                                Image(systemName: "minus.circle.fill")
                                    .font(.system(size: 20))
                                    .foregroundColor(OffriiTheme.danger)
                            }
                        }
                    }
                    .padding(.horizontal, OffriiTheme.spacingLG)
                    .padding(.vertical, OffriiTheme.spacingSM)
                    .contextMenu {
                        if isOwner && !isSelf {
                            if member.role != "owner" {
                                Button(role: .destructive) {
                                    memberToRemove = member
                                } label: {
                                    Label(
                                        NSLocalizedString("friends.remove", comment: ""),
                                        systemImage: "person.badge.minus"
                                    )
                                }
                            }
                            Button {
                                memberToTransfer = member
                            } label: {
                                Label(
                                    NSLocalizedString("circles.transferOwnership.action", comment: ""),
                                    systemImage: "arrow.right.arrow.left"
                                )
                            }
                        }
                    }

                    if member.id != detail.members.last?.id {
                        Divider()
                            .padding(.leading, 56)
                            .padding(.horizontal, OffriiTheme.spacingLG)
                    }
                }

                // Invite button
                Button {
                    showInvite = true
                } label: {
                    HStack(spacing: OffriiTheme.spacingSM) {
                        Image(systemName: "plus")
                            .font(.system(size: 13, weight: .semibold))
                            .foregroundColor(.white)
                            .frame(width: 32, height: 32)
                            .background(OffriiTheme.primary)
                            .clipShape(Circle())

                        Text(NSLocalizedString(
                            "circles.detail.inviteMembers",
                            comment: ""
                        ))
                        .font(OffriiTypography.body)
                        .foregroundColor(OffriiTheme.primary)
                    }
                    .padding(.horizontal, OffriiTheme.spacingLG)
                    .padding(.vertical, OffriiTheme.spacingMD)
                    .frame(maxWidth: .infinity, alignment: .leading)
                }
                .buttonStyle(.plain)

                // Leave button (non-owners only)
                if !isOwner {
                    Divider()
                        .padding(.horizontal, OffriiTheme.spacingLG)

                    Button {
                        showLeaveAlert = true
                    } label: {
                        HStack(spacing: OffriiTheme.spacingSM) {
                            Image(
                                systemName: "rectangle.portrait.and.arrow.right"
                            )
                            .font(.system(size: 14))
                            .foregroundColor(OffriiTheme.danger)

                            Text(NSLocalizedString(
                                "circles.members.leave",
                                comment: ""
                            ))
                            .font(OffriiTypography.body)
                            .foregroundColor(OffriiTheme.danger)
                        }
                        .padding(.horizontal, OffriiTheme.spacingLG)
                        .padding(.vertical, OffriiTheme.spacingMD)
                        .frame(maxWidth: .infinity, alignment: .leading)
                    }
                    .buttonStyle(.plain)
                }
            }
            .padding(.top, OffriiTheme.spacingSM)
        }
    }

    // MARK: - Item Card (matches WishlistGridCard design)

    @ViewBuilder
    // swiftlint:disable:next function_body_length
    private func circleItemCard(
        _ item: CircleItemResponse,
        showClaimButtons: Bool
    ) -> some View {
        let style = CategoryStyle(icon: item.categoryIcon)

        VStack(alignment: .leading, spacing: 0) {
            // Image zone — OG image or category gradient fallback
            ZStack {
                if let imageUrl = item.imageUrl ?? item.ogImageUrl,
                   let url = URL(string: imageUrl) {
                    LazyImage(url: url) { state in
                        if let image = state.image {
                            image
                                .resizable()
                                .aspectRatio(contentMode: .fill)
                                .frame(minWidth: 0, maxWidth: .infinity)
                                .frame(height: 130)
                                .clipped()
                        } else {
                            gradientPlaceholder(style: style)
                        }
                    }
                } else {
                    gradientPlaceholder(style: style)
                }

                // Status overlay
                if item.status == "purchased" {
                    Color.black.opacity(0.25)
                    VStack(spacing: 4) {
                        Image(systemName: "checkmark.circle.fill")
                            .font(.system(size: 20))
                        Text(NSLocalizedString("circles.detail.received", comment: ""))
                            .font(.system(size: 11, weight: .bold))
                            .tracking(1)
                            .textCase(.uppercase)
                    }
                    .foregroundColor(.white)
                } else if item.isClaimed {
                    Color.black.opacity(0.35)
                    Text(NSLocalizedString("wishlist.reserved", comment: ""))
                        .font(.system(size: 13, weight: .bold))
                        .tracking(2)
                        .textCase(.uppercase)
                        .foregroundColor(.white)
                }

                // Priority flames (top-right)
                if item.priority >= 2 {
                    HStack(spacing: 2) {
                        ForEach(0..<(Int(item.priority) - 1), id: \.self) { _ in
                            Image(systemName: "flame.fill")
                                .font(.system(size: 10))
                        }
                    }
                    .foregroundColor(
                        item.priority == 3 ? OffriiTheme.danger : OffriiTheme.primary
                    )
                    .padding(.horizontal, 6)
                    .padding(.vertical, 3)
                    .background(.white)
                    .cornerRadius(OffriiTheme.cornerRadiusXS)
                    .shadow(color: .black.opacity(0.08), radius: 4, x: 0, y: 1)
                    .frame(
                        maxWidth: .infinity, maxHeight: .infinity,
                        alignment: .topTrailing
                    )
                    .padding(OffriiTheme.spacingSM)
                }
            }
            .frame(height: 130)
            .clipped()

            // Text zone — fixed structure for uniform card height
            VStack(alignment: .leading, spacing: 2) {
                Text(item.name)
                    .font(.system(size: 14, weight: .semibold))
                    .foregroundColor(OffriiTheme.text)
                    .lineLimit(1)

                Text(item.estimatedPrice?.formatted(.currency(code: "EUR")) ?? " ")
                    .font(.system(size: 12))
                    .foregroundColor(OffriiTheme.textMuted)

                // Owner avatar + name (group only)
                if !isDirect {
                    HStack(spacing: 4) {
                        if let ownerName = item.sharedByName {
                            if let avatarStr = item.sharedByAvatarUrl, let url = URL(string: avatarStr) {
                                LazyImage(url: url) { state in
                                    if let image = state.image {
                                        image.resizable().aspectRatio(contentMode: .fill)
                                            .frame(width: 16, height: 16).clipShape(Circle())
                                    } else {
                                        itemOwnerInitial(ownerName)
                                    }
                                }
                            } else {
                                itemOwnerInitial(ownerName)
                            }
                            Text(ownerName)
                                .font(.system(size: 11))
                                .foregroundColor(OffriiTheme.textSecondary)
                                .lineLimit(1)
                        }
                    }
                    .frame(height: 16)
                }
            }
            .padding(.horizontal, OffriiTheme.spacingSM)
            .padding(.vertical, OffriiTheme.spacingSM)
            .frame(height: isDirect ? 48 : 64, alignment: .top)
        }
        .background(OffriiTheme.card)
        .cornerRadius(OffriiTheme.cornerRadiusLG)
        .shadow(color: OffriiTheme.cardShadowColor, radius: 6, x: 0, y: 2)
    }

    private func categoryIcon(for categoryId: UUID?) -> String? {
        guard let categoryId,
              let categories = viewModel.categories else { return nil }
        return categories.first { $0.id == categoryId }?.icon
    }

    private func itemOwnerInitial(_ name: String) -> some View {
        Text(String(name.prefix(1)).uppercased())
            .font(.system(size: 8, weight: .bold))
            .foregroundColor(.white)
            .frame(width: 16, height: 16)
            .background(OffriiTheme.primary)
            .clipShape(Circle())
    }

    private func gradientPlaceholder(style: CategoryStyle) -> some View {
        LinearGradient(
            colors: style.gradient,
            startPoint: .topLeading,
            endPoint: .bottomTrailing
        )
        .frame(height: 130)
        .overlay(
            Image(systemName: style.sfSymbol)
                .font(.system(size: 32, weight: .light))
                .foregroundColor(.white.opacity(0.7))
        )
    }

    // MARK: - Claim Button

    @ViewBuilder
    private func claimButton(_ item: CircleItemResponse) -> some View {
        if item.isClaimed {
            if item.claimedBy?.userId == currentUserId {
                // State: YOU claimed this — show unclaim option
                Button {
                    Task {
                        await viewModel.unclaimItem(itemId: item.id)
                        await viewModel.loadItems(circleId: circleId)
                        await viewModel.loadFeed(circleId: circleId)
                    }
                } label: {
                    HStack(spacing: 4) {
                        Image(systemName: "checkmark")
                            .font(.system(size: 9, weight: .bold))
                        Text(NSLocalizedString("circles.detail.youHandleIt", comment: ""))
                    }
                    .font(.system(size: 11, weight: .semibold))
                    .foregroundColor(OffriiTheme.primary)
                    .padding(.horizontal, OffriiTheme.spacingSM)
                    .padding(.vertical, OffriiTheme.spacingXXS)
                    .background(OffriiTheme.primary.opacity(0.15))
                    .cornerRadius(OffriiTheme.cornerRadiusXL)
                    .overlay(
                        RoundedRectangle(cornerRadius: OffriiTheme.cornerRadiusXL)
                            .strokeBorder(OffriiTheme.primary, lineWidth: 1)
                    )
                }
            } else if let claimer = item.claimedBy, !isDirect {
                // State: someone ELSE claimed — show who (group only)
                Text(String(
                    format: NSLocalizedString("circles.detail.reservedBy", comment: ""),
                    claimer.username
                ))
                .font(.system(size: 11))
                .foregroundColor(OffriiTheme.textMuted)
            } else {
                // State: someone claimed in 1:1 — no name
                Text(NSLocalizedString("wishlist.reserved", comment: ""))
                    .font(.system(size: 11))
                    .foregroundColor(OffriiTheme.textMuted)
            }
        } else {
            Button {
                Task {
                    await viewModel.claimItem(itemId: item.id)
                    await viewModel.loadItems(circleId: circleId)
                    await viewModel.loadFeed(circleId: circleId)
                }
            } label: {
                Text(NSLocalizedString("circles.detail.handleIt", comment: ""))
                    .font(.system(size: 11, weight: .semibold))
                    .foregroundColor(.white)
                    .padding(.horizontal, OffriiTheme.spacingSM)
                    .padding(.vertical, OffriiTheme.spacingXXS)
                    .background(OffriiTheme.primary)
                    .cornerRadius(OffriiTheme.cornerRadiusXL)
            }
        }
    }

    // MARK: - Helpers

    private func reload() async {
        async let detail: () = viewModel.loadDetail(circleId: circleId)
        async let categories: () = viewModel.loadCategories()
        _ = await (detail, categories)
        await viewModel.loadItems(circleId: circleId)
        await viewModel.loadFeed(circleId: circleId)
    }

    private func formattedDate(_ date: Date) -> String {
        let formatter = DateFormatter()
        formatter.dateFormat = "MMM yyyy"
        return formatter.string(from: date)
    }
}

// swiftlint:enable file_length type_body_length
