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
            ZStack {
                Circle()
                    .fill(
                        LinearGradient(
                            colors: [
                                OffriiTheme.primary.opacity(0.25),
                                OffriiTheme.accent.opacity(0.15),
                            ],
                            startPoint: .topLeading,
                            endPoint: .bottomTrailing
                        )
                    )
                    .frame(width: 48, height: 48)

                Image(systemName: circleIcon)
                    .font(.system(size: 18))
                    .foregroundColor(OffriiTheme.primary)
            }
        } else {
            ZStack {
                ForEach(Array(circle.memberNames.prefix(3).enumerated()), id: \.offset) { idx, name in
                    let initial = name.prefix(1).uppercased()
                    Text(initial)
                        .font(.system(size: 12, weight: .bold))
                        .foregroundColor(.white)
                        .frame(width: 28, height: 28)
                        .background(avatarColor(for: idx))
                        .clipShape(Circle())
                        .overlay(
                            Circle().strokeBorder(.white, lineWidth: 1.5)
                        )
                        .offset(x: CGFloat(idx) * 10 - 10)
                }
            }
            .frame(width: 48, height: 48)
        }
    }

    private func avatarColor(for index: Int) -> Color {
        let colors = [OffriiTheme.primary, OffriiTheme.accent, OffriiTheme.secondary]
        return colors[index % colors.count]
    }
}
