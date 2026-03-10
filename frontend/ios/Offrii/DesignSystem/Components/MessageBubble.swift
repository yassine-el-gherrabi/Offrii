import SwiftUI

// MARK: - MessageBubble

struct MessageBubble: View {
    let text: String
    let senderName: String?
    let timestamp: Date
    let isMine: Bool

    var body: some View {
        VStack(alignment: isMine ? .trailing : .leading, spacing: 2) {
            // Sender name (only for others' messages)
            if !isMine, let senderName {
                Text(senderName)
                    .font(OffriiTypography.caption2)
                    .foregroundColor(OffriiTheme.textMuted)
                    .padding(.horizontal, OffriiTheme.spacingXS)
            }

            // Bubble
            VStack(alignment: isMine ? .trailing : .leading, spacing: OffriiTheme.spacingXS) {
                Text(text)
                    .font(OffriiTypography.body)
                    .foregroundColor(isMine ? .white : OffriiTheme.text)

                Text(timestamp, style: .time)
                    .font(OffriiTypography.caption2)
                    .foregroundColor(isMine ? .white.opacity(0.7) : OffriiTheme.textMuted)
            }
            .padding(.horizontal, OffriiTheme.spacingMD)
            .padding(.vertical, OffriiTheme.spacingSM + 2)
            .background(isMine ? OffriiTheme.primary : OffriiTheme.cardSurface)
            .cornerRadius(OffriiTheme.cornerRadiusMD)
        }
        .frame(maxWidth: .infinity, alignment: isMine ? .trailing : .leading)
        .padding(isMine ? .leading : .trailing, OffriiTheme.spacingXL)
    }
}
