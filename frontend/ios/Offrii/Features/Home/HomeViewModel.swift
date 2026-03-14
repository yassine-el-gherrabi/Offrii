import SwiftUI

// MARK: - HomeViewModel

@Observable
@MainActor
final class HomeViewModel {
    var recentItems: [Item] = []
    var communityWishes: [CommunityWish] = []
    var isLoading = false
    var profileProgress = ProfileProgress()

    struct Stats {
        var totalItems: Int = 0
        var claimedItems: Int = 0
        var circleCount: Int = 0
        var friendCount: Int = 0
    }
    var stats = Stats()

    var isNewUser: Bool {
        recentItems.isEmpty && stats.circleCount == 0
    }

    var completedActions: Set<QuickStartAction> {
        var actions = Set<QuickStartAction>()
        if profileProgress.hasFirstItem { actions.insert(.addWish) }
        if profileProgress.hasFirstCircle { actions.insert(.createCircle) }
        if profileProgress.hasFirstFriend { actions.insert(.inviteFriend) }
        if profileProgress.hasSharedList { actions.insert(.shareList) }
        return actions
    }

    // MARK: - Load

    func load(authManager: AuthManager) async {
        isLoading = true
        defer { isLoading = false }

        async let itemsResult = ItemService.shared.listItems(
            sort: "created_at", order: "desc", page: 1, perPage: 3
        )
        async let communityResult = CommunityWishService.shared.listWishes(page: 1, limit: 3)
        async let circlesResult = CircleService.shared.listCircles()
        async let friendsResult = FriendService.shared.listFriends()

        // Items
        if let response = try? await itemsResult {
            recentItems = response.items
            stats.totalItems = response.total
            stats.claimedItems = response.items.filter { $0.isClaimed }.count
        }

        // Community
        if let response = try? await communityResult {
            communityWishes = response.data
        }

        // Circles
        if let circles = try? await circlesResult {
            stats.circleCount = circles.count
        }

        // Friends
        if let friends = try? await friendsResult {
            stats.friendCount = friends.count
        }

        // Compute profile progress
        computeProgress(user: authManager.currentUser)
    }

    // MARK: - Progress Computation

    private func computeProgress(user: User?) {
        guard let user else { return }

        profileProgress.hasUsername = !user.username.isEmpty && user.username != user.email
        profileProgress.hasDisplayName = user.displayName != nil && !user.displayName!.isEmpty
        profileProgress.hasFirstItem = stats.totalItems > 0
        profileProgress.hasFirstCircle = stats.circleCount > 0
        profileProgress.hasFirstFriend = stats.friendCount > 0
        profileProgress.hasReminders = user.reminderFreq != "never"
        // hasSharedList stays false until we track share events
        profileProgress.hasSharedList = false
    }
}
