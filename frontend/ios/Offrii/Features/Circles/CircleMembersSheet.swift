import SwiftUI

struct CircleMembersSheet: View {
    @Environment(\.dismiss) private var dismiss
    let circleId: UUID
    let members: [CircleMember]
    let ownerId: UUID
    let onLeft: () -> Void

    @State private var isLeaving = false
    @State private var showLeaveConfirm = false
    let currentUserId: UUID?

    var body: some View {
        NavigationStack {
            ZStack {
                OffriiTheme.cardSurface.ignoresSafeArea()

                VStack(spacing: 0) {
                    List {
                        ForEach(members) { member in
                            HStack(spacing: OffriiTheme.spacingSM) {
                                AvatarView(member.displayName ?? member.username, size: .small)

                                VStack(alignment: .leading, spacing: 2) {
                                    HStack(spacing: OffriiTheme.spacingXS) {
                                        Text(member.displayName ?? member.username)
                                            .font(OffriiTypography.body)
                                            .foregroundColor(OffriiTheme.text)

                                        if member.userId == currentUserId {
                                            Text(NSLocalizedString("circles.members.you", comment: ""))
                                                .font(OffriiTypography.caption)
                                                .foregroundColor(OffriiTheme.primary)
                                        }
                                    }

                                    Text("@\(member.username)")
                                        .font(OffriiTypography.caption)
                                        .foregroundColor(OffriiTheme.textMuted)
                                }

                                Spacer()

                                if member.role == "owner" {
                                    Text(NSLocalizedString("circles.members.owner", comment: ""))
                                        .font(OffriiTypography.caption)
                                        .foregroundColor(OffriiTheme.accent)
                                }
                            }
                            .padding(.vertical, OffriiTheme.spacingXS)
                        }
                    }
                    .listStyle(.plain)

                    if let userId = currentUserId, userId != ownerId {
                        Button {
                            showLeaveConfirm = true
                        } label: {
                            HStack {
                                Spacer()
                                if isLeaving {
                                    ProgressView()
                                        .tint(.white)
                                } else {
                                    Text(NSLocalizedString("circles.members.leave", comment: ""))
                                        .font(OffriiTypography.body)
                                        .fontWeight(.semibold)
                                }
                                Spacer()
                            }
                            .foregroundColor(.white)
                            .padding(.vertical, OffriiTheme.spacingSM)
                            .background(OffriiTheme.danger)
                            .cornerRadius(OffriiTheme.cornerRadiusSM)
                        }
                        .disabled(isLeaving)
                        .padding(OffriiTheme.spacingLG)
                    }
                }
            }
            .navigationTitle(NSLocalizedString("circles.members.title", comment: ""))
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .confirmationAction) {
                    Button(NSLocalizedString("common.ok", comment: "")) {
                        dismiss()
                    }
                }
            }
            .alert(
                NSLocalizedString("circles.members.leaveConfirm.title", comment: ""),
                isPresented: $showLeaveConfirm
            ) {
                Button(NSLocalizedString("circles.members.leave", comment: ""), role: .destructive) {
                    Task { await leaveCircle() }
                }
                Button(NSLocalizedString("common.cancel", comment: ""), role: .cancel) {}
            } message: {
                Text(NSLocalizedString("circles.members.leaveConfirm.message", comment: ""))
            }
        }
    }

    private func leaveCircle() async {
        guard let userId = currentUserId else { return }
        isLeaving = true
        do {
            try await CircleService.shared.removeMember(circleId: circleId, userId: userId)
            onLeft()
            dismiss()
        } catch {
            // Silently fail — user can retry
        }
        isLeaving = false
    }
}
