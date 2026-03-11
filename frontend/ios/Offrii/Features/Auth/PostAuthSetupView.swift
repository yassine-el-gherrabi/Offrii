import SwiftUI
import UserNotifications

struct PostAuthSetupView: View {
    @Environment(AuthManager.self) private var authManager

    let onComplete: () -> Void

    @State private var step: SetupStep = .displayName
    @State private var displayName = ""
    @State private var username = ""
    @State private var usernameError: String?
    @State private var isLoading = false

    private enum SetupStep {
        case displayName
        case username
        case notifications
    }

    var body: some View {
        ZStack {
            OffriiTheme.background.ignoresSafeArea()

            BlobBackground(preset: .auth)
                .ignoresSafeArea()
                .opacity(0.3)

            Group {
                switch step {
                case .displayName:
                    displayNameStep
                case .username:
                    usernameStep
                case .notifications:
                    notificationsStep
                }
            }
            .transition(.asymmetric(
                insertion: .move(edge: .trailing).combined(with: .opacity),
                removal: .move(edge: .leading).combined(with: .opacity)
            ))
            .animation(OffriiAnimation.modal, value: step)
        }
    }

    // MARK: - Display Name Step

    private var displayNameStep: some View {
        VStack(spacing: OffriiTheme.spacingLG) {
            Spacer()

            Image(systemName: "person.crop.circle")
                .font(.system(size: 50))
                .foregroundColor(OffriiTheme.primary)

            Text(NSLocalizedString("postauth.displayName.title", comment: ""))
                .font(OffriiTypography.titleLarge)
                .foregroundColor(OffriiTheme.text)
                .multilineTextAlignment(.center)

            Text(NSLocalizedString("postauth.displayName.subtitle", comment: ""))
                .font(OffriiTypography.body)
                .foregroundColor(OffriiTheme.textSecondary)
                .multilineTextAlignment(.center)
                .padding(.horizontal, OffriiTheme.spacingXL)

            OffriiTextField(
                label: "",
                text: $displayName,
                placeholder: NSLocalizedString("auth.displayNamePlaceholder", comment: ""),
                style: .filled,
                textContentType: .name,
                autocapitalization: .words
            )
            .padding(.horizontal, OffriiTheme.spacingBase)

            Spacer()

            VStack(spacing: OffriiTheme.spacingMD) {
                OffriiButton(
                    NSLocalizedString("common.continue", comment: ""),
                    variant: .primary,
                    isLoading: isLoading
                ) {
                    Task { await saveDisplayName() }
                }

                Button {
                    advanceToUsername()
                } label: {
                    Text(NSLocalizedString("postauth.username.skip", comment: ""))
                        .font(OffriiTypography.subheadline)
                        .foregroundColor(OffriiTheme.textSecondary)
                }
            }
            .padding(.horizontal, OffriiTheme.spacingXL)
            .padding(.bottom, OffriiTheme.spacingXXL)
        }
    }

    // MARK: - Username Step

    private var usernameStep: some View {
        VStack(spacing: OffriiTheme.spacingLG) {
            Spacer()

            Image(systemName: "at")
                .font(.system(size: 50))
                .foregroundColor(OffriiTheme.primary)

            Text(NSLocalizedString("postauth.username.title", comment: ""))
                .font(OffriiTypography.titleLarge)
                .foregroundColor(OffriiTheme.text)
                .multilineTextAlignment(.center)

            Text(NSLocalizedString("postauth.username.subtitle", comment: ""))
                .font(OffriiTypography.body)
                .foregroundColor(OffriiTheme.textSecondary)
                .multilineTextAlignment(.center)

            OffriiTextField(
                label: "",
                text: $username,
                placeholder: NSLocalizedString("postauth.username.placeholder", comment: ""),
                errorMessage: usernameError,
                style: .filled,
                autocapitalization: .never
            )
            .padding(.horizontal, OffriiTheme.spacingBase)

            Spacer()

            VStack(spacing: OffriiTheme.spacingMD) {
                OffriiButton(
                    NSLocalizedString("postauth.username.continue", comment: ""),
                    variant: .primary,
                    isLoading: isLoading
                ) {
                    Task { await saveUsername() }
                }

                Button {
                    advanceToNotifications()
                } label: {
                    Text(NSLocalizedString("postauth.username.skip", comment: ""))
                        .font(OffriiTypography.subheadline)
                        .foregroundColor(OffriiTheme.textSecondary)
                }
            }
            .padding(.horizontal, OffriiTheme.spacingXL)
            .padding(.bottom, OffriiTheme.spacingXXL)
        }
    }

