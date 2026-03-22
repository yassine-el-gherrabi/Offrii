import UIKit
@preconcurrency import UserNotifications

final class AppDelegate: NSObject, UIApplicationDelegate {
    /// Shared router reference, set from OffriiApp on launch
    weak var router: AppRouter? {
        didSet { deliverPendingNavigation() }
    }

    /// Pending navigation stored when didReceive fires before router is connected
    private var pendingShowFriends = false
    private var pendingCircleId: UUID?
    private var pendingShowEntraide = false

    func application(
        _ application: UIApplication,
        didFinishLaunchingWithOptions _: [UIApplication.LaunchOptionsKey: Any]? = nil
    ) -> Bool {
        UNUserNotificationCenter.current().delegate = self
        return true
    }

    func application(
        _ application: UIApplication,
        didRegisterForRemoteNotificationsWithDeviceToken deviceToken: Data
    ) {
        let token = deviceToken.map { String(format: "%02x", $0) }.joined()
        Task {
            try? await PushTokenService.shared.registerToken(token: token)
        }
    }

    func application(
        _ application: UIApplication,
        didFailToRegisterForRemoteNotificationsWithError error: Error
    ) {
        print("Push registration failed: \(error.localizedDescription)")
    }

    /// Deliver any navigation that was queued before the router was available
    private func deliverPendingNavigation() {
        guard let router else { return }
        if pendingShowFriends {
            router.showFriends = true
            pendingShowFriends = false
        }
        if let circleId = pendingCircleId {
            router.navigateToCircle(circleId)
            pendingCircleId = nil
        }
        if pendingShowEntraide {
            router.selectedTab = .entraide
            pendingShowEntraide = false
        }
    }
}

// MARK: - Push Notification Handling

extension AppDelegate: @preconcurrency UNUserNotificationCenterDelegate {
    /// Show push notification as banner when app is in foreground
    func userNotificationCenter(
        _: UNUserNotificationCenter,
        willPresent _: UNNotification,
        withCompletionHandler completionHandler: @escaping (UNNotificationPresentationOptions) -> Void
    ) {
        completionHandler([.banner, .sound, .badge])
        // Refresh badge count
        Task { await Self.refreshBadgeCount() }
    }

    /// Update the app icon badge with unread notification count
    @MainActor
    static func refreshBadgeCount() async {
        let count = (try? await NotificationCenterService.shared.unreadCount()) ?? 0
        try? await UNUserNotificationCenter.current().setBadgeCount(count)
    }

    /// Handle tap on push notification — navigate to circle/item
    func userNotificationCenter(
        _: UNUserNotificationCenter,
        didReceive response: UNNotificationResponse,
        withCompletionHandler completionHandler: @escaping () -> Void
    ) {
        let userInfo = response.notification.request.content.userInfo
        let notifType = userInfo["type"] as? String ?? ""

        if notifType.hasPrefix("friend_") {
            if let router {
                Task { @MainActor in router.showFriends = true }
            } else {
                pendingShowFriends = true
            }
        } else if notifType.hasPrefix("wish_") {
            if let router {
                Task { @MainActor in router.selectedTab = .entraide }
            } else {
                pendingShowEntraide = true
            }
        } else if let circleIdString = userInfo["circle_id"] as? String,
                  let circleId = UUID(uuidString: circleIdString) {
            if let router {
                Task { @MainActor in router.navigateToCircle(circleId) }
            } else {
                pendingCircleId = circleId
            }
        }

        // Refresh badge
        Task { await Self.refreshBadgeCount() }

        completionHandler()
    }
}
