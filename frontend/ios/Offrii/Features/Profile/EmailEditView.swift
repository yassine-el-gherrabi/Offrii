import SwiftUI

struct EmailEditView: View {
    @Bindable var viewModel: ProfileViewModel
    @Environment(\.dismiss) private var dismiss
    @State private var newEmail = ""
    @State private var isSaving = false
    @State private var errorMessage: String?
    @State private var showSuccess = false

    private var isValid: Bool {
        isValidEmail(newEmail)
    }

    private var validationMessage: String? {
        if newEmail.isEmpty { return nil }
        if !isValid {
            return NSLocalizedString("error.emailInvalid", comment: "")
        }
        return nil
    }

    var body: some View {
        ZStack {
            OffriiTheme.surface.ignoresSafeArea()

            ScrollView {
                VStack(spacing: OffriiTheme.spacingBase) {
                    OffriiCard {
                        VStack(spacing: OffriiTheme.spacingBase) {
                            if showSuccess {
                                HStack(spacing: OffriiTheme.spacingSM) {
                                    Image(systemName: "envelope.badge.shield.half.filled")
                                        .font(.system(size: 20))
                                        .foregroundColor(OffriiTheme.primary)

                                    Text(NSLocalizedString("profile.emailChangeSuccess", comment: ""))
                                        .font(OffriiTypography.body)
                                        .foregroundColor(OffriiTheme.text)
                                }
                                .padding(OffriiTheme.spacingBase)
                                .frame(maxWidth: .infinity, alignment: .leading)
                                .background(OffriiTheme.primary.opacity(0.08))
                                .cornerRadius(OffriiTheme.cornerRadiusMD)
                            }

                            OffriiTextField(
                                label: NSLocalizedString("profile.email", comment: ""),
                                text: $newEmail,
                                placeholder: NSLocalizedString("profile.emailPlaceholder", comment: ""),
                                errorMessage: validationMessage ?? errorMessage,
                                keyboardType: .emailAddress,
                                autocapitalization: .never
                            )

                            OffriiButton(
                                NSLocalizedString("common.save", comment: ""),
                                isLoading: isSaving,
                                isDisabled: !isValid || newEmail.lowercased() == viewModel.email.lowercased() || showSuccess
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
        .navigationTitle(NSLocalizedString("profile.editEmail", comment: ""))
        .navigationBarTitleDisplayMode(.inline)
        .onAppear {
            newEmail = viewModel.email
        }
    }

    private func save() async {
        isSaving = true
        errorMessage = nil
        do {
            try await viewModel.requestEmailChange(newEmail)
            showSuccess = true
        } catch let error as APIError {
            if case .conflict = error {
                errorMessage = NSLocalizedString("error.emailTaken", comment: "")
            } else if case .tooManyRequests = error {
                errorMessage = NSLocalizedString("error.tooManyRequests", comment: "")
            } else if case .badRequest = error {
                errorMessage = NSLocalizedString("error.emailInvalid", comment: "")
            } else {
                errorMessage = NSLocalizedString("error.serverError", comment: "")
            }
        } catch {
            errorMessage = NSLocalizedString("error.serverError", comment: "")
        }
        isSaving = false
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
