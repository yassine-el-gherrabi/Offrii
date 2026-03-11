import SwiftUI

// MARK: - Tooltip Arrow Direction

enum TooltipArrow {
    case top, bottom, left, right
}

// MARK: - OffriiTooltip

struct OffriiTooltip: View {
    let message: String
    var arrow: TooltipArrow = .bottom
    let onDismiss: () -> Void

    @State private var appeared = false

    var body: some View {
        VStack(spacing: 0) {
            if arrow == .bottom || arrow == .left || arrow == .right {
                tooltipBody
            }

            if arrow == .top {
                arrowShape
                    .rotationEffect(.degrees(180))
                    .frame(width: 16, height: 8)
            }

            if arrow == .top {
                tooltipBody
            }

            if arrow == .bottom {
                arrowShape
                    .frame(width: 16, height: 8)
            }
        }
        .scaleEffect(appeared ? 1 : 0.8)
        .opacity(appeared ? 1 : 0)
        .onAppear {
            withAnimation(OffriiAnimation.bouncy) {
                appeared = true
            }
            // Auto-dismiss after 8s
            DispatchQueue.main.asyncAfter(deadline: .now() + 8) {
                dismiss()
            }
        }
    }

    private var tooltipBody: some View {
        VStack(alignment: .leading, spacing: OffriiTheme.spacingSM) {
            Text(message)
                .font(OffriiTypography.subheadline)
                .foregroundColor(.white)

            Button {
                dismiss()
            } label: {
                Text(NSLocalizedString("tooltip.gotIt", comment: ""))
                    .font(OffriiTypography.footnote)
                    .fontWeight(.semibold)
                    .foregroundColor(OffriiTheme.primaryLight)
            }
        }
        .padding(.horizontal, OffriiTheme.spacingBase)
        .padding(.vertical, OffriiTheme.spacingMD)
        .background(Color(white: 0.15).opacity(0.95))
        .cornerRadius(OffriiTheme.cornerRadiusMD)
    }

    private var arrowShape: some View {
        Triangle()
            .fill(Color(white: 0.15).opacity(0.95))
    }

    private func dismiss() {
        withAnimation(OffriiAnimation.snappy) {
            appeared = false
        }
        DispatchQueue.main.asyncAfter(deadline: .now() + 0.25) {
            onDismiss()
        }
    }
}

// MARK: - Triangle Shape

struct Triangle: Shape {
    func path(in rect: CGRect) -> Path {
        var path = Path()
        path.move(to: CGPoint(x: rect.midX, y: rect.maxY))
        path.addLine(to: CGPoint(x: rect.maxX, y: rect.minY))
        path.addLine(to: CGPoint(x: rect.minX, y: rect.minY))
        path.closeSubpath()
        return path
    }
}
