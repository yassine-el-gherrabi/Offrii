import SwiftUI

struct ProfileView: View {
    @Environment(AuthManager.self) private var authManager
    @Environment(OnboardingTipManager.self) private var tipManager
    @State private var viewModel = ProfileViewModel()
    var body: some View {
        ZStack {
            OffriiTheme.background.ignoresSafeArea()

            ScrollView {
                VStack(spacing: 0) {
                    // Header with avatar
                    ZStack(alignment: .bottom) {
                        SectionHeader(
                            title: viewModel.displayName,
                            subtitle: viewModel.email,
                            variant: .profil
                        )

                        // Avatar overlapping the header bottom
                        VStack(spacing: OffriiTheme.spacingSM) {
                            AvatarView(viewModel.displayName, size: .xl)

                            if !viewModel.username.isEmpty {
                                Text("@\(viewModel.username)")
                                    .font(OffriiTypography.subheadline)
                                    .foregroundColor(.white.opacity(0.8))
                            }
                        }
                        .padding(.bottom, -OffriiTheme.spacingXXL)
                    }
                    .padding(.bottom, OffriiTheme.spacingXXL)

                    VStack(spacing: OffriiTheme.spacingBase) {
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
                            .padding(OffriiTheme.spacingBase)
                            .background(Color.orange.opacity(0.1))
                            .cornerRadius(OffriiTheme.cornerRadiusSM)
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
                        .overlay(alignment: .bottom) {
                            if tipManager.activeTip == .profileReminders {
                                OffriiTooltip(
                                    message: OnboardingTipManager.message(for: .profileReminders),
                                    arrow: .top
                                ) {
                                    tipManager.dismiss(.profileReminders)
                                }
                                .offset(y: 50)
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

                        // Community wishes section
                        profileSection(
                            title: NSLocalizedString("profile.communityWishes", comment: ""),
                            icon: "hand.raised.fill"
                        ) {
                            NavigationLink {
                                MyWishesView()
                            } label: {
                                profileRow(
                                    title: NSLocalizedString("entraide.myWishes.title", comment: ""),
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
                        .overlay(alignment: .bottom) {
                            if tipManager.activeTip == .profileUsername {
                                OffriiTooltip(
                                    message: OnboardingTipManager.message(for: .profileUsername),
                                    arrow: .top
                                ) {
                                    tipManager.dismiss(.profileUsername)
                                }
                                .offset(y: 50)
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

                                Divider().padding(.leading, OffriiTheme.spacingBase)

                                NavigationLink {
                                    LegalView()
                                } label: {
                                    profileRow(
                                        title: NSLocalizedString("profile.legal", comment: ""),
                                        value: nil
                                    )
                                }

                                Divider().padding(.leading, OffriiTheme.spacingBase)

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

                        // Reset tips
                        Button {
                            tipManager.resetAllTips()
                        } label: {
                            profileRow(
                                title: NSLocalizedString("profile.resetTips", comment: ""),
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
                        .padding(.top, OffriiTheme.spacingBase)
                    }
                    .padding(.top, OffriiTheme.spacingBase)
                    .padding(.bottom, OffriiTheme.spacingXL)
                }
            }
        }
        .navigationBarHidden(true)
        .task {
            await viewModel.loadProfile()
            if viewModel.username.isEmpty {
                tipManager.showIfNeeded(.profileUsername)
            } else if viewModel.reminderFreqLabel == NSLocalizedString("reminder.never", comment: "") {
                tipManager.showIfNeeded(.profileReminders)
            }
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
