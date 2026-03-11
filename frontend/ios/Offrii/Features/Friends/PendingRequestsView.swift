import SwiftUI

struct PendingRequestsView: View {
    @Environment(AuthManager.self) private var authManager
    @State private var requests: [FriendRequestResponse] = []
    @State private var isLoading = false
    @State private var error: String?

    var body: some View {
        ZStack {
            OffriiTheme.cardSurface.ignoresSafeArea()

            if isLoading && requests.isEmpty {
                SkeletonList(count: 3)
            } else if requests.isEmpty {
                VStack(spacing: OffriiTheme.spacingMD) {
                    Image(systemName: "bell.slash")
                        .font(.system(size: 40))
                        .foregroundColor(OffriiTheme.textMuted)
                    Text(NSLocalizedString("friends.pending.empty", comment: ""))
                        .font(OffriiTypography.body)
                        .foregroundColor(OffriiTheme.textSecondary)
                }
            } else {
                List {
                    ForEach(requests) { request in
                        HStack(spacing: OffriiTheme.spacingSM) {
                            AvatarView(request.fromDisplayName ?? request.fromUsername, size: .small)

                            VStack(alignment: .leading, spacing: 2) {
                                Text(request.fromDisplayName ?? request.fromUsername)
                                    .font(OffriiTypography.body)
                                    .foregroundColor(OffriiTheme.text)
                                Text("@\(request.fromUsername)")
                                    .font(OffriiTypography.caption)
                                    .foregroundColor(OffriiTheme.textMuted)
                            }

                            Spacer()

                            HStack(spacing: OffriiTheme.spacingSM) {
                                Button {
                                    Task { await acceptRequest(request) }
                                } label: {
                                    Text(NSLocalizedString("friends.accept", comment: ""))
                                        .font(OffriiTypography.footnote)
                                        .fontWeight(.semibold)
                                        .foregroundColor(.white)
                                        .padding(.horizontal, OffriiTheme.spacingSM)
                                        .padding(.vertical, OffriiTheme.spacingXS)
                                        .background(OffriiTheme.primary)
                                        .cornerRadius(OffriiTheme.cornerRadiusXL)
                                }

                                Button {
                                    Task { await declineRequest(request) }
                                } label: {
                                    Text(NSLocalizedString("friends.decline", comment: ""))
                                        .font(OffriiTypography.footnote)
                                        .foregroundColor(OffriiTheme.textSecondary)
                                        .padding(.horizontal, OffriiTheme.spacingSM)
                                        .padding(.vertical, OffriiTheme.spacingXS)
                                        .background(OffriiTheme.textMuted.opacity(0.15))
                                        .cornerRadius(OffriiTheme.cornerRadiusXL)
                                }
                            }
                        }
                        .padding(.vertical, OffriiTheme.spacingXS)
                    }
                }
                .listStyle(.plain)
            }
        }
        .navigationTitle(NSLocalizedString("friends.pending.title", comment: ""))
        .navigationBarTitleDisplayMode(.inline)
        .refreshable {
            await loadRequests()
        }
        .task {
            await loadRequests()
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

    private func loadRequests() async {
        isLoading = true
        do {
            requests = try await FriendService.shared.listPendingRequests()
        } catch {
            self.error = error.localizedDescription
        }
        isLoading = false
    }

    private func acceptRequest(_ request: FriendRequestResponse) async {
        do {
            _ = try await FriendService.shared.acceptRequest(id: request.id)
            requests.removeAll { $0.id == request.id }
        } catch {
            self.error = error.localizedDescription
        }
    }

    private func declineRequest(_ request: FriendRequestResponse) async {
        do {
            try await FriendService.shared.declineRequest(id: request.id)
            requests.removeAll { $0.id == request.id }
        } catch {
            self.error = error.localizedDescription
        }
    }
}
