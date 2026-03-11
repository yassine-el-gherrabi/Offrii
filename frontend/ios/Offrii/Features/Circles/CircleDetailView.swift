import SwiftUI

struct CircleDetailView: View {
    let circleId: UUID
    @Environment(AuthManager.self) private var authManager
    @Environment(OnboardingTipManager.self) private var tipManager
    @State private var viewModel = CircleDetailViewModel()
    @State private var showInvite = false
    @State private var showMembers = false

    private var currentUserId: UUID? { authManager.currentUser?.id }

    var body: some View {
        ZStack {
            OffriiTheme.background.ignoresSafeArea()

            if viewModel.isLoading && viewModel.detail == nil {
                SkeletonList(count: 4)
                    .padding(.top, OffriiTheme.spacingBase)
            } else if let detail = viewModel.detail {
                VStack(spacing: 0) {
                    // Segmented control
                    Picker("", selection: $viewModel.selectedTab) {
                        Text(NSLocalizedString("circles.detail.items", comment: ""))
                            .tag(CircleDetailViewModel.DetailTab.items)
                        Text(NSLocalizedString("circles.detail.activity", comment: ""))
                            .tag(CircleDetailViewModel.DetailTab.activity)
                    }
                    .pickerStyle(.segmented)
                    .padding(.horizontal, OffriiTheme.spacingLG)
                    .padding(.vertical, OffriiTheme.spacingSM)

                    switch viewModel.selectedTab {
                    case .items:
                        itemsContent(detail)
                    case .activity:
                        activityContent()
                    }
                }
            } else if let error = viewModel.error {
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
        }
        .navigationTitle(viewModel.detail?.name ?? NSLocalizedString("circles.unnamed", comment: ""))
        .navigationBarTitleDisplayMode(.inline)
        .toolbar {
            ToolbarItem(placement: .primaryAction) {
                HStack(spacing: OffriiTheme.spacingSM) {
                    Button {
                        showMembers = true
                    } label: {
                        Image(systemName: "person.2")
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
                                message: OnboardingTipManager.message(for: .circlesShare),
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
        .sheet(isPresented: $showInvite) {
            if let detail = viewModel.detail {
                InviteFriendsSheet(
                    circleId: circleId,
                    existingMemberIds: Set(detail.members.map(\.userId)),
                    onInvited: { Task { await reload() } }
                )
            }
        }
        .sheet(isPresented: $showMembers) {
            if let detail = viewModel.detail {
                CircleMembersSheet(
                    circleId: circleId,
                    members: detail.members,
                    ownerId: detail.ownerId,
                    onLeft: { Task { await reload() } },
                    currentUserId: currentUserId
                )
            }
        }
        .refreshable {
            await reload()
        }
        .task {
            viewModel.currentUserId = currentUserId
            await reload()
            tipManager.showIfNeeded(.circlesShare)
        }
    }

    // MARK: - Items Tab

    @ViewBuilder
    private func itemsContent(_ detail: CircleDetailResponse) -> some View {
        if viewModel.items.isEmpty {
            Spacer()
            OffriiEmptyState(
                icon: "tray",
                title: NSLocalizedString("circles.detail.noItems", comment: ""),
                subtitle: NSLocalizedString("circles.detail.noItemsSubtitle", comment: "")
            )
            Spacer()
        } else {
            ScrollView {
                LazyVStack(alignment: .leading, spacing: OffriiTheme.spacingLG) {
                    ForEach(viewModel.itemsByMember, id: \.member.userId) { section in
                        memberSection(section.member, items: section.items)
                    }
                }
                .padding(OffriiTheme.spacingBase)
            }
        }
    }

    @ViewBuilder
    private func memberSection(_ member: CircleMember, items: [CircleItemResponse]) -> some View {
        let isCurrentUser = member.userId == viewModel.currentUserId

        VStack(alignment: .leading, spacing: OffriiTheme.spacingSM) {
            HStack(spacing: OffriiTheme.spacingSM) {
                AvatarView(member.displayName ?? member.username, size: .small)
                Text(isCurrentUser
                     ? NSLocalizedString("circles.detail.myWishes", comment: "")
                     : (member.displayName ?? member.username))
                    .font(OffriiTypography.headline)
                    .foregroundColor(OffriiTheme.text)
                Text("(\(items.count))")
                    .font(OffriiTypography.caption)
                    .foregroundColor(OffriiTheme.textMuted)
            }

            ForEach(items) { item in
                itemRow(item, isOwner: isCurrentUser)
            }
        }
    }

    @ViewBuilder
    private func itemRow(_ item: CircleItemResponse, isOwner: Bool) -> some View {
        HStack {
            VStack(alignment: .leading, spacing: 2) {
                Text(item.name)
                    .font(OffriiTypography.body)
                    .foregroundColor(OffriiTheme.text)

                if let price = item.estimatedPrice {
                    Text(price.formatted(.currency(code: "EUR")))
                        .font(OffriiTypography.caption)
                        .foregroundColor(OffriiTheme.textMuted)
                }
            }

            Spacer()

            if isOwner {
                // Anti-spoiler: owner sees "Reserved" but not who
                if item.isClaimed {
                    Label(NSLocalizedString("wishlist.reserved", comment: ""), systemImage: "gift.fill")
                        .font(OffriiTypography.caption)
                        .foregroundColor(OffriiTheme.accent)
                }
            } else {
                if item.isClaimed {
                    if item.claimedBy?.userId == viewModel.currentUserId {
                        Button {
                            Task {
                                await viewModel.unclaimItem(itemId: item.id)
                                await viewModel.loadItems(circleId: circleId)
                            }
                        } label: {
                            Label(NSLocalizedString("circles.detail.claimed", comment: ""), systemImage: "checkmark.circle.fill")
                                .font(OffriiTypography.caption)
                                .foregroundColor(OffriiTheme.success)
                        }
                    } else {
                        Text(NSLocalizedString("wishlist.reserved", comment: ""))
                            .font(OffriiTypography.caption)
                            .foregroundColor(OffriiTheme.textMuted)
                    }
                } else {
                    Button {
                        Task {
                            await viewModel.claimItem(itemId: item.id)
                            await viewModel.loadItems(circleId: circleId)
                        }
                    } label: {
                        Text(NSLocalizedString("circles.detail.handleIt", comment: ""))
                            .font(OffriiTypography.caption)
                            .fontWeight(.semibold)
                            .foregroundColor(.white)
                            .padding(.horizontal, OffriiTheme.spacingSM)
                            .padding(.vertical, OffriiTheme.spacingXS)
                            .background(OffriiTheme.primary)
                            .cornerRadius(OffriiTheme.cornerRadiusXL)
                    }
                }
            }
        }
        .padding(OffriiTheme.spacingMD)
        .background(OffriiTheme.card)
        .cornerRadius(OffriiTheme.cornerRadiusLG)
    }

    // MARK: - Activity Tab

    @ViewBuilder
    private func activityContent() -> some View {
        if viewModel.feed.isEmpty {
            Spacer()
            OffriiEmptyState(
                icon: "bell.slash",
                title: NSLocalizedString("circles.detail.noActivity", comment: ""),
                subtitle: NSLocalizedString("circles.detail.noActivitySubtitle", comment: "")
            )
            Spacer()
        } else {
            List {
                ForEach(viewModel.feed) { event in
                    eventRow(event)
                }
            }
            .listStyle(.plain)
        }
    }

    @ViewBuilder
    private func eventRow(_ event: CircleEventResponse) -> some View {
        HStack(spacing: OffriiTheme.spacingSM) {
            Image(systemName: iconForEvent(event.eventType))
                .foregroundColor(OffriiTheme.primary)
                .frame(width: 24)

            VStack(alignment: .leading, spacing: 2) {
                Text(descriptionForEvent(event))
                    .font(OffriiTypography.body)
                    .foregroundColor(OffriiTheme.text)

                Text(event.createdAt, style: .relative)
                    .font(OffriiTypography.caption)
                    .foregroundColor(OffriiTheme.textMuted)
            }
        }
        .padding(.vertical, OffriiTheme.spacingXS)
    }

    private func iconForEvent(_ type: String) -> String {
        switch type {
        case "item_shared": return "square.and.arrow.up"
        case "item_claimed": return "gift.fill"
        case "item_unclaimed": return "gift"
        case "member_joined": return "person.badge.plus"
        case "member_left": return "person.badge.minus"
        default: return "bell.fill"
        }
    }

    private func descriptionForEvent(_ event: CircleEventResponse) -> String {
        let actor = event.actorUsername ?? NSLocalizedString("circles.detail.someone", comment: "")
        let itemName = event.targetItemName ?? ""
        let isOwner = event.targetUserId == viewModel.currentUserId

        switch event.eventType {
        case "item_shared":
            return String(format: NSLocalizedString("circles.event.itemShared", comment: ""), actor, itemName)
        case "item_claimed":
            if isOwner {
                return String(format: NSLocalizedString("circles.event.itemClaimedAnti", comment: ""), itemName)
            }
            return String(format: NSLocalizedString("circles.event.itemClaimed", comment: ""), actor, itemName)
        case "item_unclaimed":
            return String(format: NSLocalizedString("circles.event.itemUnclaimed", comment: ""), actor, itemName)
        case "member_joined":
            let target = event.targetUsername ?? actor
            return String(format: NSLocalizedString("circles.event.memberJoined", comment: ""), target)
        case "member_left":
            let target = event.targetUsername ?? actor
            return String(format: NSLocalizedString("circles.event.memberLeft", comment: ""), target)
        default:
            return event.eventType
        }
    }

    // MARK: - Helpers

    private func reload() async {
        await viewModel.loadDetail(circleId: circleId)
        await viewModel.loadItems(circleId: circleId)
        await viewModel.loadFeed(circleId: circleId)
    }
}
