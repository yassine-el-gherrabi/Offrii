import SwiftUI

// MARK: - AvatarStack

struct AvatarStack: View {
    let members: [(initials: String, color: Color)]
    var maxVisible: Int = 3
    var size: CGFloat = 28
    var overlap: CGFloat = 8

    private var visibleMembers: [(initials: String, color: Color)] {
        Array(members.prefix(maxVisible))
    }

    private var overflowCount: Int {
        max(0, members.count - maxVisible)
    }

    var body: some View {
        HStack(spacing: -overlap) {
            ForEach(Array(visibleMembers.enumerated()), id: \.offset) { _, member in
                avatarCircle(initials: member.initials, color: member.color)
            }

            if overflowCount > 0 {
                Text("+\(overflowCount)")
                    .font(.system(size: size * 0.4, weight: .semibold))
                    .foregroundColor(OffriiTheme.textSecondary)
                    .frame(width: size, height: size)
                    .background(OffriiTheme.surface)
                    .clipShape(Circle())
                    .overlay(
                        Circle()
                            .strokeBorder(OffriiTheme.card, lineWidth: 2)
                    )
            }
        }
    }

    private func avatarCircle(initials: String, color: Color) -> some View {
        Text(initials)
            .font(.system(size: size * 0.4, weight: .semibold))
            .foregroundColor(.white)
            .frame(width: size, height: size)
            .background(color)
            .clipShape(Circle())
            .overlay(
                Circle()
                    .strokeBorder(OffriiTheme.card, lineWidth: 2)
            )
    }
}
