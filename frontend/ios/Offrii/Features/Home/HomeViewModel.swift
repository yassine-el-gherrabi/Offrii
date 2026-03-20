import SwiftUI
import UserNotifications

// MARK: - HomeViewModel

@Observable
@MainActor
final class HomeViewModel {
    var recentItems: [Item] = []
    var communityWishes: [CommunityWish] = []
    var categories: [CategoryResponse] = []
    var recentNotifications: [AppNotification] = []
    var unreadNotificationCount: Int = 0
    var isLoading = false
    var profileProgress = ProfileProgress()

    struct Stats {
        var totalItems: Int = 0
        var claimedItems: Int = 0
        var sharedItems: Int = 0
        var purchasedItems: Int = 0
    }
    var stats = Stats()

    // MARK: - Load

    func load(authManager: AuthManager) async {
        isLoading = true
        defer { isLoading = false }

        // Group 1: Content data
        async let itemsResult = ItemService.shared.listItems(
            sort: "created_at", order: "desc", page: 1, perPage: 4
        )
        async let purchasedResult = ItemService.shared.listItems(
            status: "purchased", page: 1, perPage: 1
        )
        async let communityResult = CommunityWishService.shared.listWishes(page: 1, limit: 3)
        async let categoriesResult = CategoryService.shared.listCategories()
        async let notificationsResult = NotificationCenterService.shared.list(page: 1, limit: 5)
        async let unreadResult = NotificationCenterService.shared.unreadCount()

        // Items (active)
        if let response = try? await itemsResult {
            recentItems = response.items
            stats.totalItems = response.total
            stats.claimedItems = response.items.filter { $0.isClaimed }.count
            stats.sharedItems = response.items.filter { !$0.sharedCircles.isEmpty }.count
        }

        // Purchased items count
        if let response = try? await purchasedResult {
            stats.purchasedItems = response.total
        }

        // Community
        if let response = try? await communityResult {
            communityWishes = response.data
        }

        // Categories
        if let cats = try? await categoriesResult {
            categories = cats
        }

        // Notifications
        if let response = try? await notificationsResult {
            recentNotifications = response.data
        }
        unreadNotificationCount = (try? await unreadResult) ?? 0

        // Group 2: Profile progress data (separate calls to avoid concurrency issues)
        let circleCount = (try? await CircleService.shared.listCircles())?.count ?? 0
        let friendCount = (try? await FriendService.shared.listFriends())?.count ?? 0
        let shareRules = (try? await CircleService.shared.listMyShareRules()) ?? []
        let pushEnabled = await checkPushEnabled()
        let hasVisitedEntraide = UserDefaults.standard.bool(forKey: "entraide.hasVisited")

        // Compute profile progress
        computeProgress(
            user: authManager.currentUser,
            circleCount: circleCount,
            friendCount: friendCount,
            shareRules: shareRules,
            pushEnabled: pushEnabled,
            hasVisitedEntraide: hasVisitedEntraide
        )
    }

    // MARK: - Helpers

    func category(for id: UUID?) -> CategoryResponse? {
        guard let id else { return nil }
        return categories.first { $0.id == id }
    }

    var sanitizedNotifications: [AppNotification] {
        recentNotifications
    }

    // MARK: - Progress Computation

    // swiftlint:disable:next function_parameter_count
    private func computeProgress(
        user: User?,
        circleCount: Int,
        friendCount: Int,
        shareRules: [CircleShareRuleSummary],
        pushEnabled: Bool,
        hasVisitedEntraide: Bool
    ) {
        guard let user else { return }

        profileProgress.update(id: "displayName", completed: user.displayName != nil && !user.displayName!.isEmpty)
        profileProgress.update(id: "username", completed: !user.username.isEmpty && user.username != user.email)
        profileProgress.update(id: "avatar", completed: user.avatarUrl != nil && !user.avatarUrl!.isEmpty)
        profileProgress.update(id: "emailVerified", completed: user.emailVerified ?? false)
        profileProgress.update(id: "firstItem", completed: stats.totalItems > 0)
        profileProgress.update(id: "shareList", completed: shareRules.contains { $0.shareMode != "none" })
        profileProgress.update(id: "firstFriend", completed: friendCount > 0)
        profileProgress.update(id: "firstCircle", completed: circleCount > 0)
        profileProgress.update(id: "pushNotifications", completed: pushEnabled)
        profileProgress.update(id: "firstNeed", completed: hasVisitedEntraide)
    }

    private func checkPushEnabled() async -> Bool {
        let center = UNUserNotificationCenter.current()
        let settings = await center.notificationSettings()
        return settings.authorizationStatus == .authorized
    }
}
