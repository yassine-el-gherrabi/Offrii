import Foundation

final class NotificationCenterService: Sendable {
    static let shared = NotificationCenterService()
    private let client = APIClient.shared
    private init() {}

    func list(page: Int = 1, limit: Int = 20) async throws -> PaginatedResponse<AppNotification> {
        try await client.request(.listNotifications(page: page, limit: limit))
    }

    func markRead(id: UUID) async throws {
        try await client.requestVoid(.markNotificationRead(id: id))
    }

    func markAllRead() async throws {
        try await client.requestVoid(.markAllNotificationsRead)
    }

    func delete(id: UUID) async throws {
        try await client.requestVoid(.deleteNotification(id: id))
    }

    func unreadCount() async throws -> Int {
        let response: UnreadCountResponse = try await client.request(.unreadNotificationCount)
        return response.count
    }
}
