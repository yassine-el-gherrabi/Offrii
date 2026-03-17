import SwiftUI

struct NotificationCenterView: View {
    @Environment(\.dismiss) private var dismiss
    @State private var notifications: [AppNotification] = []
    @State private var isLoading = false
    @State private var hasMore = true
    @State private var page = 1

    var body: some View {
        NavigationStack {
            ZStack {
                OffriiTheme.background.ignoresSafeArea()

                if isLoading && notifications.isEmpty {
                    SkeletonList(count: 6)
                        .padding(.top, OffriiTheme.spacingBase)
                } else if notifications.isEmpty {
                    Spacer()
                    OffriiEmptyState(
                        icon: "bell.slash",
                        title: NSLocalizedString("notifications.empty", comment: ""),
                        subtitle: NSLocalizedString("notifications.emptySubtitle", comment: "")
                    )
                    Spacer()
                } else {
                    ScrollView {
                        LazyVStack(spacing: 0) {
                            ForEach(notifications) { notif in
                                notificationRow(notif)
                                    .onAppear {
                                        if notif.id == notifications.last?.id && hasMore {
                                            Task { await loadMore() }
                                        }
                                    }

                                Divider()
                                    .padding(.leading, 56)
                                    .padding(.horizontal, OffriiTheme.spacingLG)
                            }
                        }
                    }
                }
            }
            .navigationTitle(NSLocalizedString("notifications.title", comment: ""))
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button(NSLocalizedString("common.ok", comment: "")) {
                        dismiss()
                    }
                }
                ToolbarItem(placement: .primaryAction) {
                    if !notifications.isEmpty {
                        Button {
                            Task { await markAllRead() }
                        } label: {
                            Image(systemName: "checkmark.circle")
                                .font(.system(size: 16))
                                .foregroundColor(OffriiTheme.primary)
                        }
                    }
                }
            }
            .task {
                await loadNotifications()
            }
        }
    }

    private func notificationRow(_ notif: AppNotification) -> some View {
        Button {
            Task {
                if !notif.read {
                    try? await NotificationCenterService.shared.markRead(id: notif.id)
                    if let idx = notifications.firstIndex(where: { $0.id == notif.id }) {
                        // Reload to get updated read state
                        notifications[idx] = AppNotification(
                            id: notif.id, type: notif.type, title: notif.title,
                            body: notif.body, read: true, circleId: notif.circleId,
                            itemId: notif.itemId, actorId: notif.actorId, createdAt: notif.createdAt
                        )
                    }
                }
                // TODO: Navigate to context (circle/item) based on notif type
            }
        } label: {
            HStack(spacing: OffriiTheme.spacingSM) {
                // Icon
                Image(systemName: notif.icon)
                    .font(.system(size: 18))
                    .foregroundColor(OffriiTheme.primary)
                    .frame(width: 36, height: 36)
                    .background(OffriiTheme.primary.opacity(0.1))
                    .clipShape(Circle())

                VStack(alignment: .leading, spacing: 2) {
                    Text(notif.title)
                        .font(OffriiTypography.body)
                        .fontWeight(notif.read ? .regular : .semibold)
                        .foregroundColor(OffriiTheme.text)

                    Text(notif.body)
                        .font(OffriiTypography.caption)
                        .foregroundColor(OffriiTheme.textSecondary)
                        .lineLimit(2)

                    Text(notif.createdAt, style: .relative)
                        .font(.system(size: 10))
                        .foregroundColor(OffriiTheme.textMuted)
                }

                Spacer()

                if !notif.read {
                    Circle()
                        .fill(OffriiTheme.primary)
                        .frame(width: 8, height: 8)
                }
            }
            .padding(.horizontal, OffriiTheme.spacingLG)
            .padding(.vertical, OffriiTheme.spacingSM)
            .background(notif.read ? Color.clear : OffriiTheme.primary.opacity(0.03))
        }
        .buttonStyle(.plain)
    }

    private func loadNotifications() async {
        isLoading = true
        do {
            let response = try await NotificationCenterService.shared.list(page: 1, limit: 20)
            notifications = response.data
            hasMore = response.data.count == 20
            page = 1
        } catch {}
        isLoading = false
    }

    private func loadMore() async {
        let nextPage = page + 1
        do {
            let response = try await NotificationCenterService.shared.list(page: nextPage, limit: 20)
            notifications.append(contentsOf: response.data)
            hasMore = response.data.count == 20
            page = nextPage
        } catch {}
    }

    private func markAllRead() async {
        do {
            try await NotificationCenterService.shared.markAllRead()
            notifications = notifications.map { notif in
                AppNotification(
                    id: notif.id, type: notif.type, title: notif.title,
                    body: notif.body, read: true, circleId: notif.circleId,
                    itemId: notif.itemId, actorId: notif.actorId, createdAt: notif.createdAt
                )
            }
            OffriiHaptics.success()
        } catch {}
    }
}
