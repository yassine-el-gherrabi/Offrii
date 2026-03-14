import SwiftUI

// MARK: - HomeView

struct HomeView: View {
    @State private var vm = HomeViewModel()
    @State private var showProfile = false
    @Environment(AuthManager.self) private var authManager

    private var greeting: String {
        if let name = authManager.currentUser?.displayName, !name.isEmpty {
            return String(format: NSLocalizedString("home.greeting", comment: ""), name)
        }
        return NSLocalizedString("home.greetingDefault", comment: "")
    }

    var body: some View {
        ZStack {
            OffriiTheme.background.ignoresSafeArea()

            ScrollView {
                VStack(spacing: 0) {
                    // Header
                    SectionHeader(
                        title: greeting,
                        variant: .home
                    ) {
                        NavigationLink(destination: ProfileView()) {
                            ProfileAvatarButton(
                                initials: ProfileAvatarButton.initials(from: authManager.currentUser?.displayName)
                            )
                        }
                    }

                    VStack(spacing: OffriiTheme.spacingLG) {
                        // Section 1: Progress or Summary
                        if vm.isNewUser || vm.profileProgress.percentage < 100 {
                            ProfileProgressCard(progress: vm.profileProgress)
                        } else {
                            HomeSummaryCard(stats: vm.stats)
                        }

                        // Section 2: Quick Start (new user)
                        if vm.isNewUser {
                            QuickStartSection(completedActions: vm.completedActions)
                        }

                        // Section 3: Wishlist preview (established user)
                        if !vm.isNewUser && !vm.recentItems.isEmpty {
                            WishlistPreviewSection(items: vm.recentItems)
                        }

                        // Section 4: Community spotlight (always)
                        CommunitySpotlightSection(wishes: vm.communityWishes)
                    }
                    .padding(.horizontal, OffriiTheme.spacingBase)
                    .padding(.top, OffriiTheme.spacingBase)
                    .padding(.bottom, OffriiTheme.spacingXXL)
                }
            }
        }
        .navigationBarHidden(true)
        .task { await vm.load(authManager: authManager) }
        .refreshable { await vm.load(authManager: authManager) }
    }
}
