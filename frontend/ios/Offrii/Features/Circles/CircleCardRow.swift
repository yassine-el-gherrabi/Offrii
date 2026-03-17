import NukeUI
import SwiftUI

// MARK: - CircleCardRow

struct CircleCardRow: View {
    let circle: OffriiCircle

    var body: some View {
        HStack(spacing: OffriiTheme.spacingMD) {
            avatarSection

            VStack(alignment: .leading, spacing: OffriiTheme.spacingXXS) {
                Text(circle.name ?? NSLocalizedString("circles.unnamed", comment: ""))
                    .font(OffriiTypography.headline)
                    .foregroundColor(OffriiTheme.text)
                    .lineLimit(1)

                if circle.isDirect {
                    directSubtitle
                } else {
                    groupSubtitle
                }

                if let activity = circle.lastActivity {
                    Text(activityText(activity))
                        .font(OffriiTypography.caption)
                        .foregroundColor(OffriiTheme.textSecondary)
                        .lineLimit(1)
                }
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
            radius: 6,
            x: 0,
            y: 2
        )
    }

    // MARK: - Avatar Section

    @ViewBuilder
    private var avatarSection: some View {
        if let imageUrl = circle.imageUrl, let url = URL(string: imageUrl) {
            // Circle has a custom image
            LazyImage(url: url) { state in
                if let image = state.image {
                    image
                        .resizable()
                        .aspectRatio(contentMode: .fill)
                        .frame(width: 44, height: 44)
                        .clipShape(Circle())
                } else {
                    AvatarView(circle.name, size: .medium)
                }
            }
        } else if circle.isDirect {
            // Single avatar for 1:1
            AvatarView(
                circle.memberNames.first ?? circle.name,
                size: .medium,
                url: circle.memberAvatars.first.flatMap { $0 }.flatMap { URL(string: $0) }
            )
        } else if circle.memberNames.isEmpty {
            AvatarView(circle.name, size: .medium)
        } else if circle.memberNames.count == 1 {
            AvatarView(
                circle.memberNames[0],
                size: .medium,
                url: circle.memberAvatars.first.flatMap { $0 }.flatMap { URL(string: $0) }
            )
        } else {
            ZStack {
                ForEach(
                    Array(circle.memberNames.prefix(3).enumerated()),
                    id: \.offset
                ) { idx, name in
                    let avatarUrl = idx < circle.memberAvatars.count
                        ? circle.memberAvatars[idx].flatMap { URL(string: $0) }
                        : nil
                    AvatarView(name, size: .small, url: avatarUrl)
                        .overlay(
                            Circle().strokeBorder(.white, lineWidth: 1.5)
                        )
                        .offset(x: CGFloat(idx) * 12 - 12)
                }
            }
            .frame(width: 52, height: 44)
        }
    }

    // MARK: - Direct (1:1) Subtitle

    @ViewBuilder
    private var directSubtitle: some View {
        if circle.unreservedItemCount > 0 {
            Text(String(
                format: NSLocalizedString("circles.detail.wishCount", comment: ""),
                circle.unreservedItemCount
            ))
            .font(OffriiTypography.caption)
            .fontWeight(.medium)
            .foregroundColor(OffriiTheme.primary)
        } else {
            Text(NSLocalizedString("circles.noUnreserved", comment: ""))
                .font(OffriiTypography.caption)
                .foregroundColor(OffriiTheme.textMuted)
        }
    }

    // MARK: - Group Subtitle

    @ViewBuilder
    private var groupSubtitle: some View {
        HStack(spacing: OffriiTheme.spacingSM) {
            Text(String(
                format: NSLocalizedString("circles.memberCount", comment: ""),
                circle.memberCount
            ))
            .font(OffriiTypography.caption)
            .foregroundColor(OffriiTheme.textMuted)

            if circle.unreservedItemCount > 0 {
                Text("·")
                    .foregroundColor(OffriiTheme.textMuted)
                Text(String(
                    format: NSLocalizedString("circles.unreservedCount", comment: ""),
                    circle.unreservedItemCount
                ))
                .font(OffriiTypography.caption)
                .fontWeight(.medium)
                .foregroundColor(OffriiTheme.primary)
            }
        }
    }

    // MARK: - Activity Text

    private func activityText(_ activity: String) -> String {
        if circle.isDirect {
            return activity
        }
        // For groups, the activity string may already contain the sender prefix
        return activity
    }
}
