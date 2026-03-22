import SwiftUI

// MARK: - Quick Action Sheet (Tous filter FAB)

struct CirclesQuickActionSheet: View {
    let onCreateCircle: () -> Void
    let onAddFriend: () -> Void

    var body: some View {
        VStack(spacing: OffriiTheme.spacingBase) {
            actionRow(
                icon: "person.2.fill",
                iconColor: OffriiTheme.accent,
                title: NSLocalizedString("create.createCircle", comment: ""),
                subtitle: NSLocalizedString("create.createCircleSubtitle", comment: "")
            ) {
                onCreateCircle()
            }

            actionRow(
                icon: "person.badge.plus",
                iconColor: OffriiTheme.accent,
                title: NSLocalizedString("create.addFriend", comment: ""),
                subtitle: NSLocalizedString("create.addFriendSubtitle", comment: "")
            ) {
                onAddFriend()
            }
        }
        .padding(.horizontal, OffriiTheme.spacingLG)
        .padding(.top, OffriiTheme.spacingLG)
    }

    private func actionRow(
        icon: String,
        iconColor: Color,
        title: String,
        subtitle: String,
        action: @escaping () -> Void
    ) -> some View {
        Button(action: action) {
            HStack(spacing: OffriiTheme.spacingBase) {
                RoundedRectangle(cornerRadius: OffriiTheme.cornerRadiusMD)
                    .fill(iconColor.opacity(0.12))
                    .frame(width: 48, height: 48)
                    .overlay(
                        Image(systemName: icon)
                            .font(.system(size: 20))
                            .foregroundColor(iconColor)
                    )

                VStack(alignment: .leading, spacing: OffriiTheme.spacingXXS) {
                    Text(title)
                        .font(OffriiTypography.headline)
                        .foregroundColor(OffriiTheme.text)
                    Text(subtitle)
                        .font(OffriiTypography.subheadline)
                        .foregroundColor(OffriiTheme.textSecondary)
                }

                Spacer()

                Image(systemName: "chevron.right")
                    .font(.system(size: 12, weight: .semibold))
                    .foregroundColor(OffriiTheme.textMuted)
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
        .buttonStyle(.plain)
    }
}
