import Foundation

// MARK: - Quick Start Action

enum QuickStartAction: String, CaseIterable {
    case addWish
    case createCircle
    case inviteFriend
    case shareList
}

// MARK: - Profile Progress

struct ProfileProgress {
    var hasUsername: Bool = false
    var hasDisplayName: Bool = false
    var hasFirstItem: Bool = false
    var hasFirstCircle: Bool = false
    var hasFirstFriend: Bool = false
    var hasSharedList: Bool = false
    var hasReminders: Bool = false

    var percentage: Int {
        let actions: [Bool] = [
            hasUsername, hasDisplayName, hasFirstItem,
            hasFirstCircle, hasFirstFriend, hasSharedList, hasReminders
        ]
        let done = actions.filter { $0 }.count
        return Int(Double(done) / Double(actions.count) * 100)
    }

    var completedCount: Int {
        let actions: [Bool] = [
            hasUsername, hasDisplayName, hasFirstItem,
            hasFirstCircle, hasFirstFriend, hasSharedList, hasReminders
        ]
        return actions.filter { $0 }.count
    }

    var totalCount: Int { 7 }

    var nextAction: String? {
        if !hasFirstItem { return NSLocalizedString("home.quickStart.addWish", comment: "") }
        if !hasUsername { return NSLocalizedString("profile.editUsername", comment: "") }
        if !hasFirstCircle { return NSLocalizedString("home.quickStart.createCircle", comment: "") }
        if !hasFirstFriend { return NSLocalizedString("home.quickStart.inviteFriend", comment: "") }
        if !hasDisplayName { return NSLocalizedString("postauth.displayName.title", comment: "") }
        if !hasReminders { return NSLocalizedString("profile.reminders", comment: "") }
        if !hasSharedList { return NSLocalizedString("home.quickStart.shareList", comment: "") }
        return nil
    }
}
