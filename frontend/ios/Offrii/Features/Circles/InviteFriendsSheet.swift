import SwiftUI

struct InviteFriendsSheet: View {
    @Environment(\.dismiss) private var dismiss
    let circleId: UUID
    let existingMemberIds: Set<UUID>
    let onInvited: () -> Void

    @State private var friends: [FriendResponse] = []
    @State private var selected: Set<UUID> = []
    @State private var isLoading = false
    @State private var isInviting = false
    @State private var error: String?

    // Invite link
    @State private var invites: [CircleInviteResponse] = []
    @State private var isCreatingInvite = false
    @State private var copiedInviteId: UUID?

    var body: some View {
        NavigationStack {
            ZStack {
                OffriiTheme.background.ignoresSafeArea()

                if isLoading {
                    SkeletonList(count: 5)
                        .padding(.top, OffriiTheme.spacingBase)
                } else {
                    ScrollView {
                        VStack(alignment: .leading, spacing: OffriiTheme.spacingLG) {
                            // Section 1: Invite link
                            inviteLinkSection

                            if !friends.isEmpty {
                                Divider().padding(.horizontal, OffriiTheme.spacingLG)

                                // Section 2: Add friends directly
                                friendsSection
                            }
                        }
                        .padding(.vertical, OffriiTheme.spacingBase)
                    }
                }
            }
            .navigationTitle(NSLocalizedString("circles.invite.title", comment: ""))
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button(NSLocalizedString("common.cancel", comment: "")) { dismiss() }
                }
            }
            .safeAreaInset(edge: .bottom) {
                if !selected.isEmpty {
                    OffriiButton(
                        String(format: NSLocalizedString("circles.invite.addCount", comment: ""), selected.count),
                        isLoading: isInviting
                    ) {
                        Task { await inviteSelected() }
                    }
                    .padding(.horizontal, OffriiTheme.spacingLG)
                    .padding(.bottom, OffriiTheme.spacingSM)
                    .background(OffriiTheme.background)
                }
            }
            .task {
                await loadData()
            }
            .alert(
                NSLocalizedString("common.error", comment: ""),
                isPresented: Binding(
                    get: { error != nil },
                    set: { if !$0 { error = nil } }
                )
            ) {
                Button(NSLocalizedString("common.ok", comment: ""), role: .cancel) {}
            } message: {
                if let error { Text(error) }
            }
        }
    }

    // MARK: - Invite Link Section

    private var inviteLinkSection: some View {
        VStack(alignment: .leading, spacing: OffriiTheme.spacingSM) {
            Text(NSLocalizedString("circles.invite.linkTitle", comment: ""))
                .font(OffriiTypography.headline)
                .foregroundColor(OffriiTheme.text)
                .padding(.horizontal, OffriiTheme.spacingLG)

            Text(NSLocalizedString("circles.invite.linkSubtitle", comment: ""))
                .font(OffriiTypography.caption)
                .foregroundColor(OffriiTheme.textMuted)
                .padding(.horizontal, OffriiTheme.spacingLG)

            // Active invites
            ForEach(invites) { invite in
                inviteCard(invite)
            }

            // Create button
            Button {
                Task { await createInvite() }
            } label: {
                HStack(spacing: OffriiTheme.spacingSM) {
                    if isCreatingInvite {
                        ProgressView().tint(OffriiTheme.primary)
                    } else {
                        Image(systemName: "link.badge.plus")
                    }
                    Text(NSLocalizedString("circles.invite.createLink", comment: ""))
                }
                .font(.system(size: 14, weight: .medium))
                .foregroundColor(OffriiTheme.primary)
                .frame(maxWidth: .infinity)
                .padding(.vertical, 12)
                .background(OffriiTheme.primary.opacity(0.08))
                .cornerRadius(OffriiTheme.cornerRadiusMD)
            }
            .disabled(isCreatingInvite)
            .padding(.horizontal, OffriiTheme.spacingLG)
        }
    }

    private func inviteCard(_ invite: CircleInviteResponse) -> some View {
        let isCopied = copiedInviteId == invite.id
        let inviteUrl = "https://offrii.com/join/\(invite.token)"

        return HStack(spacing: OffriiTheme.spacingSM) {
            VStack(alignment: .leading, spacing: 2) {
                Text(inviteUrl)
                    .font(.system(size: 12, weight: .medium))
                    .foregroundColor(OffriiTheme.primary)
                    .lineLimit(1)

                HStack(spacing: 4) {
                    Text("\(invite.useCount)/\(invite.maxUses)")
                        .font(.system(size: 10))
                    Text("\u{00B7}")
                    Text(invite.expiresAt, style: .relative)
                        .font(.system(size: 10))
                }
                .foregroundColor(OffriiTheme.textMuted)
            }

            Spacer()

            Button {
                UIPasteboard.general.string = inviteUrl
                OffriiHaptics.success()
                copiedInviteId = invite.id
                DispatchQueue.main.asyncAfter(deadline: .now() + 2) {
                    if copiedInviteId == invite.id { copiedInviteId = nil }
                }
            } label: {
                Image(systemName: isCopied ? "checkmark" : "doc.on.doc")
                    .font(.system(size: 14))
                    .foregroundColor(isCopied ? OffriiTheme.success : OffriiTheme.primary)
            }

            Button {
                shareInvite(url: inviteUrl)
            } label: {
                Image(systemName: "square.and.arrow.up")
                    .font(.system(size: 14))
                    .foregroundColor(OffriiTheme.primary)
            }

            Button {
                Task { await revokeInvite(invite) }
            } label: {
                Image(systemName: "trash")
                    .font(.system(size: 14))
                    .foregroundColor(OffriiTheme.danger)
            }
        }
        .padding(OffriiTheme.spacingBase)
        .background(OffriiTheme.surface)
        .cornerRadius(OffriiTheme.cornerRadiusMD)
        .padding(.horizontal, OffriiTheme.spacingLG)
    }

    // MARK: - Friends Section

    private var friendsSection: some View {
        VStack(alignment: .leading, spacing: OffriiTheme.spacingSM) {
            Text(NSLocalizedString("circles.invite.friendsTitle", comment: ""))
                .font(OffriiTypography.headline)
                .foregroundColor(OffriiTheme.text)
                .padding(.horizontal, OffriiTheme.spacingLG)

            ForEach(friends) { friend in
                let alreadyMember = existingMemberIds.contains(friend.userId)
                Button {
                    if !alreadyMember { toggleSelection(friend.userId) }
                } label: {
                    HStack(spacing: OffriiTheme.spacingSM) {
                        AvatarView(friend.displayName ?? friend.username, size: .small)

                        VStack(alignment: .leading, spacing: 2) {
                            Text(friend.displayName ?? friend.username)
                                .font(OffriiTypography.body)
                                .foregroundColor(alreadyMember ? OffriiTheme.textMuted : OffriiTheme.text)
                            Text("@\(friend.username)")
                                .font(OffriiTypography.caption)
                                .foregroundColor(OffriiTheme.textMuted)
                        }

                        Spacer()

                        if alreadyMember {
                            Text(NSLocalizedString("circles.invite.alreadyMember", comment: ""))
                                .font(OffriiTypography.caption)
                                .foregroundColor(OffriiTheme.textMuted)
                        } else {
                            Image(systemName: selected.contains(friend.userId)
                                  ? "checkmark.circle.fill" : "circle")
                                .foregroundColor(selected.contains(friend.userId)
                                                 ? OffriiTheme.primary : OffriiTheme.textMuted)
                                .font(.system(size: 22))
                        }
                    }
                    .padding(.horizontal, OffriiTheme.spacingLG)
                    .padding(.vertical, OffriiTheme.spacingSM)
                }
                .disabled(alreadyMember)
                .buttonStyle(.plain)
            }
        }
    }

    // MARK: - Actions

    private func toggleSelection(_ userId: UUID) {
        if selected.contains(userId) {
            selected.remove(userId)
        } else {
            selected.insert(userId)
        }
    }

    private func loadData() async {
        isLoading = true
        async let friendsLoad: [FriendResponse] = {
            (try? await FriendService.shared.listFriends()) ?? []
        }()
        async let invitesLoad: [CircleInviteResponse] = {
            (try? await CircleService.shared.listInvites(circleId: circleId)) ?? []
        }()
        friends = await friendsLoad
        invites = await invitesLoad
        isLoading = false
    }

    private func createInvite() async {
        isCreatingInvite = true
        do {
            let invite = try await CircleService.shared.createInvite(circleId: circleId)
            withAnimation { invites.insert(invite, at: 0) }
            let url = "https://offrii.com/join/\(invite.token)"
            UIPasteboard.general.string = url
            OffriiHaptics.success()
            copiedInviteId = invite.id
        } catch {
            self.error = error.localizedDescription
        }
        isCreatingInvite = false
    }

    private func revokeInvite(_ invite: CircleInviteResponse) async {
        do {
            try await CircleService.shared.revokeInvite(circleId: circleId, inviteId: invite.id)
            withAnimation { invites.removeAll { $0.id == invite.id } }
            OffriiHaptics.success()
        } catch {
            self.error = error.localizedDescription
        }
    }

    private func inviteSelected() async {
        isInviting = true
        var succeeded: Set<UUID> = []

        for userId in selected {
            do {
                try await CircleService.shared.addMember(circleId: circleId, userId: userId)
                succeeded.insert(userId)
            } catch {
                self.error = error.localizedDescription
            }
        }

        selected.subtract(succeeded)
        isInviting = false

        if selected.isEmpty {
            onInvited()
            dismiss()
        } else if !succeeded.isEmpty {
            onInvited()
        }
    }

    private func shareInvite(url: String) {
        guard let shareUrl = URL(string: url) else { return }
        let activityVC = UIActivityViewController(activityItems: [shareUrl], applicationActivities: nil)
        if let windowScene = UIApplication.shared.connectedScenes.first as? UIWindowScene,
           let rootVC = windowScene.windows.first?.rootViewController {
            rootVC.present(activityVC, animated: true)
        }
    }
}
