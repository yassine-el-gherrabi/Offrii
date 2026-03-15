import SwiftUI

// MARK: - CirclesListView

struct CirclesListView: View {
    @Environment(AuthManager.self) private var authManager
    @Environment(OnboardingTipManager.self) private var tipManager
    @State private var viewModel = CirclesViewModel()
    @State private var selectedSegment: Int = 0
    @State private var showCreateCircle = false
    @State private var showAddFriend = false
    @State private var showInviteContacts = false

    var body: some View {
        VStack(spacing: 0) {
            // Segmented picker
            Picker("", selection: $selectedSegment) {
                Text(NSLocalizedString("circles.tab.myCircles", comment: ""))
                    .tag(0)
                Text(NSLocalizedString("circles.tab.myFriends", comment: ""))
                    .tag(1)
            }
            .pickerStyle(.segmented)
            .padding(.horizontal, OffriiTheme.spacingLG)
            .padding(.vertical, OffriiTheme.spacingSM)

            // Content
            switch selectedSegment {
            case 0:
                CircleListContent(
                    viewModel: viewModel,
                    showCreateCircle: $showCreateCircle
                )
            default:
                FriendsListContent(
                    viewModel: viewModel,
                    showAddFriend: $showAddFriend,
                    showInviteContacts: $showInviteContacts
                )
            }
        }
        .background(OffriiTheme.background.ignoresSafeArea())
        .navigationTitle(NSLocalizedString("circles.title", comment: ""))
        .navigationBarTitleDisplayMode(.large)
        .toolbar {
            ToolbarItemGroup(placement: .topBarTrailing) {
                // Bell badge for pending requests
                Button {
                    selectedSegment = 1
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

                // Profile avatar
                NavigationLink(destination: ProfileView()) {
                    ProfileAvatarButton(
                        initials: ProfileAvatarButton.initials(
                            from: authManager.currentUser?.displayName
                        )
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
        .task {
            await viewModel.loadAll()
            tipManager.showIfNeeded(.circlesCreate)
        }
        .refreshable {
            if selectedSegment == 0 {
                await viewModel.loadCircles()
            } else {
                await viewModel.loadFriends()
                await viewModel.loadPendingRequests()
                await viewModel.loadSentRequests()
            }
        }
    }
}
