import SwiftUI

struct RegisterView: View {
    @Environment(AuthManager.self) private var authManager
    @State private var viewModel = AuthViewModel()

    let onAuthenticated: () -> Void
    let onSwitchToLogin: () -> Void

    @FocusState private var focusedField: RegisterField?

    private enum RegisterField: Hashable {
        case displayName, email, password, confirmPassword
    }

    var body: some View {
        ZStack {
            OffriiTheme.background.ignoresSafeArea()
            BlobBackground(preset: .auth)
                .ignoresSafeArea()

            GeometryReader { geometry in
                ScrollView(showsIndicators: false) {
                    VStack(spacing: OffriiTheme.spacingLG) {
                        logoSection
                            .padding(.top, 60)

                        Spacer(minLength: 0)

                        cardSection
                            .padding(.bottom, OffriiTheme.spacingBase)
                    }
                    .frame(minHeight: geometry.size.height)
                }
                .scrollBounceBehavior(.basedOnSize)
                .scrollDismissesKeyboard(.interactively)
                .scrollIndicators(.hidden)
            }
            .ignoresSafeArea(.container, edges: .bottom)
        }
    }

    // MARK: - Logo

    private var logoSection: some View {
        VStack(spacing: OffriiTheme.spacingMD) {
            RoundedRectangle(cornerRadius: OffriiTheme.cornerRadiusLG)
                .fill(OffriiTheme.primary)
                .frame(width: 72, height: 72)
                .overlay(
                    Image(systemName: "gift.fill")
                        .font(.system(size: 32))
                        .foregroundColor(.white)
                )

            Text("Offrii")
                .font(OffriiTypography.titleLarge)
                .foregroundColor(OffriiTheme.text)
        }
    }

    // MARK: - Card

    private var cardSection: some View {
        VStack(alignment: .leading, spacing: OffriiTheme.spacingLG) {
            Text(NSLocalizedString("auth.register", comment: ""))
                .font(OffriiTypography.titleLarge)
                .foregroundColor(OffriiTheme.text)

            // SSO buttons
            VStack(spacing: OffriiTheme.spacingSM) {
                HStack(spacing: OffriiTheme.spacingSM) {
                    SSOButton(provider: .google) {}
                    SSOButton(provider: .facebook) {}
                }
                SSOButton(provider: .apple) {}
            }

            // Divider
            HStack {
                Rectangle().fill(OffriiTheme.border).frame(height: 1)
                Text(NSLocalizedString("auth.or", comment: ""))
                    .font(OffriiTypography.caption)
                    .foregroundColor(OffriiTheme.textMuted)
                Rectangle().fill(OffriiTheme.border).frame(height: 1)
            }

            // Fields
            VStack(spacing: OffriiTheme.spacingMD) {
                OffriiTextField(
                    label: "",
                    text: $viewModel.displayName,
                    placeholder: NSLocalizedString("auth.displayName", comment: ""),
                    style: .filled,
                    textContentType: .name,
                    autocapitalization: .words
                )
                .focused($focusedField, equals: .displayName)
                .submitLabel(.next)
                .onSubmit { focusedField = .email }

                OffriiTextField(
                    label: "",
                    text: $viewModel.email,
                    placeholder: NSLocalizedString("auth.enterEmail", comment: ""),
                    errorMessage: viewModel.emailError,
                    style: .filled,
                    keyboardType: .emailAddress,
                    textContentType: .emailAddress,
                    autocapitalization: .never
                )
                .focused($focusedField, equals: .email)
                .submitLabel(.next)
                .onSubmit { focusedField = .password }

                OffriiTextField(
                    label: "",
                    text: $viewModel.password,
                    placeholder: NSLocalizedString("auth.password", comment: ""),
                    errorMessage: viewModel.passwordError,
                    isSecure: true,
                    style: .filled,
                    textContentType: .newPassword
                )
                .focused($focusedField, equals: .password)
                .submitLabel(.next)
                .onSubmit { focusedField = .confirmPassword }

                OffriiTextField(
                    label: "",
                    text: $viewModel.confirmPassword,
                    placeholder: NSLocalizedString("auth.confirmPassword", comment: ""),
                    errorMessage: viewModel.confirmPasswordError,
                    isSecure: true,
                    style: .filled,
                    textContentType: .newPassword
                )
                .focused($focusedField, equals: .confirmPassword)
                .submitLabel(.go)
                .onSubmit {
                    Task {
                        if await viewModel.register(authManager: authManager) {
                            onAuthenticated()
                        }
                    }
                }
            }

            // Error + CTA
            VStack(spacing: OffriiTheme.spacingMD) {
                if case .error(let message) = viewModel.state {
                    Text(message)
                        .font(OffriiTypography.caption)
                        .foregroundColor(OffriiTheme.danger)
                        .frame(maxWidth: .infinity, alignment: .leading)
                }

                OffriiButton(
                    NSLocalizedString("auth.register", comment: ""),
                    variant: .primary,
                    isLoading: viewModel.isLoading
                ) {
                    Task {
                        if await viewModel.register(authManager: authManager) {
                            onAuthenticated()
                        }
                    }
                }
            }

            // Switch link
            Button {
                onSwitchToLogin()
            } label: {
                HStack(spacing: OffriiTheme.spacingXS) {
                    Text(NSLocalizedString("auth.alreadyAccount", comment: ""))
                        .foregroundColor(OffriiTheme.textSecondary)
                    Text(NSLocalizedString("auth.signIn", comment: ""))
                        .foregroundColor(OffriiTheme.primary)
                        .fontWeight(.semibold)
                }
                .font(OffriiTypography.subheadline)
                .frame(maxWidth: .infinity)
            }
        }
        .padding(.horizontal, OffriiTheme.spacingXL)
        .padding(.vertical, OffriiTheme.spacingXXL)
        .background(OffriiTheme.card)
        .cornerRadius(OffriiTheme.cornerRadiusXXL)
        .shadow(color: .black.opacity(0.08), radius: 20, y: 8)
        .padding(.horizontal, OffriiTheme.spacingBase)
    }
}
