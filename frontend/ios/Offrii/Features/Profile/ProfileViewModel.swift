import Foundation

@Observable
@MainActor
final class ProfileViewModel {
    var displayName = ""
    var username = ""
    var email = ""
    var avatarUrlString: String?
    var isLoggingOut = false
    var loadError: String?
    var createdAt: Date?
    var emailVerified: Bool?

    // Stats
    var totalItems = 0
    var receivedItems = 0
    var circlesCount = 0
    var friendsCount = 0

    // Display name editing
    var isEditingDisplayName = false
    var editedDisplayName = ""
    var isSavingDisplayName = false

    var avatarUrl: URL? {
        guard let str = avatarUrlString else { return nil }
        return URL(string: str)
    }

    var initials: String {
        let name = displayName.isEmpty ? email : displayName
        let parts = name.components(separatedBy: " ")
        if parts.count >= 2 {
            return String(parts[0].prefix(1) + parts[1].prefix(1)).uppercased()
        }
        return String(name.prefix(2)).uppercased()
    }

    var memberSinceText: String {
        guard let date = createdAt else { return "" }
        let formatter = DateFormatter()
        formatter.dateFormat = "MMMM yyyy"
        formatter.locale = Locale.current
        let dateStr = formatter.string(from: date)
        return String(format: NSLocalizedString("profile.memberSince", comment: ""), dateStr)
    }

    var truncatedEmail: String {
        guard email.count > 24 else { return email }
        let parts = email.split(separator: "@")
        guard parts.count == 2 else { return String(email.prefix(24)) + "..." }
        let local = parts[0]
        let domain = parts[1]
        let maxLocal = max(4, 24 - Int(domain.count) - 4) // 4 for @...
        if local.count > maxLocal {
            return String(local.prefix(maxLocal)) + "...@" + domain
        }
        return email
    }

    var appVersion: String {
        Bundle.main.infoDictionary?["CFBundleShortVersionString"] as? String ?? "?"
    }

    func loadProfile() async {
        loadError = nil
        do {
            let profile = try await UserService.shared.getProfile()
            displayName = profile.displayName ?? ""
            username = profile.username
            email = profile.email
            avatarUrlString = profile.avatarUrl
            createdAt = profile.createdAt
            emailVerified = profile.emailVerified
        } catch {
            loadError = NSLocalizedString("error.loadProfileFailed", comment: "")
        }

        // Load stats in parallel
        await loadStats()
    }

    func loadStats() async {
        async let itemsResult = ItemService.shared.listItems(page: 1, perPage: 1)
        async let receivedResult = ItemService.shared.listItems(status: "purchased", page: 1, perPage: 1)
        async let circlesResult = CircleService.shared.listCircles()
        async let friendsResult = FriendService.shared.listFriends()

        if let items = try? await itemsResult {
            totalItems = items.total
        }
        if let received = try? await receivedResult {
            receivedItems = received.total
        }
        if let circles = try? await circlesResult {
            circlesCount = circles.count
        }
        if let friends = try? await friendsResult {
            friendsCount = friends.count
        }
    }

    func updateUsername(_ newUsername: String) async throws {
        let profile = try await UserService.shared.updateProfile(username: newUsername)
        username = profile.username
    }

    func requestEmailChange(_ newEmail: String) async throws {
        try await UserService.shared.requestEmailChange(newEmail: newEmail)
    }

    func updateDisplayName(_ newName: String) async throws {
        let profile = try await UserService.shared.updateProfile(displayName: newName)
        displayName = profile.displayName ?? ""
    }
}
