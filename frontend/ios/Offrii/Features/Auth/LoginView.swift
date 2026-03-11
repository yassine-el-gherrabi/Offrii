import SwiftUI

struct LoginView: View {
    @Environment(AuthManager.self) private var authManager
    @State private var viewModel = AuthViewModel()

    let isReturningUser: Bool
    let onAuthenticated: () -> Void
    let onSwitchToRegister: () -> Void

    @State private var showForgotPassword = false
    @FocusState private var focusedField: LoginField?

    private enum LoginField: Hashable {
        case email, password
    }

    var body: some View {
        ZStack {
            // Warm white background
            OffriiTheme.background.ignoresSafeArea()

            // Blob decorations
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
        .sheet(isPresented: $showForgotPassword) {
            ForgotPasswordView(onDone: { showForgotPassword = false })
                .environment(authManager)
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
            // Title
            Text(NSLocalizedString(isReturningUser ? "auth.welcomeBack" : "auth.login", comment: ""))
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
                    text: $viewModel.email,
                    placeholder: NSLocalizedString("auth.email", comment: ""),
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
                    .foregroundColor(OffriiTheme.secondary)
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
                    variant: .primary,
                    isLoading: viewModel.isLoading
                ) {
                    Task {
                        if await viewModel.login(authManager: authManager) {
                            onAuthenticated()
                        }
                    }
                }
            }

            // Switch link
            Button {
                onSwitchToRegister()
            } label: {
                HStack(spacing: OffriiTheme.spacingXS) {
                    Text(NSLocalizedString("auth.noAccount", comment: ""))
                        .foregroundColor(OffriiTheme.textSecondary)
                    Text(NSLocalizedString("auth.signUp", comment: ""))
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
