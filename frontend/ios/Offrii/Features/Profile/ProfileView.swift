// swiftlint:disable file_length
import PhotosUI
import SwiftUI
import UserNotifications

// swiftlint:disable:next type_body_length
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
    @State private var pushEnabled = false
    @State private var isResendingVerification = false

    var body: some View {
        ZStack {
            OffriiTheme.background.ignoresSafeArea()

            ScrollView {
                VStack(spacing: OffriiTheme.spacingLG) {
                    // Error banner
                    if let error = viewModel.loadError {
                        errorBanner(error)
                    }

                    // Section 1: Hero Identity
                    heroIdentitySection

                    // Section 2: Stats Row
                    statsRow

                    // Section 3: Profile Progress
                    if profileProgress.percentage < 100 {
                        ProfileProgressCard(progress: profileProgress)
                            .padding(.horizontal, OffriiTheme.spacingLG)
                    }

                    // Section 4: Mon compte
                    profileSection(
                        title: NSLocalizedString("profile.myAccount", comment: ""),
                        icon: "person.fill"
                    ) {
                        VStack(spacing: 0) {
                            // Display name
                            Button {
                                viewModel.editedDisplayName = viewModel.displayName
                                viewModel.isEditingDisplayName = true
                            } label: {
                                profileRow(
                                    title: NSLocalizedString("profile.displayName", comment: ""),
                                    value: viewModel.displayName.isEmpty ? nil : viewModel.displayName
                                )
                            }

                            Divider().padding(.leading, OffriiTheme.spacingBase)

                            // Username
                            NavigationLink {
                                UsernameEditView(viewModel: viewModel)
                            } label: {
                                profileRow(
                                    title: NSLocalizedString("profile.username", comment: ""),
                                    value: viewModel.username.isEmpty ? nil : "@\(viewModel.username)"
                                )
                            }

                            Divider().padding(.leading, OffriiTheme.spacingBase)

                            // Email (display only)
                            profileRow(
                                title: NSLocalizedString("profile.email", comment: ""),
                                value: viewModel.truncatedEmail,
                                showChevron: false
                            )

                            Divider().padding(.leading, OffriiTheme.spacingBase)

                            // Profile photo
                            Button {
                                showAvatarSourceSheet = true
                            } label: {
                                profileRow(
                                    title: NSLocalizedString("profile.photo", comment: ""),
                                    value: nil
                                )
                            }
                        }
                    }

                    // Section 5: Préférences
                    profileSection(
                        title: NSLocalizedString("profile.preferences", comment: ""),
                        icon: "gearshape.fill"
                    ) {
                        VStack(spacing: 0) {
                            // Reminders
                            NavigationLink {
                                ReminderSettingsView()
                                    .environment(authManager)
                            } label: {
                                profileRow(
                                    title: NSLocalizedString("profile.reminders", comment: ""),
                                    value: viewModel.reminderFreqLabel
                                )
                            }

                            Divider().padding(.leading, OffriiTheme.spacingBase)

                            // Push notifications
                            Button {
                                handlePushNotificationTap()
                            } label: {
                                profileRow(
                                    title: pushEnabled
                                        ? NSLocalizedString("profile.notifications.enabled", comment: "")
                                        : NSLocalizedString("profile.notifications.openSettings", comment: ""),
                                    value: nil
                                )
                            }
                        }
                    }

                    // Section 6: Mes réservations
                    profileSection(
                        title: NSLocalizedString("profile.myReservations", comment: ""),
                        icon: "gift.fill"
                    ) {
                        NavigationLink {
                            ReservationsListView()
                                .navigationTitle(NSLocalizedString("profile.myReservations", comment: ""))
                                .navigationBarTitleDisplayMode(.inline)
                        } label: {
                            profileRow(
                                title: NSLocalizedString("profile.myReservations", comment: ""),
                                value: nil
                            )
                        }
                    }

                    // Section 7: Données & confidentialité
                    profileSection(
                        title: NSLocalizedString("profile.dataPrivacy", comment: ""),
                        icon: "lock.shield.fill"
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

                            Divider().padding(.leading, OffriiTheme.spacingBase)

                            NavigationLink {
                                DataManagementView()
                                    .environment(authManager)
                            } label: {
                                HStack {
                                    Text(NSLocalizedString("profile.deleteAccount", comment: ""))
                                        .font(OffriiTypography.body)
                                        .foregroundColor(OffriiTheme.danger)
                                    Spacer()
                                    Image(systemName: "chevron.right")
                                        .font(.system(size: 12, weight: .semibold))
                                        .foregroundColor(OffriiTheme.textMuted)
                                }
                                .padding(.vertical, OffriiTheme.spacingSM)
                            }
                        }
                    }

                    // Section 8: Logout + version
                    VStack(spacing: OffriiTheme.spacingBase) {
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

                        Text(String(format: NSLocalizedString("profile.version", comment: ""), viewModel.appVersion))
                            .font(OffriiTypography.caption)
                            .foregroundColor(OffriiTheme.textMuted)
                            .frame(maxWidth: .infinity, alignment: .center)
                    }
                    .padding(.horizontal, OffriiTheme.spacingLG)
                    .padding(.top, OffriiTheme.spacingSM)
                }
                .padding(.top, OffriiTheme.spacingBase)
                .padding(.bottom, OffriiTheme.spacingXXL)
            }
        }
        .navigationTitle(NSLocalizedString("profile.navTitle", comment: ""))
        .navigationBarTitleDisplayMode(.inline)
        .alert(
            NSLocalizedString("profile.displayName.edit", comment: ""),
            isPresented: $viewModel.isEditingDisplayName
        ) {
            TextField(
                NSLocalizedString("profile.displayName", comment: ""),
                text: $viewModel.editedDisplayName
            )
            Button(NSLocalizedString("common.cancel", comment: ""), role: .cancel) {}
            Button(NSLocalizedString("common.save", comment: "")) {
                Task { await saveDisplayName() }
            }
        }
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
            if viewModel.avatarUrl != nil || selectedAvatarImage != nil {
                Button(NSLocalizedString("imagePicker.removePhoto", comment: ""), role: .destructive) {
                    Task { await removeAvatar() }
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
            await refreshPushStatus()
        }
        .onReceive(NotificationCenter.default.publisher(for: UIApplication.willEnterForegroundNotification)) { _ in
            Task { await refreshPushStatus() }
        }
    }

    // MARK: - Hero Identity Section

    @ViewBuilder
    private var heroIdentitySection: some View {
        OffriiCard {
            VStack(spacing: OffriiTheme.spacingSM) {
                // Avatar
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

                // Display name (tappable)
                Button {
                    viewModel.editedDisplayName = viewModel.displayName
                    viewModel.isEditingDisplayName = true
                } label: {
                    Text(viewModel.displayName.isEmpty ? viewModel.email : viewModel.displayName)
                        .font(OffriiTypography.title)
                        .fontWeight(.bold)
                        .foregroundColor(OffriiTheme.text)
                        .multilineTextAlignment(.center)
                }

                // @username
                if !viewModel.username.isEmpty {
                    Text("@\(viewModel.username)")
                        .font(OffriiTypography.subheadline)
                        .foregroundColor(OffriiTheme.textSecondary)
                }

                // Email + verification badge
                emailVerificationRow

                // Member since
                if !viewModel.memberSinceText.isEmpty {
                    Text(viewModel.memberSinceText)
                        .font(OffriiTypography.caption)
                        .foregroundColor(OffriiTheme.textMuted)
                }
            }
            .frame(maxWidth: .infinity)
        }
        .padding(.horizontal, OffriiTheme.spacingLG)
    }

    // MARK: - Email Verification Row

    @ViewBuilder
    private var emailVerificationRow: some View {
        let isVerified = authManager.currentUser?.emailVerified == true

        if isVerified {
            HStack(spacing: 4) {
                Text(viewModel.truncatedEmail)
                    .font(OffriiTypography.caption)
                    .foregroundColor(OffriiTheme.textSecondary)
                Image(systemName: "checkmark.seal.fill")
                    .font(.system(size: 12))
                    .foregroundColor(OffriiTheme.success)
            }
        } else {
            Button {
                Task { await resendVerification() }
            } label: {
                HStack(spacing: 4) {
                    Text(viewModel.truncatedEmail)
                        .font(OffriiTypography.caption)
                        .foregroundColor(OffriiTheme.textSecondary)
                    Text("(\(NSLocalizedString("profile.emailNotVerified", comment: "")))")
                        .font(OffriiTypography.caption)
                        .foregroundColor(.orange)
                }
            }
            .disabled(isResendingVerification)
        }
    }

    // MARK: - Stats Row

    @ViewBuilder
    private var statsRow: some View {
        ZStack {
            ScrollView(.horizontal, showsIndicators: false) {
                HStack(spacing: OffriiTheme.spacingSM) {
                    statChip(
                        icon: "heart.fill",
                        value: viewModel.totalItems,
                        label: NSLocalizedString("profile.stats.wishes", comment: "")
                    )
                    statChip(
                        icon: "gift.fill",
                        value: viewModel.receivedItems,
                        label: NSLocalizedString("profile.stats.received", comment: "")
                    )
                    statChip(
                        icon: "person.2.fill",
                        value: viewModel.circlesCount,
                        label: NSLocalizedString("profile.stats.circles", comment: "")
                    )
                    statChip(
                        icon: "person.fill",
                        value: viewModel.friendsCount,
                        label: NSLocalizedString("profile.stats.friends", comment: "")
                    )
                }
                .padding(.horizontal, OffriiTheme.spacingLG + 2)
            }

            // Fade hints on both edges
            HStack {
                LinearGradient(
                    colors: [OffriiTheme.background, OffriiTheme.background.opacity(0)],
                    startPoint: .leading,
                    endPoint: .trailing
                )
                .frame(width: 12)

                Spacer()

                LinearGradient(
                    colors: [OffriiTheme.background.opacity(0), OffriiTheme.background],
                    startPoint: .leading,
                    endPoint: .trailing
                )
                .frame(width: 20)
            }
            .allowsHitTesting(false)
        }
    }

    private func statChip(icon: String, value: Int, label: String) -> some View {
        HStack(spacing: 6) {
            Image(systemName: icon)
                .font(.system(size: 12))
                .foregroundColor(OffriiTheme.primary)

            Text("\(value)")
                .font(.system(size: 15, weight: .bold))
                .foregroundColor(OffriiTheme.primary)

            Text(label.lowercased())
                .font(.system(size: 13))
                .foregroundColor(OffriiTheme.textSecondary)
        }
        .padding(.horizontal, OffriiTheme.spacingMD)
        .padding(.vertical, OffriiTheme.spacingSM)
        .background(OffriiTheme.primary.opacity(0.08))
        .cornerRadius(OffriiTheme.cornerRadiusFull)
    }

    // MARK: - Profile Progress

    private func computeProfileProgress() async {
        guard let user = authManager.currentUser else { return }
        profileProgress.update(id: "username", completed: !user.username.isEmpty && user.username != user.email)
        profileProgress.update(id: "displayName", completed: user.displayName != nil && !(user.displayName ?? "").isEmpty)
        profileProgress.update(id: "avatar", completed: user.avatarUrl != nil && !(user.avatarUrl ?? "").isEmpty)
        profileProgress.update(id: "reminders", completed: user.reminderFreq != "never")

        if let items = try? await ItemService.shared.listItems(page: 1, perPage: 1) {
            profileProgress.update(id: "firstItem", completed: items.total > 0)
        }
        if let circles = try? await CircleService.shared.listCircles() {
            profileProgress.update(id: "firstCircle", completed: !circles.isEmpty)
        }
        if let friends = try? await FriendService.shared.listFriends() {
            profileProgress.update(id: "firstFriend", completed: !friends.isEmpty)
        }
        if let rules = try? await CircleService.shared.listMyShareRules() {
            profileProgress.update(id: "shareList", completed: rules.contains { $0.shareMode != "none" })
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

    private func profileRow(title: String, value: String?, showChevron: Bool = true) -> some View {
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
            if showChevron {
                Image(systemName: "chevron.right")
                    .font(.system(size: 12, weight: .semibold))
                    .foregroundColor(OffriiTheme.textMuted)
            }
        }
        .padding(.vertical, OffriiTheme.spacingSM)
    }

    @ViewBuilder
    private func errorBanner(_ error: String) -> some View {
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

    // MARK: - Display Name

    private func saveDisplayName() async {
        let trimmed = viewModel.editedDisplayName.trimmingCharacters(in: .whitespaces)
        guard !trimmed.isEmpty else { return }
        viewModel.isSavingDisplayName = true
        do {
            try await viewModel.updateDisplayName(trimmed)
            try? await authManager.loadCurrentUser()
        } catch {}
        viewModel.isSavingDisplayName = false
    }

    // MARK: - Email Verification

    private func resendVerification() async {
        isResendingVerification = true
        do {
            try await UserService.shared.resendVerification()
        } catch {}
        isResendingVerification = false
    }

    // MARK: - Push Notifications

    private func handlePushNotificationTap() {
        if pushEnabled {
            if let url = URL(string: UIApplication.openSettingsURLString) {
                UIApplication.shared.open(url)
            }
        } else {
            Task {
                let center = UNUserNotificationCenter.current()
                let settings = await center.notificationSettings()
                if settings.authorizationStatus == .notDetermined {
                    let granted = (try? await center.requestAuthorization(
                        options: [.alert, .badge, .sound]
                    )) ?? false
                    pushEnabled = granted
                    if granted {
                        await MainActor.run {
                            UIApplication.shared.registerForRemoteNotifications()
                        }
                    }
                } else {
                    if let url = URL(string: UIApplication.openSettingsURLString) {
                        await MainActor.run {
                            UIApplication.shared.open(url)
                        }
                    }
                }
            }
        }
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
                displayName: nil, username: nil, avatarUrl: .some(url),
                reminderFreq: nil, reminderTime: nil, timezone: nil, locale: nil
            )
            _ = try await APIClient.shared.request(.updateProfile(body)) as UserProfileResponse
            viewModel.avatarUrlString = url
            try? await authManager.loadCurrentUser()
            selectedAvatarImage = nil
        } catch {
            selectedAvatarImage = nil
        }
        isUploadingAvatar = false
    }

    private func removeAvatar() async {
        isUploadingAvatar = true
        do {
            let body = UpdateProfileBody(
                displayName: nil, username: nil, avatarUrl: .some(nil),
                reminderFreq: nil, reminderTime: nil, timezone: nil, locale: nil
            )
            _ = try await APIClient.shared.request(.updateProfile(body)) as UserProfileResponse
            viewModel.avatarUrlString = nil
            selectedAvatarImage = nil
            try? await authManager.loadCurrentUser()
        } catch {}
        isUploadingAvatar = false
    }

    private func refreshPushStatus() async {
        let center = UNUserNotificationCenter.current()
        let settings = await center.notificationSettings()
        let wasEnabled = pushEnabled
        pushEnabled = settings.authorizationStatus == .authorized
        if pushEnabled && !wasEnabled {
            await MainActor.run {
                UIApplication.shared.registerForRemoteNotifications()
            }
        }
    }
}
