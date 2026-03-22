import SwiftUI

struct UsernameEditView: View {
    @Bindable var viewModel: ProfileViewModel
    @Environment(\.dismiss) private var dismiss
    @Environment(AuthManager.self) private var authManager
    @State private var newUsername = ""
    @State private var isSaving = false
    @State private var errorMessage: String?

    private var isValid: Bool {
        isValidUsername(newUsername)
    }

    private var validationMessage: String? {
        if newUsername.isEmpty { return nil }
        if !isValid {
            return NSLocalizedString("error.usernameInvalid", comment: "")
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
                            OffriiTextField(
                                label: NSLocalizedString("profile.username", comment: ""),
                                text: $newUsername,
                                placeholder: NSLocalizedString("profile.usernamePlaceholder", comment: ""),
                                errorMessage: validationMessage ?? errorMessage,
                                autocapitalization: .never
                            )

                            OffriiButton(
                                NSLocalizedString("common.save", comment: ""),
                                isLoading: isSaving,
                                isDisabled: !isValid || newUsername == viewModel.username
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
        .navigationTitle(NSLocalizedString("profile.editUsername", comment: ""))
        .navigationBarTitleDisplayMode(.inline)
        .onAppear {
            newUsername = viewModel.username
        }
    }

    private func save() async {
        isSaving = true
        errorMessage = nil
        do {
            try await viewModel.updateUsername(newUsername)
            try? await authManager.loadCurrentUser()
            dismiss()
        } catch let error as APIError {
            if case .conflict = error {
                errorMessage = NSLocalizedString("error.usernameTaken", comment: "")
            } else if case .badRequest = error {
                errorMessage = NSLocalizedString("error.usernameInvalid", comment: "")
            } else {
                errorMessage = NSLocalizedString("error.serverError", comment: "")
            }
        } catch {
            errorMessage = NSLocalizedString("error.serverError", comment: "")
        }
        isSaving = false
    }

    private func isValidUsername(_ input: String) -> Bool {
        guard input.count >= 3, input.count <= 30 else { return false }
        guard let first = input.first, first.isLowercase, first.isASCII else { return false }
        return input.dropFirst().allSatisfy { ($0.isLowercase && $0.isASCII) || ($0.isASCII && $0.isNumber) || $0 == "_" }
    }
}