    // MARK: - Notifications Step

    private var notificationsStep: some View {
        VStack(spacing: OffriiTheme.spacingLG) {
            Spacer()

            Image(systemName: "bell.badge")
                .font(.system(size: 50))
                .foregroundColor(OffriiTheme.accent)

            Text(NSLocalizedString("postauth.notifications.title", comment: ""))
                .font(OffriiTypography.titleLarge)
                .foregroundColor(OffriiTheme.text)
                .multilineTextAlignment(.center)

            Text(NSLocalizedString("postauth.notifications.subtitle", comment: ""))
                .font(OffriiTypography.body)
                .foregroundColor(OffriiTheme.textSecondary)
                .multilineTextAlignment(.center)
                .padding(.horizontal, OffriiTheme.spacingXL)

            Spacer()

            VStack(spacing: OffriiTheme.spacingMD) {
                OffriiButton(
                    NSLocalizedString("postauth.notifications.enable", comment: ""),
                    variant: .primary
                ) {
                    Task { await requestNotifications() }
                }

                Button {
                    onComplete()
                } label: {
                    Text(NSLocalizedString("postauth.notifications.skip", comment: ""))
                        .font(OffriiTypography.subheadline)
                        .foregroundColor(OffriiTheme.textSecondary)
                }
            }
            .padding(.horizontal, OffriiTheme.spacingXL)
            .padding(.bottom, OffriiTheme.spacingXXL)
        }
    }

    // MARK: - Actions

    private func saveDisplayName() async {
        let trimmed = displayName.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmed.isEmpty else {
            advanceToUsername()
            return
        }

        isLoading = true
        defer { isLoading = false }

        do {
            let body = UpdateProfileBody(
                displayName: trimmed,
                username: nil,
                reminderFreq: nil,
                reminderTime: nil,
                timezone: nil,
                locale: nil
            )
            try await APIClient.shared.requestVoid(.updateProfile(body))
            try? await authManager.loadCurrentUser()
        } catch {
            // Non-critical — proceed anyway
        }
        advanceToUsername()
    }

    private func saveUsername() async {
        let trimmed = username.trimmingCharacters(in: .whitespacesAndNewlines).lowercased()
        guard !trimmed.isEmpty else {
            usernameError = NSLocalizedString("error.usernameInvalid", comment: "")
            return
        }

        isLoading = true
        defer { isLoading = false }

        do {
            let body = UpdateProfileBody(
                displayName: nil,
                username: trimmed,
                reminderFreq: nil,
                reminderTime: nil,
                timezone: nil,
                locale: nil
            )
            try await APIClient.shared.requestVoid(.updateProfile(body))
            try? await authManager.loadCurrentUser()
            advanceToNotifications()
        } catch {
            usernameError = NSLocalizedString("error.usernameTaken", comment: "")
        }
    }

    private func advanceToUsername() {
        withAnimation(OffriiAnimation.modal) {
            step = .username
        }
    }

    private func advanceToNotifications() {
        withAnimation(OffriiAnimation.modal) {
            step = .notifications
        }
    }

    private func requestNotifications() async {
        let center = UNUserNotificationCenter.current()
        _ = try? await center.requestAuthorization(options: [.alert, .badge, .sound])
        onComplete()
    }
}
