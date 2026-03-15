import Foundation

// MARK: - Auth State

enum AuthState: Equatable {
    case idle
    case loading
    case loadingSSO(SSOProvider)
    case error(String)
}

// MARK: - Forgot Password Step

enum ForgotPasswordStep: Int, CaseIterable {
    case email = 1
    case code = 2
    case newPassword = 3
}

// MARK: - Auth View Model

@Observable
@MainActor
final class AuthViewModel {
    // MARK: - Shared Fields

    var email = ""
    var password = ""
    var state: AuthState = .idle

    // MARK: - Field Errors

    var emailError: String?
    var passwordError: String?

    // MARK: - Forgot Password

    var forgotStep: ForgotPasswordStep = .email
    var resetCode = ""
    var newPassword = ""
    var resetEmail = ""
    var codeError: String?
    var newPasswordError: String?

    var isLoading: Bool { state == .loading }

    func isSSOLoading(_ provider: SSOProvider) -> Bool {
        if case .loadingSSO(let current) = state { return current == provider }
        return false
    }

    var isAnyLoading: Bool {
        switch state {
        case .loading, .loadingSSO: return true
        default: return false
        }
    }

    // MARK: - Validation

    func validateEmail(_ value: String? = nil) -> Bool {
        let email = value ?? self.email
        if email.trimmingCharacters(in: .whitespaces).isEmpty {
            emailError = NSLocalizedString("error.emailRequired", comment: "")
            return false
        }
        let pattern = #"^[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}$"#
        if email.range(of: pattern, options: .regularExpression) == nil {
            emailError = NSLocalizedString("error.invalidEmail", comment: "")
            return false
        }
        emailError = nil
        return true
    }

    func validatePassword(_ value: String? = nil) -> Bool {
        let password = value ?? self.password
        if password.isEmpty {
            passwordError = NSLocalizedString("error.passwordRequired", comment: "")
            return false
        }
        if password.count < 8 {
            passwordError = NSLocalizedString("error.passwordTooShort", comment: "")
            return false
        }
        passwordError = nil
        return true
    }

    func validateNewPassword() -> Bool {
        if newPassword.isEmpty {
            newPasswordError = NSLocalizedString("error.passwordRequired", comment: "")
            return false
        }
        if newPassword.count < 8 {
            newPasswordError = NSLocalizedString("error.passwordTooShort", comment: "")
            return false
        }
        newPasswordError = nil
        return true
    }

    // MARK: - Login

    func login(authManager: AuthManager) async -> Bool {
        clearErrors()
        let emailValid = validateEmail()
        let passwordValid = validatePassword()
        guard emailValid && passwordValid else { return false }

        state = .loading
        do {
            try await authManager.login(email: email.trimmingCharacters(in: .whitespaces), password: password)
            state = .idle
            return true
        } catch {
            state = .error(mapError(error))
            return false
        }
    }

    // MARK: - Google Sign-In

    /// Returns `nil` if cancelled/failed, otherwise `true` = new user, `false` = existing user.
    func signInWithGoogle(authManager: AuthManager, ssoService: SSOService) async -> Bool? {
        state = .loadingSSO(.google)
        do {
            let idToken = try await ssoService.signInWithGoogle()
            let isNew = try await authManager.loginWithGoogle(idToken: idToken)
            state = .idle
            return isNew
        } catch let error as SSOError where error == .cancelled {
            state = .idle
            return nil
        } catch {
            state = .error(mapError(error))
            return nil
        }
    }

    // MARK: - Apple Sign-In

    /// Returns `nil` if cancelled/failed, otherwise `true` = new user, `false` = existing user.
    func signInWithApple(authManager: AuthManager, ssoService: SSOService) async -> Bool? {
        state = .loadingSSO(.apple)
        do {
            let (idToken, fullName) = try await ssoService.signInWithApple()
            let displayName = [fullName?.givenName, fullName?.familyName]
                .compactMap { $0 }
                .joined(separator: " ")
            let isNew = try await authManager.loginWithApple(
                idToken: idToken,
                displayName: displayName.isEmpty ? nil : displayName
            )
            state = .idle
            return isNew
        } catch let error as SSOError where error == .cancelled {
            state = .idle
            return nil
        } catch {
            state = .error(mapError(error))
            return nil
        }
    }

    // MARK: - Register

