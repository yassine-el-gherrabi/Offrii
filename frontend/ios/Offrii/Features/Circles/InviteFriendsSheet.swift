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

    var body: some View {
        NavigationStack {
            ZStack {
                OffriiTheme.cardSurface.ignoresSafeArea()

                if isLoading {
                    ProgressView()
                } else if friends.isEmpty {
                    VStack(spacing: OffriiTheme.spacingMD) {
                        Image(systemName: "person.2.slash")
                            .font(.system(size: 40))
                            .foregroundColor(OffriiTheme.textMuted)
                        Text(NSLocalizedString("circles.invite.noFriends", comment: ""))
                            .font(OffriiTypography.body)
                            .foregroundColor(OffriiTheme.textSecondary)
                            .multilineTextAlignment(.center)
                    }
                    .padding(OffriiTheme.spacingXL)
                } else {
                    VStack(spacing: 0) {
                        List {
                            ForEach(friends) { friend in
                                let alreadyMember = existingMemberIds.contains(friend.userId)
                                Button {
                                    if !alreadyMember {
                                        toggleSelection(friend.userId)
                                    }
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
                                                  ? "checkmark.circle.fill"
                                                  : "circle")
                                                .foregroundColor(selected.contains(friend.userId)
                                                                 ? OffriiTheme.primary
                                                                 : OffriiTheme.textMuted)
                                                .font(.system(size: 22))
                                        }
                                    }
                                }
                                .disabled(alreadyMember)
                            }
                        }
                        .listStyle(.plain)

                        if !selected.isEmpty {
                            OffriiButton(
                                String(format: NSLocalizedString("circles.invite.addCount", comment: ""), selected.count),
                                isLoading: isInviting
                            ) {
                                Task { await inviteSelected() }
                            }
                            .padding(OffriiTheme.spacingLG)
                        }
                    }
                }
            }
            .navigationTitle(NSLocalizedString("circles.invite.title", comment: ""))
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button(NSLocalizedString("common.cancel", comment: "")) {
                        dismiss()
                    }
                }
            }
            .task {
                await loadFriends()
            }
        }
    }

    private func toggleSelection(_ userId: UUID) {
        if selected.contains(userId) {
            selected.remove(userId)
        } else {
            selected.insert(userId)
        }
    }

    private func loadFriends() async {
        isLoading = true
        do {
            friends = try await FriendService.shared.listFriends()
        } catch {
            self.error = error.localizedDescription
        }
        isLoading = false
    }

    private func inviteSelected() async {
        isInviting = true
        for userId in selected {
            do {
                try await CircleService.shared.addMember(circleId: circleId, userId: userId)
            } catch {
                self.error = error.localizedDescription
            }
        }
        isInviting = false
        onInvited()
        dismiss()
    }
}
