import SwiftUI

struct ForgotPasswordView: View {
    @Environment(AuthManager.self) private var authManager
    @State private var viewModel = AuthViewModel()

    let onDone: () -> Void

    @Environment(\.dismiss) private var dismiss

    var body: some View {
        NavigationStack {
            ZStack {
                OffriiTheme.cardSurface.ignoresSafeArea()

                ScrollView {
                    VStack(spacing: 0) {
                        HeaderView(
                            title: NSLocalizedString("auth.resetPassword", comment: ""),
                            subtitle: stepSubtitle
                        )

                        OffriiCard {
                            VStack(spacing: OffriiTheme.spacingMD) {
                                // Step indicator
                                Text(String(
                                    format: NSLocalizedString("auth.step", comment: ""),
                                    viewModel.forgotStep.rawValue,
                                    ForgotPasswordStep.allCases.count
                                ))
                                .font(OffriiTypography.caption)
                                .foregroundColor(OffriiTheme.textMuted)
                                .frame(maxWidth: .infinity, alignment: .leading)

                                stepContent

                                if case .error(let message) = viewModel.state {
                                    Text(message)
                                        .font(OffriiTypography.caption)
                                        .foregroundColor(OffriiTheme.danger)
                                        .frame(maxWidth: .infinity, alignment: .leading)
                                }

                                stepButton
                            }
                        }
                        .padding(.horizontal, OffriiTheme.spacingLG)
                        .padding(.top, -OffriiTheme.spacingLG)
                    }
                }
                .scrollDismissesKeyboard(.interactively)
            }
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button(NSLocalizedString("common.cancel", comment: "")) {
                        viewModel.resetForgotState()
                        dismiss()
                    }
                    .foregroundColor(OffriiTheme.primary)
                }
            }
        }
    }

    // MARK: - Step Subtitle

    private var stepSubtitle: String {
        switch viewModel.forgotStep {
        case .email:
            return NSLocalizedString("auth.forgotSubtitle", comment: "")
        case .code:
            return NSLocalizedString("auth.codeSent", comment: "")
        case .newPassword:
            return NSLocalizedString("auth.newPasswordSubtitle", comment: "")
        }
    }

    // MARK: - Step Content

    @ViewBuilder
    private var stepContent: some View {
        switch viewModel.forgotStep {
        case .email:
            OffriiTextField(
                label: NSLocalizedString("auth.email", comment: ""),
                text: $viewModel.resetEmail,
                placeholder: NSLocalizedString("auth.enterEmail", comment: ""),
                errorMessage: viewModel.emailError,
                keyboardType: .emailAddress,
                textContentType: .emailAddress,
                autocapitalization: .never
            )

        case .code:
            VStack(spacing: OffriiTheme.spacingSM) {
                OffriiTextField(
                    label: NSLocalizedString("auth.enterCode", comment: ""),
                    text: $viewModel.resetCode,
                    placeholder: "000000",
                    errorMessage: viewModel.codeError,
                    keyboardType: .numberPad
                )

                Text(NSLocalizedString("auth.codeSentTo", comment: "") + " " + viewModel.resetEmail)
                    .font(OffriiTypography.caption)
                    .foregroundColor(OffriiTheme.textMuted)
                    .frame(maxWidth: .infinity, alignment: .leading)
            }

        case .newPassword:
            OffriiTextField(
                label: NSLocalizedString("auth.newPassword", comment: ""),
                text: $viewModel.newPassword,
                placeholder: NSLocalizedString("auth.passwordPlaceholder", comment: ""),
                errorMessage: viewModel.newPasswordError,
                isSecure: true,
                textContentType: .newPassword
            )
        }
    }

    // MARK: - Step Button

    @ViewBuilder
    private var stepButton: some View {
        switch viewModel.forgotStep {
        case .email:
            OffriiButton(
                NSLocalizedString("onboarding.continue", comment: ""),
                isLoading: viewModel.isLoading
            ) {
                Task {
                    await viewModel.sendResetCode()
                }
            }

        case .code:
            OffriiButton(
                NSLocalizedString("onboarding.continue", comment: ""),
                isLoading: viewModel.isLoading
            ) {
                Task {
                    viewModel.forgotStep = .newPassword
                }
            }

        case .newPassword:
            OffriiButton(
                NSLocalizedString("auth.resetPassword", comment: ""),
                isLoading: viewModel.isLoading
            ) {
                Task {
                    if await viewModel.verifyCodeAndReset() {
                        onDone()
                    }
                }
            }
        }
    }
}
