import SwiftUI

struct CircleDetailView: View {
    let circleId: UUID
    @Environment(AuthManager.self) private var authManager
    @Environment(OnboardingTipManager.self) private var tipManager
    @Environment(\.dismiss) private var dismiss
    @State private var viewModel = CircleDetailViewModel()
    @State private var showInvite = false
    @State private var showEdit = false
    @State private var showLeaveAlert = false

    private var currentUserId: UUID? { authManager.currentUser?.id }
    private var isOwner: Bool { viewModel.detail?.ownerId == currentUserId }

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
                VStack(spacing: 0) {
                    // Member carousel filter
                    MemberCarousel(
                        members: detail.members,
                        selectedMemberId: $viewModel.selectedMemberFilter,
                        currentUserId: currentUserId
                    )

                    // 3-tab segmented control
                    Picker("", selection: $viewModel.selectedTab) {
                        Text(NSLocalizedString("circles.detail.items", comment: ""))
                            .tag(CircleDetailViewModel.DetailTab.items)
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
                        itemsTabContent(detail)
                    case .members:
                        membersTabContent(detail)
                    case .activity:
                        CircleActivityFeed(
                            events: viewModel.feed,
                            currentUserId: currentUserId
                        )
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
        .navigationTitle(
            viewModel.detail?.name
                ?? NSLocalizedString("circles.unnamed", comment: "")
        )
        .navigationBarTitleDisplayMode(.inline)
        .toolbar {
            ToolbarItem(placement: .primaryAction) {
                HStack(spacing: OffriiTheme.spacingSM) {
                    if isOwner {
                        Button {
                            showEdit = true
                        } label: {
                            Image(systemName: "pencil")
                                .font(.system(size: 16))
                        }
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
                    currentName: name
                ) {
                    Task { await reload() }
                }
                .presentationDetents([.medium])
            }
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
    private func itemsTabContent(_ detail: CircleDetailResponse) -> some View {
        let displayed = viewModel.filteredItems

        if displayed.isEmpty {
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
                    ForEach(displayed) { item in
                        circleItemCard(item)
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
                    }
                    .padding(.horizontal, OffriiTheme.spacingLG)
                    .padding(.vertical, OffriiTheme.spacingSM)
                    .contextMenu {
                        if isOwner && !isSelf && member.role != "owner" {
                            Button(role: .destructive) {
                                Task {
                                    await viewModel.removeMember(
                                        circleId: circleId,
                                        userId: member.userId
                                    )
                                }
                            } label: {
                                Label(
                                    NSLocalizedString("friends.remove", comment: ""),
                                    systemImage: "person.badge.minus"
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

    // MARK: - Item Card

    @ViewBuilder
    private func circleItemCard(_ item: CircleItemResponse) -> some View {
        let itemIsOwner = item.sharedBy == currentUserId

        VStack(alignment: .leading, spacing: 0) {
            LinearGradient(
                colors: [
                    OffriiTheme.primary.opacity(0.25),
                    OffriiTheme.accent.opacity(0.15),
                ],
                startPoint: .topLeading,
                endPoint: .bottomTrailing
            )
            .frame(height: 100)
            .overlay(
                Image(systemName: "gift.fill")
                    .font(.system(size: 28, weight: .light))
                    .foregroundColor(.white.opacity(0.6))
            )
            .overlay(alignment: .topTrailing) {
                if item.isClaimed && itemIsOwner {
                    HStack(spacing: 3) {
                        Image(systemName: "lock.fill")
                            .font(.system(size: 8))
                        Text(NSLocalizedString("wishlist.reserved", comment: ""))
                            .font(.system(size: 9, weight: .semibold))
                    }
                    .foregroundColor(OffriiTheme.accent)
                    .padding(.horizontal, 6)
                    .padding(.vertical, 3)
                    .background(.ultraThinMaterial)
                    .cornerRadius(OffriiTheme.cornerRadiusXS)
                    .padding(OffriiTheme.spacingSM)
                }
            }

            VStack(alignment: .leading, spacing: OffriiTheme.spacingXXS) {
                Text(item.name)
                    .font(.system(size: 14, weight: .semibold))
                    .foregroundColor(OffriiTheme.text)
                    .lineLimit(2)

                if let price = item.estimatedPrice {
                    Text(price.formatted(.currency(code: "EUR")))
                        .font(.system(size: 12))
                        .foregroundColor(OffriiTheme.textMuted)
                }

                if !itemIsOwner {
                    claimButton(item)
                }
            }
            .padding(.horizontal, OffriiTheme.spacingSM)
            .padding(.vertical, OffriiTheme.spacingSM)
        }
        .background(OffriiTheme.card)
        .cornerRadius(OffriiTheme.cornerRadiusLG)
        .shadow(color: OffriiTheme.cardShadowColor, radius: 6, x: 0, y: 2)
    }

    // MARK: - Claim Button

    @ViewBuilder
    private func claimButton(_ item: CircleItemResponse) -> some View {
        if item.isClaimed {
            if item.claimedBy?.userId == currentUserId {
                Button {
                    Task {
                        await viewModel.unclaimItem(itemId: item.id)
                        await viewModel.loadItems(circleId: circleId)
                    }
                } label: {
                    Label(
                        NSLocalizedString("circles.detail.claimed", comment: ""),
                        systemImage: "checkmark.circle.fill"
                    )
                    .font(.system(size: 11, weight: .medium))
                    .foregroundColor(OffriiTheme.success)
                }
            } else {
                Text(NSLocalizedString("wishlist.reserved", comment: ""))
                    .font(.system(size: 11))
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
        await viewModel.loadDetail(circleId: circleId)
        await viewModel.loadItems(circleId: circleId)
        await viewModel.loadFeed(circleId: circleId)
    }
}
