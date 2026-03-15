import SwiftUI

// MARK: - CircleCardRow

struct CircleCardRow: View {
    let circle: OffriiCircle

    private var circleIcon: String {
        circle.isDirect ? "bubble.left.fill" : "person.2.fill"
    }

    private var memberCountText: String {
        if circle.isDirect {
            return "1-to-1"
        }
        return String(format: NSLocalizedString("circles.memberCount", comment: ""), circle.memberCount)
    }

    var body: some View {
        HStack(spacing: OffriiTheme.spacingMD) {
            // Circle icon with member count overlay
            ZStack {
                Circle()
                    .fill(
                        LinearGradient(
                            colors: [OffriiTheme.primary.opacity(0.25), OffriiTheme.accent.opacity(0.15)],
                            startPoint: .topLeading,
                            endPoint: .bottomTrailing
                        )
                    )
                    .frame(width: 52, height: 52)

                Image(systemName: circleIcon)
                    .font(.system(size: 20))
                    .foregroundColor(OffriiTheme.primary)
            }

            // Text content
            VStack(alignment: .leading, spacing: OffriiTheme.spacingXXS) {
                Text(circle.name ?? NSLocalizedString("circles.unnamed", comment: ""))
                    .font(OffriiTypography.headline)
                    .foregroundColor(OffriiTheme.text)
                    .lineLimit(1)

                Text(memberCountText)
                    .font(OffriiTypography.caption)
                    .foregroundColor(OffriiTheme.textMuted)
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
}
