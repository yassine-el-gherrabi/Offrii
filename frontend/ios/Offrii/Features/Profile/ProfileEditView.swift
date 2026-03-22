import SwiftUI

struct ProfileEditView: View {
    @Bindable var viewModel: ProfileViewModel
    @Environment(\.dismiss) private var dismiss
    @Environment(AuthManager.self) private var authManager
    @State private var editDisplayName = ""
    @State private var editUsername = ""
    @State private var editEmail = ""
    @State private var isSaving = false
    @State private var displayNameError: String?
    @State private var usernameError: String?
    @State private var emailError: String?
    @State private var emailChangeSuccess = false

    private var hasChanges: Bool {
        editDisplayName != viewModel.displayName
            || editUsername != viewModel.username
            || editEmail.lowercased() != viewModel.email.lowercased()
    }

    private var isValid: Bool {
        hasChanges
            && (editUsername == viewModel.username || isValidUsername(editUsername))
            && (editEmail.lowercased() == viewModel.email.lowercased() || isValidEmail(editEmail))
    }

    var body: some View {
        ZStack {
            OffriiTheme.surface.ignoresSafeArea()

            ScrollView {
                VStack(spacing: OffriiTheme.spacingBase) {
                    OffriiCard {
                        VStack(spacing: OffriiTheme.spacingLG) {
                            // Display name
                            OffriiTextField(
                                label: NSLocalizedString("profile.displayName", comment: ""),
                                text: $editDisplayName,
                                placeholder: NSLocalizedString("profile.displayNamePlaceholder", comment: ""),
                                errorMessage: displayNameError
                            )

                            // Username
                            OffriiTextField(
                                label: NSLocalizedString("profile.username", comment: ""),
                                text: $editUsername,
                                placeholder: NSLocalizedString("profile.usernamePlaceholder", comment: ""),
                                errorMessage: usernameError,
                                autocapitalization: .never
                            )

                            // Email
                            VStack(alignment: .leading, spacing: 4) {
                                OffriiTextField(
                                    label: NSLocalizedString("profile.email", comment: ""),
                                    text: $editEmail,
                                    placeholder: NSLocalizedString("profile.emailPlaceholder", comment: ""),
                                    errorMessage: emailError,
                                    keyboardType: .emailAddress,
                                    autocapitalization: .never
                                )

                                if editEmail.lowercased() != viewModel.email.lowercased()
                                    && isValidEmail(editEmail)
                                    && !emailChangeSuccess {
                                    Text(NSLocalizedString("profile.emailChangeHint", comment: ""))
                                        .font(.system(size: 11))
                                        .foregroundColor(OffriiTheme.textMuted)
                                }

                                if emailChangeSuccess {
                                    HStack(spacing: OffriiTheme.spacingXS) {
                                        Image(systemName: "envelope.badge.shield.half.filled")
                                            .font(.system(size: 14))
                                            .foregroundColor(OffriiTheme.primary)
                                        Text(NSLocalizedString("profile.emailChangeSuccess", comment: ""))
                                            .font(.system(size: 12))
                                            .foregroundColor(OffriiTheme.primary)
                                    }
                                }
                            }

                            // Save
                            OffriiButton(
                                NSLocalizedString("common.save", comment: ""),
                                isLoading: isSaving,
                                isDisabled: !isValid
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
        .navigationTitle(NSLocalizedString("profile.editProfile", comment: ""))
        .navigationBarTitleDisplayMode(.inline)
        .onAppear {
            editDisplayName = viewModel.displayName
            editUsername = viewModel.username
            editEmail = viewModel.email
        }
    }

    // MARK: - Save

    private func save() async {
        isSaving = true
        displayNameError = nil
        usernameError = nil
        emailError = nil

        // 1. Update display name + username if changed
        let nameChanged = editDisplayName != viewModel.displayName
        let usernameChanged = editUsername != viewModel.username
        let emailChanged = editEmail.lowercased() != viewModel.email.lowercased()

        if nameChanged || usernameChanged {
            do {
                if nameChanged && usernameChanged {
                    try await viewModel.updateDisplayName(editDisplayName)
                    try await viewModel.updateUsername(editUsername)
                } else if nameChanged {
                    try await viewModel.updateDisplayName(editDisplayName)
                } else {
                    try await viewModel.updateUsername(editUsername)
                }
                try? await authManager.loadCurrentUser()
            } catch let error as APIError {
                if case .conflict = error {
                    usernameError = NSLocalizedString("error.usernameTaken", comment: "")
                } else if case .badRequest = error {
                    usernameError = NSLocalizedString("error.usernameInvalid", comment: "")
                } else {
                    displayNameError = NSLocalizedString("error.serverError", comment: "")
                }
                isSaving = false
                return
            } catch {
                displayNameError = NSLocalizedString("error.serverError", comment: "")
                isSaving = false
                return
            }
        }

        // 2. Request email change if changed
        if emailChanged {
            do {
                try await viewModel.requestEmailChange(editEmail)
                emailChangeSuccess = true
            } catch let error as APIError {
                if case .conflict = error {
                    emailError = NSLocalizedString("error.emailTaken", comment: "")
                } else if case .tooManyRequests = error {
                    emailError = NSLocalizedString("error.tooManyRequests", comment: "")
                } else if case .badRequest = error {
                    emailError = NSLocalizedString("error.emailInvalid", comment: "")
                } else {
                    emailError = NSLocalizedString("error.serverError", comment: "")
                }
                isSaving = false
                return
            } catch {
                emailError = NSLocalizedString("error.serverError", comment: "")
                isSaving = false
                return
            }
        }

        isSaving = false

        // Dismiss only if no email change (email change shows success message)
        if !emailChanged {
            dismiss()
        }
    }

    // MARK: - Validation

    private func isValidUsername(_ input: String) -> Bool {
        guard input.count >= 3, input.count <= 30 else { return false }
        guard let first = input.first, first.isLowercase, first.isASCII else { return false }
        return input.dropFirst().allSatisfy {
            ($0.isLowercase && $0.isASCII) || ($0.isASCII && $0.isNumber) || $0 == "_"
        }
    }

    private func isValidEmail(_ input: String) -> Bool {
        let trimmed = input.trimmingCharacters(in: .whitespaces)
        guard !trimmed.isEmpty else { return false }
        let parts = trimmed.split(separator: "@", maxSplits: 2, omittingEmptySubsequences: false)
        guard parts.count == 2,
              !parts[0].isEmpty,
              parts[1].contains("."),
              !parts[1].hasPrefix("."),
              !parts[1].hasSuffix(".")
        else { return false }
        return true
    }
}
