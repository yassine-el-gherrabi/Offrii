import AuthenticationServices
import GoogleSignIn

/// Handles native Google and Apple Sign-In flows, returning ID tokens
/// that the backend can verify via JWKS.
@MainActor
final class SSOService: NSObject {

    // MARK: - Google Sign-In

    /// Triggers the Google Sign-In flow and returns the ID token string.
    func signInWithGoogle() async throws -> String {
        guard Bundle.main.object(forInfoDictionaryKey: "GIDClientID") != nil else {
            throw SSOError.notConfigured("Google Sign-In is not configured (missing GIDClientID in Info.plist)")
        }

        guard let windowScene = UIApplication.shared.connectedScenes
            .compactMap({ $0 as? UIWindowScene }).first,
              let rootVC = windowScene.windows.first(where: \.isKeyWindow)?.rootViewController else {
            throw SSOError.noPresentingViewController
        }

        let result = try await GIDSignIn.sharedInstance.signIn(withPresenting: rootVC)

        guard let idToken = result.user.idToken?.tokenString else {
            throw SSOError.missingIDToken
        }

        return idToken
    }

    // MARK: - Apple Sign-In

    /// Triggers the Apple Sign-In flow and returns the identity token + optional full name.
    func signInWithApple() async throws -> (idToken: String, fullName: PersonNameComponents?) {
        try await withCheckedThrowingContinuation { continuation in
            let provider = ASAuthorizationAppleIDProvider()
            let request = provider.createRequest()
            request.requestedScopes = [.email, .fullName]

            let delegate = AppleSignInDelegate(continuation: continuation)
            // Prevent delegate from being deallocated during the flow
            objc_setAssociatedObject(
                request, &AppleSignInDelegate.associatedKey, delegate,
                .OBJC_ASSOCIATION_RETAIN_NONATOMIC
            )

            let controller = ASAuthorizationController(authorizationRequests: [request])
            controller.delegate = delegate
            controller.performRequests()
        }
    }
}

// MARK: - Apple Sign-In Delegate

private final class AppleSignInDelegate: NSObject, ASAuthorizationControllerDelegate {
    static var associatedKey: UInt8 = 0

    private let continuation: CheckedContinuation<(idToken: String, fullName: PersonNameComponents?), Error>
    private var hasResumed = false

    init(continuation: CheckedContinuation<(idToken: String, fullName: PersonNameComponents?), Error>) {
        self.continuation = continuation
    }

    func authorizationController(controller: ASAuthorizationController,
                                 didCompleteWithAuthorization authorization: ASAuthorization) {
        guard !hasResumed else { return }
        hasResumed = true

        guard let credential = authorization.credential as? ASAuthorizationAppleIDCredential,
              let tokenData = credential.identityToken,
              let idToken = String(data: tokenData, encoding: .utf8) else {
            continuation.resume(throwing: SSOError.missingIDToken)
            return
        }

        continuation.resume(returning: (idToken: idToken, fullName: credential.fullName))
    }

    func authorizationController(controller: ASAuthorizationController,
                                 didCompleteWithError error: Error) {
        guard !hasResumed else { return }
        hasResumed = true

        if let asError = error as? ASAuthorizationError, asError.code == .canceled {
            continuation.resume(throwing: SSOError.cancelled)
        } else {
            continuation.resume(throwing: error)
        }
    }
}

// MARK: - SSO Errors

enum SSOError: LocalizedError, Equatable {
    case noPresentingViewController
    case missingIDToken
    case cancelled
    case notConfigured(String)

    var errorDescription: String? {
        switch self {
        case .noPresentingViewController:
            return "Unable to find a presenting view controller"
        case .missingIDToken:
            return "Sign-in succeeded but no ID token was returned"
        case .cancelled:
            return NSLocalizedString("error.ssoUserCancelled", comment: "User cancelled sign-in")
        case .notConfigured(let msg):
            return msg
        }
    }
}
