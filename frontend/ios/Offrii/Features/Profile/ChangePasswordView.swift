import SwiftUI

struct ChangePasswordView: View {
    @Environment(\.dismiss) private var dismiss
    @Environment(AuthManager.self) private var authManager
    @State private var currentPassword = ""
    @State private var newPassword = ""
    @State private var confirmPassword = ""
    @State private var isSaving = false
    @State private var errorMessage: String?
    @State private var showSuccess = false

    private var isValid: Bool {
        !currentPassword.isEmpty
            && newPassword.count >= 8
            && newPassword == confirmPassword
    }

    private var confirmError: String? {
        if !confirmPassword.isEmpty && newPassword != confirmPassword {
            return NSLocalizedString("error.passwordMismatch", comment: "")
        }
        return nil
    }

    var body: some View {
        ZStack {
            OffriiTheme.surface.ignoresSafeArea()

            ScrollView {
                VStack(spacing: OffriiTheme.spacingBase) {
                    OffriiCard {
                        VStack(spacing: OffriiTheme.spacingLG) {
                            if showSuccess {
                                HStack(spacing: OffriiTheme.spacingSM) {
                                    Image(systemName: "checkmark.circle.fill")
                                        .font(.system(size: 20))
                                        .foregroundColor(OffriiTheme.primary)
                                    Text(NSLocalizedString("profile.passwordChanged", comment: ""))
                                        .font(OffriiTypography.body)
                                        .foregroundColor(OffriiTheme.text)
                                }
                                .padding(OffriiTheme.spacingBase)
                                .frame(maxWidth: .infinity, alignment: .leading)
                                .background(OffriiTheme.primary.opacity(0.08))
                                .cornerRadius(OffriiTheme.cornerRadiusMD)
                            }

                            OffriiTextField(
                                label: NSLocalizedString("profile.currentPassword", comment: ""),
                                text: $currentPassword,
                                placeholder: "",
                                errorMessage: errorMessage,
                                isSecure: true,
                                textContentType: .password
                            )

                            OffriiTextField(
                                label: NSLocalizedString("profile.newPassword", comment: ""),
                                text: $newPassword,
                                placeholder: "",
                                isSecure: true,
                                textContentType: .newPassword
                            )

                            VStack(alignment: .leading, spacing: 4) {
                                OffriiTextField(
                                    label: NSLocalizedString("profile.confirmPassword", comment: ""),
                                    text: $confirmPassword,
                                    placeholder: "",
                                    errorMessage: confirmError,
                                    isSecure: true,
                                    textContentType: .newPassword
                                )

                                if newPassword.isEmpty || newPassword.count >= 8 {
                                    // No hint needed
                                } else {
                                    Text(NSLocalizedString("auth.passwordHint", comment: ""))
                                        .font(.system(size: 11))
                                        .foregroundColor(OffriiTheme.textMuted)
                                }
                            }

                            OffriiButton(
                                NSLocalizedString("common.save", comment: ""),
                                isLoading: isSaving,
                                isDisabled: !isValid || showSuccess
                            ) {
                                Task { await save() }
                            }
                        }
                    }
                    .padding(.horizontal, OffriiTheme.spacingLG)
                }
                .padding(.top, OffriiTheme.spacingBase)
            }
        }
        .navigationTitle(NSLocalizedString("profile.changePassword", comment: ""))
        .navigationBarTitleDisplayMode(.inline)
    }

    private func save() async {
        isSaving = true
        errorMessage = nil
        do {
            try await authManager.changePassword(
                currentPassword: currentPassword,
                newPassword: newPassword
            )
            showSuccess = true
            currentPassword = ""
            newPassword = ""
            confirmPassword = ""
        } catch let error as APIError {
            if case .badRequest(let msg) = error {
                if msg.contains("wrong_current_password") {
                    errorMessage = NSLocalizedString("error.wrongPassword", comment: "")
                } else if msg.contains("common") {
                    errorMessage = NSLocalizedString("error.passwordCommon", comment: "")
                } else if msg.contains("breach") {
                    errorMessage = NSLocalizedString("error.passwordBreached", comment: "")
                } else {
                    errorMessage = NSLocalizedString("error.passwordTooShort", comment: "")
                }
            } else {
                errorMessage = NSLocalizedString("error.serverError", comment: "")
            }
        } catch {
            errorMessage = NSLocalizedString("error.serverError", comment: "")
        }
        isSaving = false
    }
}
