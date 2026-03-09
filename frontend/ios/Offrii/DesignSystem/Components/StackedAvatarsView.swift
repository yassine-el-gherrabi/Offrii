import SwiftUI

struct StackedAvatarsView: View {
    let names: [String?]
    let totalCount: Int

    init(names: [String?], totalCount: Int) {
        self.names = Array(names.prefix(3))
        self.totalCount = totalCount
    }

    private var extraCount: Int {
        max(0, totalCount - 3)
    }

    var body: some View {
        HStack(spacing: -8) {
            ForEach(Array(names.enumerated()), id: \.offset) { index, name in
                AvatarView(name, size: .small)
                    .overlay(
                        Circle()
                            .stroke(OffriiTheme.card, lineWidth: 2)
                    )
                    .zIndex(Double(names.count - index))
            }

            if extraCount > 0 {
                Circle()
                    .fill(OffriiTheme.textMuted.opacity(0.2))
                    .frame(width: 28, height: 28)
                    .overlay(
                        Text("+\(extraCount)")
                            .font(.system(size: 10, weight: .semibold))
                            .foregroundColor(OffriiTheme.textSecondary)
                    )
                    .overlay(
                        Circle()
                            .stroke(OffriiTheme.card, lineWidth: 2)
                    )
            }
        }
    }
}
