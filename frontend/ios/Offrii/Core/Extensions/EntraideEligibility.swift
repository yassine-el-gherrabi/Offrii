import Foundation

/// Checks if the current user is eligible for entraide actions
/// (publishing needs, offering help, reporting).
/// Requirements: account > 24h old + email verified.
struct EntraideEligibility {
    let isEligible: Bool
    let isAccountTooRecent: Bool
    let isEmailVerified: Bool

    init(user: User?) {
        guard let user else {
            isAccountTooRecent = true
            isEmailVerified = false
            isEligible = false
            return
        }
        isAccountTooRecent = Date().timeIntervalSince(user.createdAt) < 24 * 3600
        isEmailVerified = user.emailVerified ?? false
        isEligible = !isAccountTooRecent && isEmailVerified
    }
}
