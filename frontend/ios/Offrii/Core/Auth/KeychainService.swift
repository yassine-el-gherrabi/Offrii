import Foundation
import KeychainAccess

// MARK: - Keychain Service

/// Secure token storage backed by the system Keychain via KeychainAccess.
///
/// `KeychainAccess.Keychain` is thread-safe internally but not marked `Sendable`.
/// We use `@unchecked Sendable` to bridge this gap.
final class KeychainService: @unchecked Sendable {
    static let shared = KeychainService()

    private let keychain = Keychain(service: "com.offrii.auth")
        .accessibility(.afterFirstUnlockThisDeviceOnly)

    private init() {}

    // MARK: - Access Token

    var accessToken: String? {
        get { try? keychain.get("accessToken") }
        set {
            if let newValue {
                try? keychain.set(newValue, key: "accessToken")
            } else {
                try? keychain.remove("accessToken")
            }
        }
    }

    // MARK: - Refresh Token

    var refreshToken: String? {
        get { try? keychain.get("refreshToken") }
        set {
            if let newValue {
                try? keychain.set(newValue, key: "refreshToken")
            } else {
                try? keychain.remove("refreshToken")
            }
        }
    }

    // MARK: - Clear

    func clearAll() {
        try? keychain.removeAll()
    }
}
