import SwiftUI

// MARK: - OffriiSpinner (3 bouncing dots)

struct OffriiSpinner: View {
    let color: Color
    @State private var animating = false

    init(color: Color = .white) {
        self.color = color
    }

    var body: some View {
        HStack(spacing: 4) {
            ForEach(0..<3, id: \.self) { index in
                Circle()
                    .fill(color)
                    .frame(width: 6, height: 6)
                    .offset(y: animating ? -4 : 4)
                    .animation(
                        OffriiAnimation.bouncy
                            .repeatForever(autoreverses: true)
                            .delay(Double(index) * 0.15),
                        value: animating
                    )
            }
        }
        .onAppear { animating = true }
    }
}
