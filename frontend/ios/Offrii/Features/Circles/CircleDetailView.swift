import SwiftUI

struct CircleDetailView: View {
    let circleId: UUID
    @Environment(AuthManager.self) private var authManager
    @Environment(OnboardingTipManager.self) private var tipManager
    @State private var viewModel = CircleDetailViewModel()
    @State private var showInvite = false
    @State private var showMembers = false
    @State private var selectedMemberId: UUID?

    private var currentUserId: UUID? { authManager.currentUser?.id }

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
                    // Member carousel
                    MemberCarousel(
                        members: detail.members,
                        selectedMemberId: $selectedMemberId,
                        currentUserId: currentUserId
                    )

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
                        itemsGridContent(detail)
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

    // MARK: - Items Grid

    @ViewBuilder
    private func itemsGridContent(_ detail: CircleDetailResponse) -> some View {
        let filteredItems = filteredItemsForMember()

        if filteredItems.isEmpty {
            Spacer()
            OffriiEmptyState(
                icon: "tray",
                title: NSLocalizedString("circles.detail.noItems", comment: ""),
                subtitle: NSLocalizedString("circles.detail.noItemsSubtitle", comment: "")
            )
            Spacer()
        } else {
            ScrollView {
                LazyVGrid(columns: gridColumns, spacing: OffriiTheme.spacingSM) {
                    ForEach(filteredItems) { item in
                        circleItemCard(item)
                    }
                }
                .padding(.horizontal, OffriiTheme.spacingBase)
                .padding(.vertical, OffriiTheme.spacingSM)
            }
        }
    }

    private func filteredItemsForMember() -> [CircleItemResponse] {
        guard let memberId = selectedMemberId else {
            return viewModel.items
        }
        return viewModel.items.filter { $0.sharedBy == memberId }
    }

    @ViewBuilder
    private func circleItemCard(_ item: CircleItemResponse) -> some View {
        let isOwner = item.sharedBy == currentUserId

        VStack(alignment: .leading, spacing: 0) {
            // Image placeholder
            LinearGradient(
                colors: [OffriiTheme.primary.opacity(0.25), OffriiTheme.accent.opacity(0.15)],
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
                if item.isClaimed {
                    if isOwner {
                        // Owner sees "reserved" but not by whom
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
            }

            // Text + claim action
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

                if !isOwner {
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
                    Label(NSLocalizedString("circles.detail.claimed", comment: ""), systemImage: "checkmark.circle.fill")
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
