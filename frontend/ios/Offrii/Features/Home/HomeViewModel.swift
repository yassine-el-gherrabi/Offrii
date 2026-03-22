import SwiftUI

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

        // Profile progress (centralized computation)
        profileProgress = await ProfileProgress.compute(
            user: authManager.currentUser,
            totalItems: stats.totalItems
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

}
