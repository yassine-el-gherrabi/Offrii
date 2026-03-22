import SwiftUI

// MARK: - Friends List Content (extracted from CirclesListView for type_body_length)

struct FriendsListContent: View {
    var viewModel: CirclesViewModel
    @Binding var showAddFriend: Bool
    @Binding var friendToRemove: FriendResponse?
    @Binding var showAcceptToast: Bool
    @Binding var acceptedName: String

    var body: some View {
        if viewModel.isLoadingFriends && viewModel.friends.isEmpty {
            ScrollView {
                SkeletonList(count: 5)
                    .padding(.top, OffriiTheme.spacingBase)
            }
        } else if viewModel.friends.isEmpty
                    && viewModel.pendingRequests.isEmpty
                    && viewModel.sentRequests.isEmpty {
            Spacer()
            OffriiEmptyState(
                icon: "person.crop.circle.badge.plus",
                title: NSLocalizedString("friends.emptyTitle", comment: ""),
                subtitle: NSLocalizedString("friends.emptySubtitle", comment: ""),
                ctaTitle: NSLocalizedString("friends.add.title", comment: ""),
                ctaAction: { showAddFriend = true }
            )
            Spacer()
        } else {
            ScrollView {
                LazyVStack(spacing: OffriiTheme.spacingSM) {
                    if !viewModel.pendingRequests.isEmpty {
                        pendingSection
                    }

                    if !viewModel.sentRequests.isEmpty {
                        sentSection
                    }

                    ForEach(viewModel.filteredFriends) { friend in
                        friendRow(friend)
                    }
                }
                .padding(.horizontal, OffriiTheme.spacingBase)
                .padding(.vertical, OffriiTheme.spacingSM)
            }
            .refreshable {
                await viewModel.loadAll()
            }
        }
    }

    // MARK: - Pending Section

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
                    .background(OffriiTheme.primary)
                    .clipShape(Capsule())

                Spacer()
            }

            ForEach(viewModel.pendingRequests) { request in
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
            AvatarView(request.fromDisplayName ?? request.fromUsername, size: .small)

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
                    Task {
                        let name = request.fromDisplayName ?? request.fromUsername
                        await viewModel.acceptRequest(request)
                        OffriiHaptics.success()
                        acceptedName = name
                        showAcceptToast = true
                    }
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
                    Task {
                        await viewModel.declineRequest(request)
                        OffriiHaptics.tap()
                    }
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

    // MARK: - Sent Section

    @ViewBuilder
    private var sentSection: some View {
        VStack(alignment: .leading, spacing: OffriiTheme.spacingSM) {
            Text(NSLocalizedString("friends.sent", comment: ""))
                .font(OffriiTypography.headline)
                .foregroundColor(OffriiTheme.text)

            ForEach(viewModel.sentRequests) { request in
                HStack(spacing: OffriiTheme.spacingSM) {
                    AvatarView(request.toDisplayName ?? request.toUsername, size: .small)

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
                        Task {
                            await viewModel.cancelRequest(request)
                            OffriiHaptics.tap()
                        }
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
            }
        }
        .padding(OffriiTheme.spacingBase)
        .background(OffriiTheme.card)
        .cornerRadius(OffriiTheme.cornerRadiusLG)
        .shadow(color: OffriiTheme.cardShadowColor, radius: 6, x: 0, y: 2)
    }

    // MARK: - Friend Row

    @ViewBuilder
    private func friendRow(_ friend: FriendResponse) -> some View {
        let cardContent = HStack(spacing: OffriiTheme.spacingSM) {
            AvatarView(friend.displayName ?? friend.username, size: .small)

            VStack(alignment: .leading, spacing: 2) {
                Text(friend.displayName ?? friend.username)
                    .font(OffriiTypography.body)
                    .foregroundColor(OffriiTheme.text)
                Text("@\(friend.username)")
                    .font(OffriiTypography.caption)
                    .foregroundColor(OffriiTheme.textMuted)
            }

            Spacer()

            if friend.sharedItemCount > 0 {
                Text(String(
                    format: NSLocalizedString("friends.wishCount", comment: ""),
                    friend.sharedItemCount
                ))
                .font(OffriiTypography.caption)
                .foregroundColor(OffriiTheme.primary)
            }

            if viewModel.directCircle(for: friend) != nil {
                Image(systemName: "chevron.right")
                    .font(.system(size: 12))
                    .foregroundColor(OffriiTheme.textMuted)
            }
        }
        .padding(OffriiTheme.spacingSM)
        .background(OffriiTheme.card)
        .cornerRadius(OffriiTheme.cornerRadiusSM)

        Group {
            if let circleId = viewModel.directCircle(for: friend)?.id {
                NavigationLink(value: circleId) { cardContent }.buttonStyle(.plain)
            } else {
                cardContent
            }
        }
        .contextMenu {
            Button(role: .destructive) {
                friendToRemove = friend
            } label: {
                Label(NSLocalizedString("friends.remove", comment: ""), systemImage: "person.badge.minus")
            }
        }
    }

}
