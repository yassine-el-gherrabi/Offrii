import SwiftUI

struct FriendsView: View {
    @State private var viewModel = CirclesViewModel()
    @State private var showAddFriend = false

    var body: some View {
        ZStack {
            OffriiTheme.background.ignoresSafeArea()

            if viewModel.isLoadingFriends && viewModel.friends.isEmpty {
                SkeletonList(count: 5)
                    .padding(.top, OffriiTheme.spacingBase)
            } else if viewModel.friends.isEmpty {
                OffriiEmptyState(
                    icon: "person.crop.circle.badge.plus",
                    title: NSLocalizedString("friends.emptyTitle", comment: ""),
                    subtitle: NSLocalizedString("friends.emptySubtitle", comment: ""),
                    ctaTitle: NSLocalizedString("friends.search.placeholder", comment: ""),
                    ctaAction: { showAddFriend = true }
                )
            } else {
                List {
                    ForEach(viewModel.friends) { friend in
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
                                    format: NSLocalizedString(
                                        "friends.wishCount",
                                        comment: ""
                                    ),
                                    friend.sharedItemCount
                                ))
                                .font(OffriiTypography.caption)
                                .foregroundColor(OffriiTheme.primary)
                            }
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
                    Image(systemName: "person.badge.plus")
                        .font(.system(size: 16))
                }
            }
        }
        .sheet(isPresented: $showAddFriend) {
            AddFriendSheet {
                Task { await viewModel.loadFriends() }
            }
        }
        .task {
            await viewModel.loadFriends()
        }
        .refreshable {
            await viewModel.loadFriends()
        }
    }
}
