import SwiftUI

struct RegisterView: View {
    @Environment(AuthManager.self) private var authManager
    @State private var viewModel = AuthViewModel()

    let onAuthenticated: (_ isNewUser: Bool) -> Void
    let onSwitchToLogin: () -> Void

    @State private var ssoService = SSOService()
    @FocusState private var focusedField: RegisterField?
    @State private var appeared = false

    private enum RegisterField: Hashable {
        case email, password
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
                            .padding(.top, 40)

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
            ShinyIcon(systemName: "gift.fill", color: OffriiTheme.primary)

            Text("Offrii")
                .font(OffriiTypography.titleLarge)
                .foregroundColor(OffriiTheme.text)
        }
    }

    // MARK: - Card

    private var cardSection: some View {
        VStack(alignment: .leading, spacing: OffriiTheme.spacingLG) {
            // Title + subtitle
            VStack(alignment: .leading, spacing: OffriiTheme.spacingXS) {
                Text(NSLocalizedString("auth.register", comment: ""))
                    .font(OffriiTypography.titleLarge)
                    .foregroundColor(OffriiTheme.text)

                Text(NSLocalizedString("auth.registerSubtitle", comment: ""))
                    .font(OffriiTypography.body)
                    .foregroundColor(OffriiTheme.textSecondary)
            }

            // Fields — email + password only
            VStack(spacing: OffriiTheme.spacingMD) {
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
                    placeholder: NSLocalizedString("auth.passwordPlaceholder", comment: ""),
                    errorMessage: viewModel.passwordError,
                    isSecure: true,
                    style: .filled,
                    textContentType: .newPassword
                )
                .focused($focusedField, equals: .password)
                .submitLabel(.go)
                .onSubmit {
                    Task {
                        if await viewModel.register(authManager: authManager) {
                            onAuthenticated(true)
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
                            onAuthenticated(true)
                        }
                    }
                }
            }

            // Divider
            HStack {
                Rectangle().fill(OffriiTheme.border).frame(height: 1)
                Text(NSLocalizedString("auth.or", comment: ""))
                    .font(OffriiTypography.caption)
                    .foregroundColor(OffriiTheme.textMuted)
                Rectangle().fill(OffriiTheme.border).frame(height: 1)
            }

            // SSO buttons
            VStack(spacing: OffriiTheme.spacingSM) {
                SSOButton(provider: .google, isLoading: viewModel.isSSOLoading(.google)) {
                    Task {
                        if let isNew = await viewModel.signInWithGoogle(authManager: authManager, ssoService: ssoService) {
                            onAuthenticated(isNew)
                        }
                    }
                }
                .disabled(viewModel.isAnyLoading)
                SSOButton(provider: .apple, isLoading: viewModel.isSSOLoading(.apple)) {
                    Task {
                        if let isNew = await viewModel.signInWithApple(authManager: authManager, ssoService: ssoService) {
                            onAuthenticated(isNew)
                        }
                    }
                }
                .disabled(viewModel.isAnyLoading)
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
        .opacity(appeared ? 1 : 0)
        .offset(y: appeared ? 0 : 30)
        .onAppear {
            withAnimation(OffriiAnimation.modal.delay(0.15)) {
                appeared = true
            }
        }
    }
}
