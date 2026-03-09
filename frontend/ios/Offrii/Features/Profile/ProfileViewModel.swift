import Foundation

@Observable
@MainActor
final class ProfileViewModel {
    var displayName = ""
    var username = ""
    var email = ""
    var reminderFreq = "never"
    var reminderTime = "09:00"
    var isLoggingOut = false

    var initials: String {
        let name = displayName.isEmpty ? email : displayName
        let parts = name.components(separatedBy: " ")
        if parts.count >= 2 {
            return String(parts[0].prefix(1) + parts[1].prefix(1)).uppercased()
        }
        return String(name.prefix(2)).uppercased()
    }

    var reminderFreqLabel: String {
        switch reminderFreq {
        case "daily": return NSLocalizedString("reminder.daily", comment: "")
        case "weekly": return NSLocalizedString("reminder.weekly", comment: "")
        case "monthly": return NSLocalizedString("reminder.monthly", comment: "")
        default: return NSLocalizedString("reminder.never", comment: "")
        }
    }

    func loadProfile() async {
        do {
            let profile = try await UserService.shared.getProfile()
            displayName = profile.displayName ?? ""
            username = profile.username
            email = profile.email
            reminderFreq = profile.reminderFreq ?? "never"
            reminderTime = profile.reminderTime ?? "09:00"
        } catch {
            // Profile will show empty state; user can still navigate
        }
    }

    func updateUsername(_ newUsername: String) async throws {
        let profile = try await UserService.shared.updateProfile(username: newUsername)
        username = profile.username
    }
}
