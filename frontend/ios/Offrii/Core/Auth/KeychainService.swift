import Foundation

// MARK: - Keychain Service

/// Secure token storage.
///
/// Currently backed by `UserDefaults` for development convenience.
/// Swap the getter/setter bodies to a real Keychain wrapper (e.g. KeychainAccess)
/// before shipping to production.
final class KeychainService: Sendable {
    static let shared = KeychainService()

    private let accessTokenKey = "com.offrii.accessToken"
    private let refreshTokenKey = "com.offrii.refreshToken"

    private init() {}

    // MARK: - Access Token

    var accessToken: String? {
        get { UserDefaults.standard.string(forKey: accessTokenKey) }
        set {
            if let newValue {
                UserDefaults.standard.set(newValue, forKey: accessTokenKey)
            } else {
                UserDefaults.standard.removeObject(forKey: accessTokenKey)
            }
        }
    }

    // MARK: - Refresh Token

    var refreshToken: String? {
        get { UserDefaults.standard.string(forKey: refreshTokenKey) }
        set {
            if let newValue {
                UserDefaults.standard.set(newValue, forKey: refreshTokenKey)
            } else {
                UserDefaults.standard.removeObject(forKey: refreshTokenKey)
            }
        }
    }

    // MARK: - Clear

    func clearAll() {
        accessToken = nil
        refreshToken = nil
    }
}