    func register(authManager: AuthManager) async -> Bool {
        clearErrors()
        let emailValid = validateEmail()
        let passwordValid = validatePassword()
        guard emailValid && passwordValid else { return false }

        state = .loading
        do {
            try await authManager.register(
                email: email.trimmingCharacters(in: .whitespaces),
                password: password,
                displayName: nil
            )
            state = .idle
            return true
        } catch {
            state = .error(mapError(error))
            return false
        }
    }

    // MARK: - Forgot Password Flow

    func sendResetCode() async -> Bool {
        clearErrors()
        guard validateEmail(resetEmail) else { return false }

        state = .loading
        do {
            try await APIClient.shared.requestVoid(
                .forgotPassword(ForgotPasswordBody(email: resetEmail.trimmingCharacters(in: .whitespaces)))
            )
            state = .idle
            forgotStep = .code
            return true
        } catch {
            state = .error(mapError(error))
            return false
        }
    }

    func verifyResetCode() async -> Bool {
        clearErrors()
        let code = resetCode.trimmingCharacters(in: .whitespaces)
        if code.count != 6 {
            codeError = NSLocalizedString("error.invalidCode", comment: "")
            return false
        }

        state = .loading
        do {
            try await APIClient.shared.requestVoid(
                .verifyResetCode(VerifyResetCodeBody(
                    email: resetEmail.trimmingCharacters(in: .whitespaces),
                    code: code
                ))
            )
            state = .idle
            return true
        } catch {
            state = .error(mapError(error))
            return false
        }
    }

    func verifyCodeAndReset() async -> Bool {
        clearErrors()
        guard validateNewPassword() else { return false }

        if resetCode.trimmingCharacters(in: .whitespaces).count != 6 {
            codeError = NSLocalizedString("error.invalidCode", comment: "")
            return false
        }

        state = .loading
        do {
            try await APIClient.shared.requestVoid(
                .resetPassword(ResetPasswordBody(
                    email: resetEmail.trimmingCharacters(in: .whitespaces),
                    code: resetCode.trimmingCharacters(in: .whitespaces),
                    newPassword: newPassword
                ))
            )
            state = .idle
            return true
        } catch {
            state = .error(mapError(error))
            return false
        }
    }

    func resetForgotState() {
        forgotStep = .email
        resetEmail = ""
        resetCode = ""
        newPassword = ""
        codeError = nil
        newPasswordError = nil
        emailError = nil
        state = .idle
    }

    // MARK: - Helpers

    private func clearErrors() {
        emailError = nil
        passwordError = nil
        codeError = nil
        newPasswordError = nil
        if case .error = state { state = .idle }
    }

    private func mapError(_ error: Error) -> String {
        guard let apiError = error as? APIError else {
            return NSLocalizedString("error.networkError", comment: "")
        }
        switch apiError {
        case .badRequest(let msg):
            return mapBadRequest(msg)
        case .conflict(let msg):
            return mapConflict(msg)
        case .unauthorized(let msg):
            let lower = msg.lowercased()
            if lower.contains("credentials") || lower.contains("invalid") {
                return NSLocalizedString("error.invalidCredentials", comment: "")
            }
            return msg
        case .unknown(429, _):
            return NSLocalizedString("error.rateLimited", comment: "")
        case .serverError:
            return NSLocalizedString("error.serverError", comment: "")
        case .networkError:
            return NSLocalizedString("error.networkError", comment: "")
        default:
            return apiError.localizedDescription
        }
    }

    private func mapBadRequest(_ msg: String) -> String {
        let lower = msg.lowercased()
        let mapping: [(String, String)] = [
            ("password_common", "error.passwordCommon"),
            ("password_breached", "error.passwordBreached"),
            ("password_length", "error.passwordTooShort"),
            ("invalid_or_expired_code", "error.invalidOrExpiredCode"),
            ("too_many_attempts", "error.tooManyAttempts"),
        ]
        if lower.contains("email") && lower.contains("taken") {
            return NSLocalizedString("error.emailTaken", comment: "")
        }
        for (keyword, key) in mapping where lower.contains(keyword) {
            return NSLocalizedString(key, comment: "")
        }
        return msg
    }

    private func mapConflict(_ msg: String) -> String {
        let lower = msg.lowercased()
        if lower.contains("email_uses_oauth:google") {
            return NSLocalizedString("error.emailUsesGoogle", comment: "")
        }
        if lower.contains("email_uses_oauth:apple") {
            return NSLocalizedString("error.emailUsesApple", comment: "")
        }
        if lower.contains("email") {
            return NSLocalizedString("error.emailTaken", comment: "")
        }
        return msg
    }
}
