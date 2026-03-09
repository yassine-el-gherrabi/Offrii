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
            OffriiTheme.primary.ignoresSafeArea()
            DecorativeSquares(preset: .authScreen)

            GeometryReader { geometry in
                ScrollView(showsIndicators: false) {
                    VStack(spacing: OffriiTheme.spacingLG) {
                        logoSection
                            .padding(.top, 60)

                        Spacer(minLength: 0)

                        cardWithStackEffect
                            .padding(.bottom, 14)
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
            RoundedRectangle(cornerRadius: 24)
                .fill(OffriiTheme.text)
                .frame(width: 96, height: 90)
                .overlay(
                    RoundedRectangle(cornerRadius: 26)
                        .stroke(Color.white.opacity(0.20), lineWidth: 3)
                        .frame(width: 100, height: 94)
                )
                .overlay(
                    Image(systemName: "gift.fill")
                        .font(.system(size: 40))
                        .foregroundColor(.white)
                )

            Text("Offrii")
                .font(OffriiTypography.title)
                .foregroundColor(.white)
        }
    }

    // MARK: - Card with Stack Effect

    private var cardWithStackEffect: some View {
        cardInner
            .background(
                RoundedRectangle(cornerRadius: 44)
                    .fill(Color.white.opacity(0.20))
                    .padding(.horizontal, 12)
                    .offset(y: -16)
            )
            .background(
                RoundedRectangle(cornerRadius: 44)
                    .fill(Color.white.opacity(0.08))
                    .padding(.horizontal, 24)
                    .offset(y: -32)
            )
            .padding(.horizontal, 14)
    }

    // MARK: - Card

    private var cardInner: some View {
        VStack(alignment: .leading, spacing: OffriiTheme.spacingLG) {
            // Sparkle + title
            VStack(alignment: .leading, spacing: OffriiTheme.spacingXS) {
                Image(systemName: "sparkle")
                    .font(.system(size: 20, weight: .bold))
                    .foregroundColor(OffriiTheme.text)

                Text(NSLocalizedString("auth.register", comment: ""))
                    .font(OffriiTypography.largeTitle)
                    .foregroundColor(OffriiTheme.text)
            }

            // SSO buttons
            VStack(spacing: OffriiTheme.spacingSM) {
                HStack(spacing: OffriiTheme.spacingSM) {
                    SSOButton(provider: .google) {}
                    SSOButton(provider: .facebook) {}
                }
                SSOButton(provider: .apple) {}
            }

            // Fields
            VStack(spacing: OffriiTheme.spacingSM) {
                OffriiTextField(
                    label: "",
                    text: $viewModel.displayName,
                    placeholder: NSLocalizedString("auth.displayName", comment: ""),
                    style: .underline,
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
                    style: .underline,
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
                    style: .underline,
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
                    style: .underline,
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
                    variant: .dark,
                    isLoading: viewModel.isLoading
                ) {
                    Task {
                        if await viewModel.register(authManager: authManager) {
                            onAuthenticated()
                        }
                    }
                }
            }

            // Switch link — inside card
            Button {
                onSwitchToLogin()
            } label: {
                HStack(spacing: OffriiTheme.spacingXS) {
                    Text(NSLocalizedString("auth.alreadyAccount", comment: ""))
                        .foregroundColor(OffriiTheme.textSecondary)
                    Text(NSLocalizedString("auth.signIn", comment: ""))
                        .foregroundColor(OffriiTheme.text)
                        .fontWeight(.semibold)
                }
                .font(OffriiTypography.subheadline)
                .frame(maxWidth: .infinity)
            }
        }
        .padding(.horizontal, 28)
        .padding(.top, 30)
        .padding(.bottom, 40)
        .background(Color.white)
        .cornerRadius(44)
        .shadow(color: .black.opacity(0.08), radius: 20, y: 8)
    }

}
