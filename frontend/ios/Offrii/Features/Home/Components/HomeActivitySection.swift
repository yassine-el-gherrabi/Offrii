import SwiftUI

// MARK: - HomeActivitySection

struct HomeActivitySection: View {
    let notifications: [AppNotification]
    @State private var showNotificationCenter = false

    var body: some View {
        VStack(alignment: .leading, spacing: OffriiTheme.spacingSM) {
            // Section title + "Tout voir"
            HStack {
                Text(NSLocalizedString("home.activity.title", comment: ""))
                    .font(OffriiTypography.headline)
                    .foregroundColor(OffriiTheme.text)

                Spacer()

                Button {
                    showNotificationCenter = true
                } label: {
                    HStack(spacing: OffriiTheme.spacingXXS) {
                        Text(NSLocalizedString("home.activity.seeAll", comment: ""))
                            .font(OffriiTypography.subheadline)
                        Image(systemName: "arrow.right")
                            .font(.system(size: 12))
                    }
                    .foregroundColor(OffriiTheme.primary)
                }
            }

            // Notification rows
            VStack(spacing: 0) {
                ForEach(Array(notifications.prefix(5).enumerated()), id: \.element.id) { index, notif in
                    activityRow(notif)

                    if index < min(notifications.count, 5) - 1 {
                        Divider()
                            .padding(.leading, 52)
                    }
                }
            }
            .padding(OffriiTheme.spacingBase)
            .background(OffriiTheme.card)
            .cornerRadius(OffriiTheme.cornerRadiusLG)
            .shadow(
                color: OffriiTheme.cardShadowColor,
                radius: OffriiTheme.cardShadowRadius,
                x: 0,
                y: OffriiTheme.cardShadowY
            )
        }
        .sheet(isPresented: $showNotificationCenter) {
            NotificationCenterView()
        }
    }

    // MARK: - Activity Row

    private func activityRow(_ notif: AppNotification) -> some View {
        HStack(spacing: OffriiTheme.spacingSM) {
            Image(systemName: notif.icon)
                .font(.system(size: 14))
                .foregroundColor(OffriiTheme.primary)
                .frame(width: 32, height: 32)
                .background(OffriiTheme.primary.opacity(0.1))
                .clipShape(Circle())

            VStack(alignment: .leading, spacing: 2) {
                Text(notif.localizedBody)
                    .font(.system(size: 13, weight: notif.read ? .regular : .semibold))
                    .foregroundColor(OffriiTheme.text)
                    .lineLimit(2)

                Text(notif.createdAt, style: .relative)
                    .font(.system(size: 11))
                    .foregroundColor(OffriiTheme.textMuted)
            }

            Spacer()

            if !notif.read {
                Circle()
                    .fill(OffriiTheme.primary)
                    .frame(width: 6, height: 6)
            }
        }
        .padding(.vertical, OffriiTheme.spacingXS)
    }
}
