import SwiftUI

// MARK: - FriendsListContent

struct FriendsListContent: View {
    var viewModel: CirclesViewModel
    @Binding var showAddFriend: Bool
    @Binding var showInviteContacts: Bool

    var body: some View {
        let isEmpty = viewModel.friends.isEmpty
            && viewModel.pendingRequests.isEmpty
            && viewModel.sentRequests.isEmpty

        if viewModel.isLoadingFriends && isEmpty {
            SkeletonList(count: 5)
                .padding(.top, OffriiTheme.spacingBase)
        } else if isEmpty {
            Spacer()
            OffriiEmptyState(
                icon: "person.crop.circle.badge.plus",
                title: NSLocalizedString("friends.emptyTitle", comment: ""),
                subtitle: NSLocalizedString("friends.emptySubtitle", comment: ""),
                ctaTitle: NSLocalizedString("friends.inviteContacts", comment: ""),
                ctaAction: { showAddFriend = true }
            )
            Spacer()
        } else {
            ScrollView {
                VStack(spacing: 0) {
                    searchBar
                        .padding(.horizontal, OffriiTheme.spacingLG)
                        .padding(.top, OffriiTheme.spacingSM)
                        .padding(.bottom, OffriiTheme.spacingBase)

                    if !viewModel.pendingRequests.isEmpty {
                        pendingSection
                    }

                    if !viewModel.sentRequests.isEmpty {
                        sentSection
                    }

                    friendsSection

                    actionButtons
                        .padding(.top, OffriiTheme.spacingLG)
                        .padding(.bottom, OffriiTheme.spacingXXL)
                }
            }
        }
    }

    // MARK: - Search Bar

    @ViewBuilder
    private var searchBar: some View {
        HStack(spacing: OffriiTheme.spacingSM) {
            Image(systemName: "magnifyingglass")
                .foregroundColor(OffriiTheme.textMuted)

            TextField(
                NSLocalizedString("friends.search.placeholder", comment: ""),
                text: Bindable(viewModel).friendSearchQuery
            )
            .font(OffriiTypography.body)
            .autocapitalization(.none)
            .autocorrectionDisabled()

            if !viewModel.friendSearchQuery.isEmpty {
                Button {
                    viewModel.friendSearchQuery = ""
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

    // MARK: - Pending Section

    @ViewBuilder
    private var pendingSection: some View {
        VStack(alignment: .leading, spacing: OffriiTheme.spacingSM) {
            HStack {
                Text(NSLocalizedString("friends.pending", comment: ""))
                    .font(OffriiTypography.headline)
                    .foregroundColor(OffriiTheme.text)

                Text("\(viewModel.pendingRequests.count)")
                    .font(OffriiTypography.caption)
                    .fontWeight(.bold)
                    .foregroundColor(.white)
                    .padding(.horizontal, 6)
                    .padding(.vertical, 2)
                    .background(OffriiTheme.danger)
                    .clipShape(Capsule())
            }
            .padding(.horizontal, OffriiTheme.spacingLG)

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
                .padding(.vertical, OffriiTheme.spacingXS)
            }
        }
        .padding(.bottom, OffriiTheme.spacingBase)
    }

    // MARK: - Sent Section

    @ViewBuilder
    private var sentSection: some View {
        VStack(alignment: .leading, spacing: OffriiTheme.spacingSM) {
            Text(NSLocalizedString("friends.sent", comment: ""))
                .font(OffriiTypography.headline)
                .foregroundColor(OffriiTheme.text)
                .padding(.horizontal, OffriiTheme.spacingLG)

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
        .padding(.bottom, OffriiTheme.spacingBase)
    }

    // MARK: - Friends Section

    @ViewBuilder
    private var friendsSection: some View {
        let displayed = viewModel.filteredFriends

        VStack(alignment: .leading, spacing: OffriiTheme.spacingSM) {
            if !viewModel.friends.isEmpty {
                Text(NSLocalizedString("friends.title", comment: ""))
                    .font(OffriiTypography.headline)
                    .foregroundColor(OffriiTheme.text)
                    .padding(.horizontal, OffriiTheme.spacingLG)
            }

            ForEach(displayed) { friend in
                HStack(spacing: OffriiTheme.spacingSM) {
                    AvatarView(
                        friend.displayName ?? friend.username,
                        size: .small
                    )

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
                        .padding(.horizontal, OffriiTheme.spacingSM)
                        .padding(.vertical, OffriiTheme.spacingXXS)
                        .background(OffriiTheme.primary.opacity(0.1))
                        .cornerRadius(OffriiTheme.cornerRadiusFull)
                    }
                }
                .padding(.horizontal, OffriiTheme.spacingLG)
                .padding(.vertical, OffriiTheme.spacingXS)
                .contextMenu {
                    Button(role: .destructive) {
                        Task { await viewModel.removeFriend(friend) }
                    } label: {
                        Label(
                            NSLocalizedString("friends.remove", comment: ""),
                            systemImage: "trash"
                        )
                    }
                }
            }
        }
    }

    // MARK: - Action Buttons

    @ViewBuilder
    private var actionButtons: some View {
        VStack(spacing: OffriiTheme.spacingSM) {
            Button {
                showAddFriend = true
            } label: {
                Label(
                    NSLocalizedString("friends.search.placeholder", comment: ""),
                    systemImage: "magnifyingglass"
                )
                .font(OffriiTypography.footnote)
                .fontWeight(.semibold)
                .foregroundColor(OffriiTheme.primary)
                .frame(maxWidth: .infinity)
                .padding(.vertical, OffriiTheme.spacingSM)
                .background(OffriiTheme.primary.opacity(0.1))
                .cornerRadius(OffriiTheme.cornerRadiusXL)
            }

            Button {
                showInviteContacts = true
            } label: {
                Label(
                    NSLocalizedString("friends.inviteContacts", comment: ""),
                    systemImage: "person.crop.circle.badge.plus"
                )
                .font(OffriiTypography.footnote)
                .fontWeight(.semibold)
                .foregroundColor(OffriiTheme.primary)
                .frame(maxWidth: .infinity)
                .padding(.vertical, OffriiTheme.spacingSM)
                .background(OffriiTheme.primary.opacity(0.1))
                .cornerRadius(OffriiTheme.cornerRadiusXL)
            }
        }
        .padding(.horizontal, OffriiTheme.spacingLG)
    }
}
