import SwiftUI

// MARK: - Onboarding Tip Manager

@Observable
final class OnboardingTipManager {

    // MARK: - Tip IDs

    enum TipID: String, CaseIterable {
        case wishlistFirstAdd = "tip_wishlist_first_add"
        case wishlistSwipe = "tip_wishlist_swipe"
        case circlesCreate = "tip_circles_create"
        case circlesShare = "tip_circles_share"
        case entraideBrowse = "tip_entraide_browse"
        case entraideOffer = "tip_entraide_offer"
        case profileUsername = "tip_profile_username"
        case profileReminders = "tip_profile_reminders"
    }

    // MARK: - Tip Messages

    static func message(for tip: TipID) -> String {
        switch tip {
        case .wishlistFirstAdd:
            return NSLocalizedString("tip.wishlist.firstAdd", comment: "")
        case .wishlistSwipe:
            return NSLocalizedString("tip.wishlist.swipe", comment: "")
        case .circlesCreate:
            return NSLocalizedString("tip.circles.create", comment: "")
        case .circlesShare:
            return NSLocalizedString("tip.circles.share", comment: "")
        case .entraideBrowse:
            return NSLocalizedString("tip.entraide.browse", comment: "")
        case .entraideOffer:
            return NSLocalizedString("tip.entraide.offer", comment: "")
        case .profileUsername:
            return NSLocalizedString("tip.profile.username", comment: "")
        case .profileReminders:
            return NSLocalizedString("tip.profile.reminders", comment: "")
        }
    }

    // MARK: - State

    private(set) var activeTip: TipID?

    // MARK: - Check / Show / Dismiss

    func hasBeenSeen(_ tip: TipID) -> Bool {
        UserDefaults.standard.bool(forKey: tip.rawValue)
    }

    func showIfNeeded(_ tip: TipID) {
        guard !hasBeenSeen(tip), activeTip == nil else { return }
        activeTip = tip
    }

    func dismiss(_ tip: TipID) {
        UserDefaults.standard.set(true, forKey: tip.rawValue)
        if activeTip == tip {
            activeTip = nil
        }
    }

    func dismissCurrent() {
        guard let current = activeTip else { return }
        dismiss(current)
    }

    // MARK: - Reset All Tips

    func resetAllTips() {
        for tip in TipID.allCases {
            UserDefaults.standard.removeObject(forKey: tip.rawValue)
        }
        activeTip = nil
    }
}
