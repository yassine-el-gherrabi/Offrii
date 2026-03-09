import SwiftUI

struct LoginView: View {
    @Environment(AuthManager.self) private var authManager
    @State private var viewModel = AuthViewModel()

    let onAuthenticated: () -> Void
    let onSwitchToRegister: () -> Void

    @State private var showForgotPassword = false
    @FocusState private var focusedField: LoginField?

    private enum LoginField: Hashable {
        case email, password
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
        .sheet(isPresented: $showForgotPassword) {
            ForgotPasswordView(onDone: { showForgotPassword = false })
                .environment(authManager)
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

                Text(NSLocalizedString("auth.login", comment: ""))
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
                    text: $viewModel.email,
                    placeholder: NSLocalizedString("auth.email", comment: ""),
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
                    textContentType: .password
                )
                .focused($focusedField, equals: .password)
                .submitLabel(.go)
                .onSubmit {
                    Task {
                        if await viewModel.login(authManager: authManager) {
                            onAuthenticated()
                        }
                    }
                }

                HStack {
                    Spacer()
                    Button(NSLocalizedString("auth.forgotPassword", comment: "")) {
                        showForgotPassword = true
                    }
                    .font(OffriiTypography.subheadline)
                    .foregroundColor(OffriiTheme.text)
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
                    NSLocalizedString("auth.login", comment: ""),
                    variant: .dark,
                    isLoading: viewModel.isLoading
                ) {
                    Task {
                        if await viewModel.login(authManager: authManager) {
                            onAuthenticated()
                        }
                    }
                }
            }

            // Switch link — inside card
            Button {
                onSwitchToRegister()
            } label: {
                HStack(spacing: OffriiTheme.spacingXS) {
                    Text(NSLocalizedString("auth.noAccount", comment: ""))
                        .foregroundColor(OffriiTheme.textSecondary)
                    Text(NSLocalizedString("auth.signUp", comment: ""))
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
