import SwiftUI

// MARK: - MessageBubble

struct MessageBubble: View {
    let text: String
    let senderName: String?
    let timestamp: Date
    let isMine: Bool
    let isNew: Bool

    init(text: String, senderName: String?, timestamp: Date, isMine: Bool, isNew: Bool = false) {
        self.text = text
        self.senderName = senderName
        self.timestamp = timestamp
        self.isMine = isMine
        self.isNew = isNew
    }

    var body: some View {
        VStack(alignment: isMine ? .trailing : .leading, spacing: 2) {
            // Sender name (only for others' messages)
            if !isMine, let senderName {
                Text(senderName)
                    .font(OffriiTypography.caption2)
                    .foregroundColor(OffriiTheme.textMuted)
                    .padding(.horizontal, OffriiTheme.spacingXS)
            }

            // Bubble with asymmetric corners
            VStack(alignment: isMine ? .trailing : .leading, spacing: OffriiTheme.spacingXS) {
                Text(text)
                    .font(OffriiTypography.body)
                    .foregroundColor(isMine ? .white : OffriiTheme.text)

                Text(timestamp, style: .time)
                    .font(OffriiTypography.caption2)
                    .foregroundColor(isMine ? .white.opacity(0.7) : OffriiTheme.textMuted)
            }
            .padding(.horizontal, OffriiTheme.spacingBase)
            .padding(.vertical, OffriiTheme.spacingMD)
            .background(isMine ? OffriiTheme.primary : OffriiTheme.surface)
            .clipShape(BubbleShape(isMine: isMine))
        }
        .frame(maxWidth: UIScreen.main.bounds.width * 0.75, alignment: isMine ? .trailing : .leading)
        .frame(maxWidth: .infinity, alignment: isMine ? .trailing : .leading)
        .padding(isMine ? .leading : .trailing, OffriiTheme.spacingXL)
        .transition(isNew ? .scale.combined(with: .opacity) : .identity)
    }
}

// MARK: - Bubble Shape (asymmetric corners)

struct BubbleShape: Shape {
    let isMine: Bool

    func path(in rect: CGRect) -> Path {
        let radius: CGFloat = 16
        let smallRadius: CGFloat = 4

        let topLeft = isMine ? radius : smallRadius
        let topRight = isMine ? smallRadius : radius
        let bottomLeft = radius
        let bottomRight = radius

        return Path { path in
            path.move(to: CGPoint(x: rect.minX + topLeft, y: rect.minY))
            path.addLine(to: CGPoint(x: rect.maxX - topRight, y: rect.minY))
            path.addQuadCurve(to: CGPoint(x: rect.maxX, y: rect.minY + topRight),
                              control: CGPoint(x: rect.maxX, y: rect.minY))
            path.addLine(to: CGPoint(x: rect.maxX, y: rect.maxY - bottomRight))
            path.addQuadCurve(to: CGPoint(x: rect.maxX - bottomRight, y: rect.maxY),
                              control: CGPoint(x: rect.maxX, y: rect.maxY))
            path.addLine(to: CGPoint(x: rect.minX + bottomLeft, y: rect.maxY))
            path.addQuadCurve(to: CGPoint(x: rect.minX, y: rect.maxY - bottomLeft),
                              control: CGPoint(x: rect.minX, y: rect.maxY))
            path.addLine(to: CGPoint(x: rect.minX, y: rect.minY + topLeft))
            path.addQuadCurve(to: CGPoint(x: rect.minX + topLeft, y: rect.minY),
                              control: CGPoint(x: rect.minX, y: rect.minY))
            path.closeSubpath()
        }
    }
}
