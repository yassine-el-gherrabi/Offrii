import SwiftUI

// MARK: - CircleCardRow

struct CircleCardRow: View {
    let circle: OffriiCircle

    private var circleIcon: String {
        circle.isDirect ? "bubble.left.fill" : "person.2.fill"
    }

    private var memberCountText: String {
        if circle.isDirect {
            return NSLocalizedString("circles.oneToOne", comment: "")
        }
        return String(
            format: NSLocalizedString("circles.memberCount", comment: ""),
            circle.memberCount
        )
    }

    private var unreservedText: String? {
        guard circle.unreservedItemCount > 0 else { return nil }
        return String(
            format: NSLocalizedString("circles.unreservedCount", comment: ""),
            circle.unreservedItemCount
        )
    }

    var body: some View {
        HStack(spacing: OffriiTheme.spacingMD) {
            avatarStack

            VStack(alignment: .leading, spacing: OffriiTheme.spacingXXS) {
                Text(circle.name ?? NSLocalizedString("circles.unnamed", comment: ""))
                    .font(OffriiTypography.headline)
                    .foregroundColor(OffriiTheme.text)
                    .lineLimit(1)

                HStack(spacing: OffriiTheme.spacingSM) {
                    Text(memberCountText)
                        .font(OffriiTypography.caption)
                        .foregroundColor(OffriiTheme.textMuted)

                    if let unreserved = unreservedText {
                        Text("·")
                            .foregroundColor(OffriiTheme.textMuted)
                        Text(unreserved)
                            .font(OffriiTypography.caption)
                            .fontWeight(.medium)
                            .foregroundColor(OffriiTheme.primary)
                    }
                }

                if let activity = circle.lastActivity {
                    Text(activity)
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

    // MARK: - Avatar Stack

    @ViewBuilder
    private var avatarStack: some View {
        if circle.memberNames.isEmpty {
            AvatarView(circle.name, size: .medium)
        } else if circle.memberNames.count == 1 {
            AvatarView(circle.memberNames[0], size: .medium)
        } else {
            ZStack {
                ForEach(
                    Array(circle.memberNames.prefix(3).enumerated()),
                    id: \.offset
                ) { idx, name in
                    AvatarView(name, size: .small)
                        .overlay(
                            Circle().strokeBorder(.white, lineWidth: 1.5)
                        )
                        .offset(x: CGFloat(idx) * 12 - 12)
                }
            }
            .frame(width: 52, height: 44)
        }
    }
}
