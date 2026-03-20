import SwiftUI
import UserNotifications

// MARK: - ProfileProgressSheet

struct ProfileProgressSheet: View {
    let progress: ProfileProgress
    @Environment(\.dismiss) private var dismiss
    @Environment(AuthManager.self) private var authManager
    @Environment(AppRouter.self) private var router

    // Sheet states for each action
    @State private var showAddWish = false
    @State private var showCreateCircle = false
    @State private var showAddFriend = false
    @State private var showShareSheet = false
    @State private var showAvatarSourceSheet = false
    @State private var showAvatarCamera = false
    @State private var showAvatarPhotoPicker = false
    @State private var selectedAvatarImage: UIImage?
    @State private var isUploadingAvatar = false

    private var groupedSteps: [(ProfileProgressStep.StepGroup, [ProfileProgressStep])] {
        var groups: [(ProfileProgressStep.StepGroup, [ProfileProgressStep])] = []
        for group in ProfileProgressStep.StepGroup.allCases {
            let stepsInGroup = progress.steps.filter { $0.group == group }
            if !stepsInGroup.isEmpty {
                groups.append((group, stepsInGroup))
            }
        }
        return groups
    }

    var body: some View {
        NavigationStack {
            ScrollView {
                VStack(spacing: OffriiTheme.spacingLG) {
                    // Hero progress
                    VStack(spacing: OffriiTheme.spacingSM) {
                        ZStack {
                            Circle()
                                .stroke(OffriiTheme.border, lineWidth: 8)
                                .frame(width: 100, height: 100)

                            Circle()
                                .trim(from: 0, to: CGFloat(progress.percentage) / 100)
                                .stroke(
                                    LinearGradient(
                                        colors: [OffriiTheme.primary, OffriiTheme.primaryLight],
                                        startPoint: .topLeading,
                                        endPoint: .bottomTrailing
                                    ),
                                    style: StrokeStyle(lineWidth: 8, lineCap: .round)
                                )
                                .frame(width: 100, height: 100)
                                .rotationEffect(.degrees(-90))
                                .animation(OffriiAnimation.defaultSpring, value: progress.percentage)

                            Text("\(progress.percentage)%")
                                .font(.system(size: 28, weight: .bold))
                                .foregroundColor(OffriiTheme.text)
                        }

                        Text("\(progress.completedCount) \(NSLocalizedString("progress.detail.outOf", comment: "")) \(progress.totalCount)")
                            .font(OffriiTypography.subheadline)
                            .foregroundColor(OffriiTheme.textSecondary)
                    }
                    .padding(.top, OffriiTheme.spacingBase)

                    // Grouped steps
                    ForEach(groupedSteps, id: \.0) { group, steps in
                        VStack(alignment: .leading, spacing: OffriiTheme.spacingSM) {
                            HStack(spacing: 6) {
                                Image(systemName: group.icon)
                                    .font(.system(size: 12))
                                    .foregroundColor(OffriiTheme.primary)
                                Text(NSLocalizedString(group.titleKey, comment: ""))
                                    .font(.system(size: 13, weight: .semibold))
                                    .foregroundColor(OffriiTheme.textSecondary)
                                    .textCase(.uppercase)
                            }

                            VStack(spacing: 0) {
                                ForEach(Array(steps.enumerated()), id: \.element.id) { index, step in
                                    stepRow(step)

                                    if index < steps.count - 1 {
                                        Divider()
                                            .padding(.leading, 52)
                                    }
                                }
                            }
                            .padding(OffriiTheme.spacingSM)
                            .background(OffriiTheme.card)
                            .cornerRadius(OffriiTheme.cornerRadiusLG)
                            .shadow(
                                color: OffriiTheme.cardShadowColor,
                                radius: OffriiTheme.cardShadowRadius,
                                x: 0,
                                y: OffriiTheme.cardShadowY
                            )
                        }
                    }
                }
                .padding(.horizontal, OffriiTheme.spacingBase)
                .padding(.bottom, OffriiTheme.spacingXXL)
            }
            .background(OffriiTheme.background.ignoresSafeArea())
            .navigationTitle(NSLocalizedString("progress.detail.title", comment: ""))
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button(NSLocalizedString("common.ok", comment: "")) {
                        dismiss()
                    }
                }
            }
        }
        .sheet(isPresented: $showAddWish) {
            QuickAddSheet { name, price, categoryId, priority, imageUrl, links, isPrivate in
                _ = try? await ItemService.shared.createItem(
                    name: name, estimatedPrice: price, priority: priority,
                    categoryId: categoryId, imageUrl: imageUrl, links: links,
                    isPrivate: isPrivate
                )
                return true
            }
        }
        .sheet(isPresented: $showCreateCircle) {
            CreateCircleSheet { _ in }
                .presentationDetents([.medium])
        }
        .sheet(isPresented: $showAddFriend) {
            AddFriendSheet {}
        }
        .sheet(isPresented: $showShareSheet) {
            WishlistShareSheet(
                items: [],
                selectedItemIds: [],
                categories: [],
                privateItemCount: 0
            )
            .presentationDetents([.large])
        }
        .sheet(isPresented: $showPublishNeed) {
            CreateWishSheet()
                .presentationDetents([.large])
        }
        .alert(
            NSLocalizedString("entraide.wishLimit.title", comment: ""),
            isPresented: $showWishLimitAlert
        ) {
            Button(NSLocalizedString("common.ok", comment: ""), role: .cancel) {}
        } message: {
            Text(NSLocalizedString("entraide.wishLimit.message", comment: ""))
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
                        Task { await loadAndUploadAvatar(item) }
                    }
                }
            ),
            matching: .images
        )
    }

    // MARK: - Step Row

    private func stepRow(_ step: ProfileProgressStep) -> some View {
        Button {
            if !step.isCompleted {
                handleStepTap(step)
            }
        } label: {
            HStack(spacing: OffriiTheme.spacingSM) {
                // Status icon
                if step.isCompleted {
                    Image(systemName: "checkmark.circle.fill")
                        .font(.system(size: 22))
                        .foregroundColor(OffriiTheme.primary)
                } else {
                    Image(systemName: step.icon)
                        .font(.system(size: 14))
                        .foregroundColor(OffriiTheme.primary)
                        .frame(width: 22, height: 22)
                        .background(OffriiTheme.primary.opacity(0.12))
                        .clipShape(Circle())
                }

                VStack(alignment: .leading, spacing: 2) {
                    Text(NSLocalizedString(step.titleKey, comment: ""))
                        .font(.system(size: 15, weight: step.isCompleted ? .regular : .medium))
                        .foregroundColor(step.isCompleted ? OffriiTheme.textMuted : OffriiTheme.text)
                        .strikethrough(step.isCompleted, color: OffriiTheme.textMuted)

                    if !step.isCompleted {
                        Text(NSLocalizedString(step.subtitleKey, comment: ""))
                            .font(.system(size: 12))
                            .foregroundColor(OffriiTheme.textSecondary)
                    }
                }

                Spacer()

                if !step.isCompleted {
                    Image(systemName: "chevron.right")
                        .font(.system(size: 11, weight: .semibold))
                        .foregroundColor(OffriiTheme.textMuted)
                }
            }
            .padding(.vertical, OffriiTheme.spacingXS)
            .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
        .disabled(step.isCompleted)
    }

    // MARK: - Step Actions

    @State private var showPublishNeed = false
    @State private var showWishLimitAlert = false

    private func handleStepTap(_ step: ProfileProgressStep) {
        switch step.id {
        case "displayName", "username":
            dismiss()
        case "avatar":
            showAvatarSourceSheet = true
        case "firstItem":
            showAddWish = true
        case "shareList":
            showShareSheet = true
        case "firstFriend":
            showAddFriend = true
        case "firstCircle":
            showCreateCircle = true
        case "pushNotifications":
            Task {
                let center = UNUserNotificationCenter.current()
                let settings = await center.notificationSettings()
                if settings.authorizationStatus == .notDetermined {
                    let granted = (try? await center.requestAuthorization(options: [.alert, .badge, .sound])) ?? false
                    if granted {
                        await MainActor.run { UIApplication.shared.registerForRemoteNotifications() }
                    }
                } else {
                    if let url = URL(string: UIApplication.openSettingsURLString) {
                        await MainActor.run { UIApplication.shared.open(url) }
                    }
                }
            }
        case "emailVerified":
            Task {
                try? await UserService.shared.resendVerification()
                OffriiHaptics.success()
            }
        case "firstNeed":
            router.selectedTab = .entraide
            dismiss()
        default:
            break
        }
    }

    // MARK: - Avatar Upload

    private func loadAndUploadAvatar(_ item: PhotosPickerItem) async {
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
            try? await authManager.loadCurrentUser()
            selectedAvatarImage = nil
            OffriiHaptics.success()
        } catch {
            selectedAvatarImage = nil
        }
        isUploadingAvatar = false
    }
}

// MARK: - PhotosPickerItem Identifiable (needed for .photosPicker binding)

import PhotosUI
