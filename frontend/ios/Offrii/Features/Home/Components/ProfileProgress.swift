import Foundation

// MARK: - Profile Progress Step

struct ProfileProgressStep: Identifiable {
    let id: String
    let group: StepGroup
    let icon: String
    let titleKey: String
    let subtitleKey: String
    var isCompleted: Bool

    enum StepGroup: String, CaseIterable {
        case identity
        case wishlist
        case social
        case settings

        var titleKey: String {
            switch self {
            case .identity: return "progress.group.identity"
            case .wishlist: return "progress.group.wishlist"
            case .social: return "progress.group.social"
            case .settings: return "progress.group.settings"
            }
        }

        var icon: String {
            switch self {
            case .identity: return "person.fill"
            case .wishlist: return "heart.fill"
            case .social: return "person.2.fill"
            case .settings: return "gearshape.fill"
            }
        }
    }
}

// MARK: - Profile Progress

struct ProfileProgress {
    var steps: [ProfileProgressStep] = ProfileProgress.defaultSteps()

    var percentage: Int {
        guard !steps.isEmpty else { return 0 }
        let done = steps.filter(\.isCompleted).count
        return Int(Double(done) / Double(steps.count) * 100)
    }

    var completedCount: Int {
        steps.filter(\.isCompleted).count
    }

    var totalCount: Int { steps.count }

    var nextIncompleteStep: ProfileProgressStep? {
        steps.first { !$0.isCompleted }
    }

    var nextAction: String? {
        nextIncompleteStep.map { NSLocalizedString($0.titleKey, comment: "") }
    }

    // MARK: - Step Definitions

    static func defaultSteps() -> [ProfileProgressStep] {
        [
            ProfileProgressStep(
                id: "displayName", group: .identity,
                icon: "textformat", titleKey: "progress.step.displayName",
                subtitleKey: "progress.step.displayName.sub", isCompleted: false
            ),
            ProfileProgressStep(
                id: "username", group: .identity,
                icon: "at", titleKey: "progress.step.username",
                subtitleKey: "progress.step.username.sub", isCompleted: false
            ),
            ProfileProgressStep(
                id: "avatar", group: .identity,
                icon: "camera.fill", titleKey: "progress.step.avatar",
                subtitleKey: "progress.step.avatar.sub", isCompleted: false
            ),
            ProfileProgressStep(
                id: "emailVerified", group: .identity,
                icon: "envelope.badge.fill", titleKey: "progress.step.emailVerified",
                subtitleKey: "progress.step.emailVerified.sub", isCompleted: false
            ),
            ProfileProgressStep(
                id: "firstItem", group: .wishlist,
                icon: "gift.fill", titleKey: "progress.step.firstItem",
                subtitleKey: "progress.step.firstItem.sub", isCompleted: false
            ),
            ProfileProgressStep(
                id: "shareList", group: .wishlist,
                icon: "square.and.arrow.up", titleKey: "progress.step.shareList",
                subtitleKey: "progress.step.shareList.sub", isCompleted: false
            ),
            ProfileProgressStep(
                id: "firstFriend", group: .social,
                icon: "person.badge.plus", titleKey: "progress.step.firstFriend",
                subtitleKey: "progress.step.firstFriend.sub", isCompleted: false
            ),
            ProfileProgressStep(
                id: "firstCircle", group: .social,
                icon: "person.2.fill", titleKey: "progress.step.firstCircle",
                subtitleKey: "progress.step.firstCircle.sub", isCompleted: false
            ),
            ProfileProgressStep(
                id: "pushNotifications", group: .settings,
                icon: "app.badge", titleKey: "progress.step.pushNotifications",
                subtitleKey: "progress.step.pushNotifications.sub", isCompleted: false
            ),
            ProfileProgressStep(
                id: "firstNeed", group: .social,
                icon: "hand.raised.fill", titleKey: "progress.step.firstNeed",
                subtitleKey: "progress.step.firstNeed.sub", isCompleted: false
            ),
        ]
    }

    // MARK: - Update Steps

    mutating func update(id: String, completed: Bool) {
        if let idx = steps.firstIndex(where: { $0.id == id }) {
            steps[idx].isCompleted = completed
        }
    }

    // Legacy compatibility
    var hasUsername: Bool { steps.first { $0.id == "username" }?.isCompleted ?? false }
    var hasDisplayName: Bool { steps.first { $0.id == "displayName" }?.isCompleted ?? false }
    var hasFirstItem: Bool { steps.first { $0.id == "firstItem" }?.isCompleted ?? false }
    var hasFirstCircle: Bool { steps.first { $0.id == "firstCircle" }?.isCompleted ?? false }
    var hasFirstFriend: Bool { steps.first { $0.id == "firstFriend" }?.isCompleted ?? false }
    var hasSharedList: Bool { steps.first { $0.id == "shareList" }?.isCompleted ?? false }
    var hasReminders: Bool { steps.first { $0.id == "reminders" }?.isCompleted ?? false }
}

// MARK: - Quick Start Action (legacy, kept for compatibility)

enum QuickStartAction: String, CaseIterable {
    case addWish
    case createCircle
    case inviteFriend
    case shareList
}
