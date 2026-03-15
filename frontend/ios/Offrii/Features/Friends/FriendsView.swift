import SwiftUI

struct FriendsView: View {
    @State private var viewModel = CirclesViewModel()
    @State private var showAddFriend = false
    @State private var showInviteContacts = false

    var body: some View {
        ZStack {
            OffriiTheme.background.ignoresSafeArea()

            FriendsListContent(
                viewModel: viewModel,
                showAddFriend: $showAddFriend,
                showInviteContacts: $showInviteContacts
            )
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
        .task {
            await viewModel.loadFriends()
            await viewModel.loadPendingRequests()
            await viewModel.loadSentRequests()
        }
        .refreshable {
            await viewModel.loadFriends()
            await viewModel.loadPendingRequests()
            await viewModel.loadSentRequests()
        }
    }
}
