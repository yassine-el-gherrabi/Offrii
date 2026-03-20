import SwiftUI

struct ForgotPasswordView: View {
    @Environment(AuthManager.self) private var authManager
    @State private var viewModel = AuthViewModel()

    let onDone: () -> Void

    @Environment(\.dismiss) private var dismiss
    @State private var appeared = false
    @State private var resendCooldown = 0
    @State private var cooldownTask: Task<Void, Never>?
    @State private var codeExpiryMinutes = 30
    @State private var expiryTask: Task<Void, Never>?
    @State private var showResendConfirmation = false
    @State private var resendCount = 0

    var body: some View {
        NavigationStack {
            ZStack {
                OffriiTheme.background.ignoresSafeArea()

                BlobBackground(preset: .auth)
                    .ignoresSafeArea()
                    .opacity(0.3)

                GeometryReader { geometry in
                    ScrollView(showsIndicators: false) {
                        VStack(spacing: OffriiTheme.spacingLG) {
                            Spacer(minLength: OffriiTheme.spacingXXL)

                            stepIcon

                            stepHeader

                            cardSection

                            Spacer(minLength: 0)
                        }
                        .frame(minHeight: geometry.size.height)
                        .padding(.horizontal, OffriiTheme.spacingBase)
                    }
                    .scrollBounceBehavior(.basedOnSize)
                    .scrollDismissesKeyboard(.interactively)
                    .scrollIndicators(.hidden)
                }
            }
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button(NSLocalizedString("common.cancel", comment: "")) {
                        cleanup()
                        viewModel.resetForgotState()
                        dismiss()
                    }
                    .foregroundColor(OffriiTheme.primary)
                }
            }
        }
        .onDisappear { cleanup() }
    }

    // MARK: - Step Icon

    private var stepIcon: some View {
        Group {
            switch viewModel.forgotStep {
            case .email:
                ShinyIcon(systemName: "envelope.circle", color: OffriiTheme.primary)
            case .code:
                ShinyIcon(systemName: "lock.shield", color: OffriiTheme.accent)
            case .newPassword:
                ShinyIcon(systemName: "key.fill", color: OffriiTheme.primary)
            }
        }
        .opacity(appeared ? 1 : 0)
        .offset(y: appeared ? 0 : 20)
        .onAppear {
            withAnimation(OffriiAnimation.modal.delay(0.1)) {
                appeared = true
            }
        }
    }

    // MARK: - Step Header

    private var stepHeader: some View {
        VStack(spacing: OffriiTheme.spacingXS) {
            Text(stepTitle)
                .font(OffriiTypography.titleLarge)
                .foregroundColor(OffriiTheme.text)
                .multilineTextAlignment(.center)

            Text(stepSubtitle)
                .font(OffriiTypography.body)
                .foregroundColor(OffriiTheme.textSecondary)
                .multilineTextAlignment(.center)
        }
        .opacity(appeared ? 1 : 0)
    }

    private var stepTitle: String {
        switch viewModel.forgotStep {
        case .email:
            return NSLocalizedString("auth.forgotPassword", comment: "")
        case .code:
            return NSLocalizedString("auth.enterCode", comment: "")
        case .newPassword:
            return NSLocalizedString("auth.newPassword", comment: "")
        }
    }

    private var stepSubtitle: String {
        switch viewModel.forgotStep {
        case .email:
            return NSLocalizedString("auth.forgotSubtitle", comment: "")
        case .code:
            return NSLocalizedString("auth.codeSentTo", comment: "") + " " + viewModel.resetEmail
        case .newPassword:
            return NSLocalizedString("auth.newPasswordSubtitle", comment: "")
        }
    }

    // MARK: - Card

    private var cardSection: some View {
        VStack(spacing: OffriiTheme.spacingLG) {
            // Step indicator
            Text(String(
                format: NSLocalizedString("auth.step", comment: ""),
                viewModel.forgotStep.rawValue,
                ForgotPasswordStep.allCases.count
            ))
            .font(OffriiTypography.caption)
            .foregroundColor(OffriiTheme.textMuted)
            .frame(maxWidth: .infinity, alignment: .leading)

            // Step content
            stepContent

            // Error
            if case .error(let message) = viewModel.state {
                Text(message)
                    .font(OffriiTypography.caption)
                    .foregroundColor(OffriiTheme.danger)
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .transition(.opacity.combined(with: .move(edge: .top)))
            }

            // Action button
            stepButton

            // Resend code (only on code step)
            if viewModel.forgotStep == .code {
                resendSection
            }
        }
        .padding(.horizontal, OffriiTheme.spacingXL)
        .padding(.vertical, OffriiTheme.spacingXXL)
        .background(OffriiTheme.card)
        .cornerRadius(OffriiTheme.cornerRadiusXXL)
        .shadow(color: .black.opacity(0.08), radius: 20, y: 8)
        .opacity(appeared ? 1 : 0)
        .offset(y: appeared ? 0 : 30)
    }

    // MARK: - Step Content

    @ViewBuilder
    private var stepContent: some View {
        switch viewModel.forgotStep {
        case .email:
            OffriiTextField(
                label: "",
                text: $viewModel.resetEmail,
                placeholder: NSLocalizedString("auth.enterEmail", comment: ""),
                errorMessage: viewModel.emailError,
                style: .filled,
                keyboardType: .emailAddress,
                textContentType: .emailAddress,
                autocapitalization: .never
            )

        case .code:
            VStack(spacing: OffriiTheme.spacingSM) {
                OTPField(code: $viewModel.resetCode, errorMessage: viewModel.codeError)

                // Expiry indicator
                HStack(spacing: OffriiTheme.spacingXS) {
                    Image(systemName: "clock")
                        .font(.system(size: 12))
                    Text(String(
                        format: NSLocalizedString("auth.codeExpiry", comment: ""),
                        codeExpiryMinutes
                    ))
                }
                .font(OffriiTypography.caption)
                .foregroundColor(codeExpiryMinutes <= 5 ? OffriiTheme.danger : OffriiTheme.textMuted)
                .frame(maxWidth: .infinity, alignment: .leading)
            }

        case .newPassword:
            OffriiTextField(
                label: "",
                text: $viewModel.newPassword,
                placeholder: NSLocalizedString("auth.passwordPlaceholder", comment: ""),
                errorMessage: viewModel.newPasswordError,
                isSecure: true,
                style: .filled,
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
                NSLocalizedString("common.continue", comment: ""),
                variant: .primary,
                isLoading: viewModel.isLoading
            ) {
                Task {
                    if await viewModel.sendResetCode() {
                        startExpiryTimer()
                    }
                }
            }

        case .code:
            OffriiButton(
                NSLocalizedString("common.continue", comment: ""),
                variant: .primary,
                isLoading: viewModel.isLoading
            ) {
                Task {
                    if await viewModel.verifyResetCode() {
                        withAnimation(OffriiAnimation.modal) {
                            viewModel.forgotStep = .newPassword
                        }
                    }
                }
            }

        case .newPassword:
            OffriiButton(
                NSLocalizedString("auth.resetPassword", comment: ""),
                variant: .primary,
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

    // MARK: - Resend Section

    private var resendSection: some View {
        VStack(spacing: OffriiTheme.spacingXS) {
            if resendCount >= 3 {
                Text(NSLocalizedString("auth.resendLimitReached", comment: ""))
                    .font(OffriiTypography.subheadline)
                    .foregroundColor(OffriiTheme.textMuted)
            } else if showResendConfirmation {
                HStack(spacing: OffriiTheme.spacingXS) {
                    Image(systemName: "checkmark.circle.fill")
                        .foregroundColor(OffriiTheme.success)
                    Text(NSLocalizedString("auth.codeSent", comment: ""))
                        .foregroundColor(OffriiTheme.success)
                }
                .font(OffriiTypography.subheadline)
                .transition(.opacity)
            } else if resendCooldown > 0 {
                Text(NSLocalizedString("auth.resendCode", comment: "") + " (\(resendCooldown)s)")
                    .font(OffriiTypography.subheadline)
                    .foregroundColor(OffriiTheme.textMuted)
            } else {
                Button {
                    Task { await resendCode() }
                } label: {
                    Text(NSLocalizedString("auth.resendCode", comment: ""))
                        .font(OffriiTypography.subheadline)
                        .foregroundColor(OffriiTheme.primary)
                }
            }
        }
        .frame(maxWidth: .infinity)
    }

    // MARK: - Actions

    private func resendCode() async {
        viewModel.resetCode = ""
        viewModel.codeError = nil

        if await viewModel.sendResetCode() {
            resendCount += 1
            startResendCooldown()
            showResendConfirmation = true
            codeExpiryMinutes = 30
            startExpiryTimer()

            DispatchQueue.main.asyncAfter(deadline: .now() + 3) {
                withAnimation(OffriiAnimation.snappy) {
                    showResendConfirmation = false
                }
            }
        } else {
            // If rate limited (429), start cooldown anyway so user sees the timer
            if case .error = viewModel.state {
                startResendCooldown()
                viewModel.state = .idle
            }
        }
    }

    private func startResendCooldown() {
        resendCooldown = 60
        cooldownTask?.cancel()
        cooldownTask = Task {
            while resendCooldown > 0 {
                try? await Task.sleep(for: .seconds(1))
                guard !Task.isCancelled else { return }
                resendCooldown -= 1
            }
        }
    }

    private func startExpiryTimer() {
        codeExpiryMinutes = 30
        expiryTask?.cancel()
        expiryTask = Task {
            while codeExpiryMinutes > 0 {
                try? await Task.sleep(for: .seconds(60))
                guard !Task.isCancelled else { return }
                codeExpiryMinutes -= 1
            }
        }
    }

    private func cleanup() {
        cooldownTask?.cancel()
        cooldownTask = nil
        expiryTask?.cancel()
        expiryTask = nil
    }
}
