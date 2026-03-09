import SwiftUI

struct ProfileView: View {
    @Environment(AuthManager.self) private var authManager
    @State private var viewModel = ProfileViewModel()
    @State private var showOnboarding = false

    var body: some View {
        ZStack {
            OffriiTheme.cardSurface.ignoresSafeArea()

            ScrollView {
                VStack(spacing: 0) {
                    // Header with avatar
                    ZStack {
                        OffriiTheme.primary.ignoresSafeArea(edges: .top)
                        DecorativeSquares(preset: .header)

                        VStack(spacing: OffriiTheme.spacingSM) {
                            // Avatar initials
                            Circle()
                                .fill(Color.white.opacity(0.2))
                                .frame(width: 72, height: 72)
                                .overlay(
                                    Text(viewModel.initials)
                                        .font(.system(size: 28, weight: .bold))
                                        .foregroundColor(.white)
                                )

                            Text(viewModel.displayName)
                                .font(OffriiTypography.title)
                                .foregroundColor(.white)

                            if !viewModel.username.isEmpty {
                                Text("@\(viewModel.username)")
                                    .font(OffriiTypography.subheadline)
                                    .foregroundColor(.white.opacity(0.8))
                            }

                            Text(viewModel.email)
                                .font(OffriiTypography.subheadline)
                                .foregroundColor(.white.opacity(0.8))
                        }
                        .padding(.vertical, OffriiTheme.spacingXL)
                    }
                    .frame(minHeight: 200)

                    VStack(spacing: OffriiTheme.spacingMD) {
                        if let error = viewModel.loadError {
                            HStack {
                                Image(systemName: "exclamationmark.triangle.fill")
                                    .foregroundColor(.orange)
                                Text(error)
                                    .font(OffriiTypography.subheadline)
                                    .foregroundColor(OffriiTheme.text)
                                Spacer()
                                Button(NSLocalizedString("common.retry", comment: "")) {
                                    Task { await viewModel.loadProfile() }
                                }
                                .font(OffriiTypography.subheadline)
                                .foregroundColor(OffriiTheme.primary)
                            }
                            .padding(OffriiTheme.spacingMD)
                            .background(Color.orange.opacity(0.1))
                            .cornerRadius(8)
                            .padding(.horizontal, OffriiTheme.spacingLG)
                        }

                        // Reminders section
                        profileSection(
                            title: NSLocalizedString("profile.reminders", comment: ""),
                            icon: "bell.fill"
                        ) {
                            NavigationLink {
                                ReminderSettingsView()
                                    .environment(authManager)
                            } label: {
                                profileRow(
                                    title: NSLocalizedString("profile.reminderFrequency", comment: ""),
                                    value: viewModel.reminderFreqLabel
                                )
                            }
                        }

                        // Friends section
                        profileSection(
                            title: NSLocalizedString("profile.friends", comment: ""),
                            icon: "person.2.fill"
                        ) {
                            NavigationLink {
                                FriendsView()
                                    .environment(authManager)
                            } label: {
                                profileRow(
                                    title: NSLocalizedString("friends.title", comment: ""),
                                    value: nil
                                )
                            }
                        }

                        // Account section
                        profileSection(
                            title: NSLocalizedString("profile.account", comment: ""),
                            icon: "person.fill"
                        ) {
                            NavigationLink {
                                UsernameEditView(viewModel: viewModel)
                            } label: {
                                profileRow(
                                    title: NSLocalizedString("profile.username", comment: ""),
                                    value: viewModel.username.isEmpty ? nil : "@\(viewModel.username)"
                                )
                            }
                        }

                        // Data section
                        profileSection(
                            title: NSLocalizedString("profile.data", comment: ""),
                            icon: "externaldrive.fill"
                        ) {
                            VStack(spacing: 0) {
                                NavigationLink {
                                    DataManagementView()
                                        .environment(authManager)
                                } label: {
                                    profileRow(
                                        title: NSLocalizedString("profile.exportData", comment: ""),
                                        value: nil
                                    )
                                }

                                Divider().padding(.leading, OffriiTheme.spacingMD)

                                NavigationLink {
                                    LegalView()
                                } label: {
                                    profileRow(
                                        title: NSLocalizedString("profile.legal", comment: ""),
                                        value: nil
                                    )
                                }

                                Divider().padding(.leading, OffriiTheme.spacingMD)

                                NavigationLink {
                                    LegalView(showPrivacy: true)
                                } label: {
                                    profileRow(
                                        title: NSLocalizedString("profile.privacy", comment: ""),
                                        value: nil
                                    )
                                }
                            }
                        }

                        // Tutorial replay
                        Button {
                            showOnboarding = true
                        } label: {
                            profileRow(
                                title: NSLocalizedString("profile.replayTutorial", comment: ""),
                                value: nil
                            )
                        }
                        .padding(.horizontal, OffriiTheme.spacingLG)

                        // Logout
                        OffriiButton(
                            NSLocalizedString("auth.logout", comment: ""),
                            variant: .danger,
                            isLoading: viewModel.isLoggingOut
                        ) {
                            Task {
                                viewModel.isLoggingOut = true
                                await authManager.logout()
                                viewModel.isLoggingOut = false
                            }
                        }
                        .padding(.horizontal, OffriiTheme.spacingLG)
                        .padding(.top, OffriiTheme.spacingMD)
                    }
                    .padding(.top, -OffriiTheme.spacingLG)
                    .padding(.bottom, OffriiTheme.spacingXL)
                }
            }
        }
        .navigationBarHidden(true)
        .fullScreenCover(isPresented: $showOnboarding) {
            OnboardingView(
                onComplete: { showOnboarding = false },
                onSignIn: { showOnboarding = false }
            )
        }
        .task {
            await viewModel.loadProfile()
        }
    }

    // MARK: - Helpers

    @ViewBuilder
    private func profileSection(title: String, icon: String, @ViewBuilder content: () -> some View) -> some View {
        VStack(alignment: .leading, spacing: OffriiTheme.spacingSM) {
            HStack(spacing: OffriiTheme.spacingSM) {
                Image(systemName: icon)
                    .foregroundColor(OffriiTheme.primary)
                    .font(.system(size: 14))
                Text(title)
                    .font(OffriiTypography.headline)
                    .foregroundColor(OffriiTheme.text)
            }
            .padding(.horizontal, OffriiTheme.spacingLG)

            OffriiCard {
                content()
            }
            .padding(.horizontal, OffriiTheme.spacingLG)
        }
    }

    private func profileRow(title: String, value: String?) -> some View {
        HStack {
            Text(title)
                .font(OffriiTypography.body)
                .foregroundColor(OffriiTheme.text)
            Spacer()
            if let value {
                Text(value)
                    .font(OffriiTypography.body)
                    .foregroundColor(OffriiTheme.textSecondary)
            }
            Image(systemName: "chevron.right")
                .font(.system(size: 12, weight: .semibold))
                .foregroundColor(OffriiTheme.textMuted)
        }
        .padding(.vertical, OffriiTheme.spacingSM)
    }
}
