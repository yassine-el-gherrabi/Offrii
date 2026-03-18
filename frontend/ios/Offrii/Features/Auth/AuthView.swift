import SwiftUI

// MARK: - Auth Mode

enum AuthMode: Int, CaseIterable {
    case login, register

    var label: String {
        switch self {
        case .login: return NSLocalizedString("auth.signIn", comment: "")
        case .register: return NSLocalizedString("auth.signUp", comment: "")
        }
    }
}

// MARK: - AuthView (unified login + register)

struct AuthView: View {
    @Environment(AuthManager.self) private var authManager

    let initialMode: AuthMode
    let onAuthenticated: (_ isNewUser: Bool) -> Void

    @State private var mode: AuthMode
    @State private var viewModel = AuthViewModel()
    @State private var ssoService = SSOService()
    @State private var showForgotPassword = false
    @State private var appeared = false
    @FocusState private var focusedField: AuthField?

    private enum AuthField: Hashable {
        case email, password
    }

    init(initialMode: AuthMode, onAuthenticated: @escaping (_ isNewUser: Bool) -> Void) {
        self.initialMode = initialMode
        self.onAuthenticated = onAuthenticated
        _mode = State(initialValue: initialMode)
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
        .sheet(isPresented: $showForgotPassword) {
            ForgotPasswordView(onDone: { showForgotPassword = false })
                .environment(authManager)
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
            // Segmented control
            Picker("", selection: $mode) {
                ForEach(AuthMode.allCases, id: \.self) { authMode in
                    Text(authMode.label).tag(authMode)
                }
            }
            .pickerStyle(.segmented)
            .onChange(of: mode) { _, _ in
                viewModel.state = .idle
                viewModel.emailError = nil
                viewModel.passwordError = nil
            }

            // Title + subtitle
            VStack(alignment: .leading, spacing: OffriiTheme.spacingXS) {
                Text(NSLocalizedString(
                    mode == .login ? "auth.loginSubtitle" : "auth.registerSubtitle",
                    comment: ""
                ))
                .font(OffriiTypography.body)
                .foregroundColor(OffriiTheme.textSecondary)
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
                    placeholder: NSLocalizedString(
                        mode == .login ? "auth.password" : "auth.passwordPlaceholder",
                        comment: ""
                    ),
                    errorMessage: viewModel.passwordError,
                    isSecure: true,
                    style: .filled,
                    textContentType: mode == .login ? .password : .newPassword
                )
                .focused($focusedField, equals: .password)
                .submitLabel(.go)
                .onSubmit { submit() }

                if mode == .login {
                    HStack {
                        Spacer()
                        Button(NSLocalizedString("auth.forgotPassword", comment: "")) {
                            showForgotPassword = true
                        }
                        .font(OffriiTypography.subheadline)
                        .foregroundColor(OffriiTheme.textSecondary)
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
                    NSLocalizedString(mode == .login ? "auth.login" : "auth.register", comment: ""),
                    variant: .primary,
                    isLoading: viewModel.isLoading
                ) {
                    submit()
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
                        if let isNew = await viewModel.signInWithGoogle(
                            authManager: authManager, ssoService: ssoService
                        ) {
                            onAuthenticated(isNew)
                        }
                    }
                }
                .disabled(viewModel.isAnyLoading)
                SSOButton(provider: .apple, isLoading: viewModel.isSSOLoading(.apple)) {
                    Task {
                        if let isNew = await viewModel.signInWithApple(
                            authManager: authManager, ssoService: ssoService
                        ) {
                            onAuthenticated(isNew)
                        }
                    }
                }
                .disabled(viewModel.isAnyLoading)
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

    // MARK: - Submit

    private func submit() {
        Task {
            switch mode {
            case .login:
                if await viewModel.login(authManager: authManager) {
                    onAuthenticated(false)
                }
            case .register:
                if await viewModel.register(authManager: authManager) {
                    onAuthenticated(true)
                }
            }
        }
    }
}
