import PhotosUI
import SwiftUI

struct ProfileView: View {
    @Environment(AuthManager.self) private var authManager
    @Environment(OnboardingTipManager.self) private var tipManager
    @State private var viewModel = ProfileViewModel()
    @State private var profileProgress = ProfileProgress()
    @State private var selectedAvatarImage: UIImage?
    @State private var isUploadingAvatar = false
    @State private var showAvatarSourceSheet = false
    @State private var showAvatarCamera = false
    @State private var showAvatarPhotoPicker = false
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
                            ZStack(alignment: .bottomTrailing) {
                                if let selected = selectedAvatarImage {
                                    Image(uiImage: selected)
                                        .resizable()
                                        .aspectRatio(contentMode: .fill)
                                        .frame(width: 96, height: 96)
                                        .clipShape(Circle())
                                } else {
                                    AvatarView(
                                        viewModel.displayName,
                                        size: .xl,
                                        url: viewModel.avatarUrl
                                    )
                                }

                                // Upload progress overlay
                                if isUploadingAvatar {
                                    Circle()
                                        .fill(.black.opacity(0.4))
                                        .frame(width: 96, height: 96)
                                        .overlay {
                                            ProgressView()
                                                .tint(.white)
                                        }
                                }

                                Button {
                                    showAvatarSourceSheet = true
                                } label: {
                                    Image(systemName: "camera.fill")
                                        .font(.system(size: 12))
                                        .foregroundColor(.white)
                                        .frame(width: 28, height: 28)
                                        .background(OffriiTheme.primary)
                                        .clipShape(Circle())
                                        .overlay(
                                            Circle().strokeBorder(.white, lineWidth: 2)
                                        )
                                }
                                .disabled(isUploadingAvatar)
                            }

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
                        // Profile progress
                        if profileProgress.percentage < 100 {
                            ProfileProgressCard(progress: profileProgress)
                                .padding(.horizontal, OffriiTheme.spacingLG)
                        }

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
        .confirmationDialog(
            NSLocalizedString("imagePicker.add", comment: ""),
            isPresented: $showAvatarSourceSheet,
            titleVisibility: .visible
        ) {
            if UIImagePickerController.isSourceTypeAvailable(.camera) {
                Button(NSLocalizedString("imagePicker.takePhoto", comment: "")) {
                    showAvatarCamera = true
                }
            }
            Button(NSLocalizedString("imagePicker.chooseFromGallery", comment: "")) {
                DispatchQueue.main.asyncAfter(deadline: .now() + 0.3) {
                    showAvatarPhotoPicker = true
                }
            }
        }
        .fullScreenCover(isPresented: $showAvatarCamera) {
            CameraImagePicker(image: Binding(
                get: { selectedAvatarImage },
                set: { img in
                    if let img {
                        selectedAvatarImage = img
                        Task { await uploadAvatar(img) }
                    }
                }
            ))
            .ignoresSafeArea()
        }
        .photosPicker(
            isPresented: $showAvatarPhotoPicker,
            selection: Binding(
                get: { nil },
                set: { item in
                    if let item {
                        Task { await loadAvatarImage(item) }
                    }
                }
            ),
            matching: .images
        )
        .task {
            await viewModel.loadProfile()
            await computeProfileProgress()
            if viewModel.username.isEmpty {
                tipManager.showIfNeeded(.profileUsername)
            } else if viewModel.reminderFreqLabel == NSLocalizedString("reminder.never", comment: "") {
                tipManager.showIfNeeded(.profileReminders)
            }
        }
    }

    // MARK: - Profile Progress

    private func computeProfileProgress() async {
        guard let user = authManager.currentUser else { return }
        profileProgress.hasUsername = !user.username.isEmpty && user.username != user.email
        profileProgress.hasDisplayName = user.displayName != nil && !(user.displayName ?? "").isEmpty
        profileProgress.hasReminders = user.reminderFreq != "never"

        if let items = try? await ItemService.shared.listItems(page: 1, perPage: 1) {
            profileProgress.hasFirstItem = items.total > 0
        }
        if let circles = try? await CircleService.shared.listCircles() {
            profileProgress.hasFirstCircle = !circles.isEmpty
        }
        if let friends = try? await FriendService.shared.listFriends() {
            profileProgress.hasFirstFriend = !friends.isEmpty
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

    // MARK: - Avatar Upload

    private func loadAvatarImage(_ item: PhotosPickerItem) async {
        guard let data = try? await item.loadTransferable(type: Data.self),
              let image = UIImage(data: data) else { return }
        selectedAvatarImage = image
        await uploadAvatar(image)
    }

    private func uploadAvatar(_ image: UIImage) async {
        guard let data = image.compressForUpload() else { return }
        isUploadingAvatar = true
        do {
            let url = try await APIClient.shared.uploadImage(data, type: "avatar")
            let body = UpdateProfileBody(
                displayName: nil, username: nil, avatarUrl: url,
                reminderFreq: nil, reminderTime: nil, timezone: nil, locale: nil
            )
            _ = try await APIClient.shared.request(.updateProfile(body)) as UserProfileResponse
            viewModel.avatarUrlString = url
            try? await authManager.loadCurrentUser()
            selectedAvatarImage = nil
        } catch {
            // Revert selection on error
            selectedAvatarImage = nil
        }
        isUploadingAvatar = false
    }
}
