import SwiftUI

struct FriendsView: View {
    @Environment(AuthManager.self) private var authManager
    @State private var viewModel = FriendsViewModel()
    @State private var showAddFriend = false

    var body: some View {
        ZStack {
            OffriiTheme.cardSurface.ignoresSafeArea()

            if viewModel.isLoading && viewModel.friends.isEmpty && viewModel.pendingRequests.isEmpty && viewModel.sentRequests.isEmpty {
                ProgressView()
            } else {
                List {
                    // Pending requests section
                    if !viewModel.pendingRequests.isEmpty {
                        Section {
                            ForEach(viewModel.pendingRequests) { request in
                                pendingRow(request)
                            }
                        } header: {
                            Text(String(format: NSLocalizedString("friends.pendingSection", comment: ""),
                                        viewModel.pendingRequests.count))
                                .font(OffriiTypography.headline)
                                .foregroundColor(OffriiTheme.text)
                                .textCase(nil)
                        }
                    }

                    // Sent requests section
                    if !viewModel.sentRequests.isEmpty {
                        Section {
                            ForEach(viewModel.sentRequests) { request in
                                sentRow(request)
                            }
                        } header: {
                            Text(String(format: NSLocalizedString("friends.sentSection", comment: ""),
                                        viewModel.sentRequests.count))
                                .font(OffriiTypography.headline)
                                .foregroundColor(OffriiTheme.text)
                                .textCase(nil)
                        }
                    }

                    // Friends list
                    Section {
                        if viewModel.friends.isEmpty {
                            Text(NSLocalizedString("friends.empty", comment: ""))
                                .font(OffriiTypography.body)
                                .foregroundColor(OffriiTheme.textSecondary)
                                .frame(maxWidth: .infinity, alignment: .center)
                                .padding(.vertical, OffriiTheme.spacingLG)
                        } else {
                            ForEach(viewModel.friends) { friend in
                                friendRow(friend)
                                    .swipeActions(edge: .trailing) {
                                        Button(role: .destructive) {
                                            Task { await viewModel.removeFriend(friend) }
                                        } label: {
                                            Label(NSLocalizedString("common.delete", comment: ""),
                                                  systemImage: "trash")
                                        }
                                    }
                            }
                        }
                    } header: {
                        if !viewModel.friends.isEmpty {
                            Text(String(format: NSLocalizedString("friends.count", comment: ""),
                                        viewModel.friends.count))
                                .font(OffriiTypography.headline)
                                .foregroundColor(OffriiTheme.text)
                                .textCase(nil)
                        }
                    }
                }
                .listStyle(.plain)
            }
        }
        .navigationTitle(NSLocalizedString("friends.title", comment: ""))
        .navigationBarTitleDisplayMode(.inline)
        .toolbar {
            ToolbarItem(placement: .primaryAction) {
                Button {
                    showAddFriend = true
                } label: {
                    Image(systemName: "plus")
                }
            }
        }
        .sheet(isPresented: $showAddFriend) {
            AddFriendSheet {
                Task { await viewModel.loadAll() }
            }
        }
        .refreshable {
            await viewModel.loadAll()
        }
        .task {
            await viewModel.loadAll()
        }
    }

    // MARK: - Rows

    @ViewBuilder
    private func pendingRow(_ request: FriendRequestResponse) -> some View {
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
        .padding(.vertical, OffriiTheme.spacingXS)
    }

    @ViewBuilder
    private func sentRow(_ request: SentFriendRequestResponse) -> some View {
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
                Task { await viewModel.cancelRequest(request) }
            } label: {
                Text(NSLocalizedString("friends.sent.cancel", comment: ""))
                    .font(OffriiTypography.footnote)
                    .foregroundColor(OffriiTheme.danger)
                    .padding(.horizontal, OffriiTheme.spacingSM)
                    .padding(.vertical, OffriiTheme.spacingXS)
                    .background(OffriiTheme.danger.opacity(0.1))
                    .cornerRadius(OffriiTheme.cornerRadiusXL)
            }
        }
        .padding(.vertical, OffriiTheme.spacingXS)
    }

    @ViewBuilder
    private func friendRow(_ friend: FriendResponse) -> some View {
        HStack(spacing: OffriiTheme.spacingSM) {
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
        }
        .padding(.vertical, OffriiTheme.spacingXS)
    }
}
